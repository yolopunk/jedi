import { ref } from 'vue'
import type { PodcastEpisode } from '@/api/podcast'

// Global state
const currentPlaying = ref<PodcastEpisode | null>(null)
const currentPlayingSubUrl = ref<string | null>(null) // To track which subscription the episode belongs to
const isPaused = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const audioRef = ref<HTMLAudioElement | null>(null)

// Handler references for cleanup
let pauseHandler: (() => void) | null = null
let playHandler: (() => void) | null = null
let endedHandler: (() => void) | null = null
let timeUpdateHandler: (() => void) | null = null
let loadedMetadataHandler: (() => void) | null = null

// Actions
function playEpisode(ep: PodcastEpisode, subUrl?: string) {
  if (currentPlaying.value?.audio_url === ep.audio_url) {
    togglePlay()
    return
  }

  currentPlaying.value = ep
  if (subUrl) {
    currentPlayingSubUrl.value = subUrl
  }
  isPaused.value = false
  currentTime.value = 0
  duration.value = 0 // Reset duration until metadata loads
}

function togglePlay() {
  if (!audioRef.value) return

  if (audioRef.value.paused) {
    audioRef.value.play().catch(e => console.error('Play error:', e))
    isPaused.value = false
  } else {
    audioRef.value.pause()
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
}

function seek(seconds: number) {
  if (audioRef.value) {
    audioRef.value.currentTime = Math.min(
      Math.max(audioRef.value.currentTime + seconds, 0),
      audioRef.value.duration
    )
  }
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

  el.addEventListener('pause', pauseHandler)
  el.addEventListener('play', playHandler)
  el.addEventListener('ended', endedHandler)
  el.addEventListener('timeupdate', timeUpdateHandler)
  el.addEventListener('loadedmetadata', loadedMetadataHandler)
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

  audioRef.value = null
}

export function useAudioPlayer() {
  return {
    currentPlaying,
    currentPlayingSubUrl,
    isPaused,
    currentTime,
    duration,
    audioRef,
    playEpisode,
    togglePlay,
    stop,
    seek,
    setAudioRef,
    cleanupAudioRef,
  }
}
