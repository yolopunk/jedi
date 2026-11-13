import { nextTick, ref } from 'vue'
import type { PodcastEpisode } from '@/api/podcast'
import { useStorage } from '@/composables/useStorage'

// Global state
const currentPlaying = ref<PodcastEpisode | null>(null)
const currentPlayingSubUrl = ref<string | null>(null) // To track which subscription the episode belongs to
const isPaused = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const audioRef = ref<HTMLAudioElement | null>(null)
// 播放失败原因（空音频地址 / 加载失败 / 被自动播放策略拦截），由 UI 层提示用户
const playbackError = ref<string | null>(null)

// 音量（0~1）与静音状态，跨会话持久化
const volume = ref(1)
const isMuted = ref(false)

const VOLUME_KEY = 'podcastVolume'
const MUTE_KEY = 'podcastMuted'
let volumeLoaded = false
let persistTimer: ReturnType<typeof setTimeout> | null = null

// Handler references for cleanup
let pauseHandler: (() => void) | null = null
let playHandler: (() => void) | null = null
let endedHandler: (() => void) | null = null
let timeUpdateHandler: (() => void) | null = null
let loadedMetadataHandler: (() => void) | null = null
let errorHandler: (() => void) | null = null

const MEDIA_ERROR_MESSAGES: Record<number, string> = {
  1: '播放被中断',
  2: '网络错误，音频下载失败',
  3: '音频解码失败',
  4: '音频地址不可用或格式不支持',
}

/**
 * 真正发起播放。不依赖 <audio autoplay>，这样被拦截时能拿到 reject 的原因并提示用户。
 */
function startPlayback() {
  const el = audioRef.value
  if (!el) return

  const result = el.play()
  // 老内核返回 undefined，新内核返回 Promise
  if (result && typeof result.then === 'function') {
    result
      .then(() => {
        playbackError.value = null
      })
      .catch((err: unknown) => {
        isPaused.value = true
        const name = err instanceof Error ? err.name : ''
        playbackError.value =
          name === 'NotAllowedError'
            ? '播放被系统拦截（自动播放限制），请再点一次播放'
            : `播放失败：${err instanceof Error ? err.message : String(err)}`
        console.error('[podcast] play failed:', err, el.src)
      })
  }
}

// Actions
function playEpisode(ep: PodcastEpisode, subUrl?: string) {
  if (!ep.audio_url) {
    playbackError.value = `《${ep.title}》没有可用的音频地址`
    return
  }

  if (currentPlaying.value?.audio_url === ep.audio_url) {
    togglePlay()
    return
  }

  playbackError.value = null
  currentPlaying.value = ep
  if (subUrl) {
    currentPlayingSubUrl.value = subUrl
  }
  isPaused.value = false
  currentTime.value = 0
  duration.value = 0 // Reset duration until metadata loads

  // 等 :src 绑定刷新到 DOM 后再播放
  nextTick(() => startPlayback())
}

function togglePlay() {
  const el = audioRef.value
  if (!el) return

  if (el.paused) {
    startPlayback()
  } else {
    el.pause()
    isPaused.value = true
  }
}

function stop() {
  if (audioRef.value) {
    audioRef.value.pause()
    audioRef.value.currentTime = 0
  }
  currentPlaying.value = null
  currentPlayingSubUrl.value = null
  isPaused.value = false
  playbackError.value = null
}

function seek(seconds: number) {
  const el = audioRef.value
  if (!el) return

  const target = Math.max(el.currentTime + seconds, 0)
  // duration 可能是 NaN（未加载）或 Infinity（直播流），只在有限时做上界裁剪
  el.currentTime = Number.isFinite(el.duration) ? Math.min(target, el.duration) : target
}

// ==================== Volume ====================

function clampVolume(v: number) {
  return Math.min(1, Math.max(0, v))
}

function applyVolume() {
  const el = audioRef.value
  if (!el) return
  el.volume = volume.value
  el.muted = isMuted.value
}

async function loadVolumePrefs() {
  if (volumeLoaded) return
  volumeLoaded = true
  const { getItem } = useStorage()
  const savedVolume = await getItem<number>(VOLUME_KEY)
  const savedMuted = await getItem<boolean>(MUTE_KEY)
  if (typeof savedVolume === 'number' && Number.isFinite(savedVolume)) {
    volume.value = clampVolume(savedVolume)
  }
  if (typeof savedMuted === 'boolean') {
    isMuted.value = savedMuted
  }
  applyVolume()
}

function persistVolumePrefs() {
  if (persistTimer) clearTimeout(persistTimer)
  // 拖动滑块时高频触发，稍作合并再落盘
  persistTimer = setTimeout(() => {
    const { setItem } = useStorage()
    void setItem(VOLUME_KEY, volume.value)
    void setItem(MUTE_KEY, isMuted.value)
  }, 400)
}

function setVolume(value: number) {
  volume.value = clampVolume(value)
  if (volume.value > 0 && isMuted.value) {
    isMuted.value = false
  }
  applyVolume()
  persistVolumePrefs()
}

function toggleMute() {
  const effectivelyMuted = isMuted.value || volume.value === 0
  if (effectivelyMuted) {
    isMuted.value = false
    // 之前拖到 0 的话，取消静音时恢复一个可听的音量
    if (volume.value === 0) {
      volume.value = 0.8
    }
  } else {
    isMuted.value = true
  }
  applyVolume()
  persistVolumePrefs()
}

function setAudioRef(el: HTMLAudioElement) {
  audioRef.value = el

  // Attach listeners with stored references
  pauseHandler = () => {
    isPaused.value = true
  }
  playHandler = () => {
    isPaused.value = false
  }
  endedHandler = () => {
    isPaused.value = true
  }
  timeUpdateHandler = () => {
    currentTime.value = el.currentTime
  }
  loadedMetadataHandler = () => {
    duration.value = el.duration
  }
  errorHandler = () => {
    // 没有在播节目时（src 被清空 / 停止播放）浏览器也会触发 error，忽略
    if (!currentPlaying.value || !el.getAttribute('src')) return
    isPaused.value = true
    const code = el.error?.code
    const detail = code ? MEDIA_ERROR_MESSAGES[code] || `错误码 ${code}` : '未知错误'
    playbackError.value = `音频加载失败：${detail}`
    console.error('[podcast] audio error:', code, el.error, el.src)
  }

  el.addEventListener('pause', pauseHandler)
  el.addEventListener('play', playHandler)
  el.addEventListener('ended', endedHandler)
  el.addEventListener('timeupdate', timeUpdateHandler)
  el.addEventListener('loadedmetadata', loadedMetadataHandler)
  el.addEventListener('error', errorHandler)

  // 恢复上次的音量/静音状态
  void loadVolumePrefs()
  applyVolume()
}

function cleanupAudioRef() {
  if (!audioRef.value) return

  const el = audioRef.value
  if (pauseHandler) {
    el.removeEventListener('pause', pauseHandler)
    pauseHandler = null
  }
  if (playHandler) {
    el.removeEventListener('play', playHandler)
    playHandler = null
  }
  if (endedHandler) {
    el.removeEventListener('ended', endedHandler)
    endedHandler = null
  }
  if (timeUpdateHandler) {
    el.removeEventListener('timeupdate', timeUpdateHandler)
    timeUpdateHandler = null
  }
  if (loadedMetadataHandler) {
    el.removeEventListener('loadedmetadata', loadedMetadataHandler)
    loadedMetadataHandler = null
  }
  if (errorHandler) {
    el.removeEventListener('error', errorHandler)
    errorHandler = null
  }

  audioRef.value = null
}

function clearPlaybackError() {
  playbackError.value = null
}

export function useAudioPlayer() {
  return {
    currentPlaying,
    currentPlayingSubUrl,
    isPaused,
    currentTime,
    duration,
    audioRef,
    playbackError,
    volume,
    isMuted,
    playEpisode,
    togglePlay,
    stop,
    seek,
    setVolume,
    toggleMute,
    setAudioRef,
    cleanupAudioRef,
    clearPlaybackError,
  }
}
