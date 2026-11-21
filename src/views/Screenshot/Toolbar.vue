<template>
  <div class="ctrl-root" :class="`mode-${mode}`">
    <!-- 滚动截长图控制条：随滚自动捕获，这里只做完成/取消/手动补拍 -->
    <div v-if="mode === 'scroll'" class="ctrl-bar" role="toolbar" :aria-label="t('screenshot.scrolling.title')">
      <div class="frame-counter" :title="t('screenshot.scrolling.frames', { count: frameCount })">
        <span class="fc-ring" :class="{ lost: !matchOk }"><span class="fc-dot" aria-hidden="true"></span></span>
        <span class="fc-label">
          <b>{{ stitchedHeightLabel }}</b>
          <span v-if="!matchOk" class="fc-lost">&nbsp;{{ t('screenshot.scrolling.matchLostShort') }}</span>
        </span>
      </div>

      <span class="cb-divider" aria-hidden="true"></span>

      <button class="cb-btn" type="button" :disabled="working" @click="captureFrame">
        {{ t('screenshot.scrolling.capture') }}
      </button>
      <button class="cb-btn cb-primary" type="button" :disabled="working" @click="finish">
        {{ t('screenshot.scrolling.finish') }}
      </button>
      <button class="cb-btn cb-ghost" type="button" :aria-label="t('screenshot.scrolling.cancel')" :disabled="working" @click="cancel">
        {{ t('common.cancel') }}
      </button>
    </div>

    <!-- 录屏控制条 -->
    <div v-else-if="mode === 'record'" class="ctrl-bar" role="toolbar" :aria-label="t('screenshot.record.recording')">
      <span class="rec-indicator" :title="t('screenshot.record.maxDuration')">
        <span class="rec-dot" :class="{ paused: paused }" aria-hidden="true"></span>
        <span class="rec-label">{{ paused ? 'PAUSE' : 'REC' }}</span>
        <span class="rec-elapsed">{{ elapsedLabel }}</span>
      </span>
      <button class="cb-btn cb-tonal" type="button" :disabled="working" @click="togglePause">
        <v-icon :icon="paused ? mdiPlayCircleOutline : mdiPauseCircleOutline" size="17" aria-hidden="true" />
        {{ paused ? t('screenshot.record.resume') : t('screenshot.record.pause') }}
      </button>
      <button class="cb-btn cb-stop" type="button" :disabled="working" @click="stop">
        <v-icon :icon="mdiStopCircleOutline" size="17" aria-hidden="true" />
        {{ t('screenshot.record.stop') }}
      </button>
    </div>

    <div v-if="errorText" class="ctrl-error" role="alert">{{ errorText }}</div>
  </div>
</template>

<script setup lang="ts">
import { mdiPauseCircleOutline, mdiPlayCircleOutline, mdiStopCircleOutline } from '@mdi/js'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  screenshotScrollCancel,
  screenshotScrollCapture,
  screenshotScrollFinish,
  screenshotToolbarInfo,
  stopRegionRecord,
  toggleRegionRecordPause,
} from '@/api/screenshot'

const { t } = useI18n()
const window_ = getCurrentWindow()

const mode = ref<'scroll' | 'record' | ''>('')
const frameCount = ref(0)
const stitchedHeight = ref(0)
const matchOk = ref(true)
const working = ref(false)
const errorText = ref('')
const elapsedMs = ref(0)
const paused = ref(false)
let maxDurationMs = 0
let recordStartAt = 0
let pausedMs = 0
let pauseAnchor = 0
let tickTimer: number | null = null
let errorTimer: number | null = null
let stopping = false
let autoStopFired = false
const unlisteners: UnlistenFn[] = []

const stitchedHeightLabel = computed(() =>
  t('screenshot.scrolling.height', { height: stitchedHeight.value })
)

function showError(message: string, sticky = false) {
  errorText.value = message
  if (errorTimer) window.clearTimeout(errorTimer)
  if (!sticky) {
    errorTimer = window.setTimeout(() => {
      errorText.value = ''
    }, 3000)
  }
}

const elapsedLabel = computed(() => {
  const total = Math.floor(elapsedMs.value / 1000)
  const m = Math.floor(total / 60)
  const s = total % 60
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
})

async function closeSelf() {
  try {
    await window_.close()
  } catch {
    /* 窗口可能已被后端关闭 */
  }
}

// 同一按键会走两条通路：窗口内 keydown + 按钮点击。
// 按动作类型做时间闩锁去重，避免一次操作触发两次。
const lastActionAt: Record<string, number> = {}

function isDuplicateAction(action: string) {
  const now = Date.now()
  if (now - (lastActionAt[action] ?? 0) < 300) return true
  lastActionAt[action] = now
  return false
}

async function captureFrame() {
  if (mode.value !== 'scroll' || working.value) return
  if (isDuplicateAction('capture')) return
  working.value = true
  try {
    const status = await screenshotScrollCapture()
    frameCount.value = status.frameCount
  } catch (err) {
    showError(t('screenshot.scrolling.failed', { message: String(err) }))
  } finally {
    working.value = false
  }
}

async function finish() {
  if (mode.value !== 'scroll' || working.value) return
  if (isDuplicateAction('finish')) return
  working.value = true
  try {
    await screenshotScrollFinish()
    // 成功后后端会关闭本窗口；兜底自关
    window.setTimeout(closeSelf, 1500)
  } catch (err) {
    working.value = false
    showError(t('screenshot.scrolling.failed', { message: String(err) }))
  }
}

async function cancel() {
  if (mode.value !== 'scroll' || working.value) return
  if (isDuplicateAction('cancel')) return
  working.value = true
  try {
    await screenshotScrollCancel()
    window.setTimeout(closeSelf, 1500)
  } catch (err) {
    working.value = false
    showError(t('screenshot.scrolling.failed', { message: String(err) }))
  }
}

async function togglePause() {
  if (mode.value !== 'record' || working.value) return
  if (isDuplicateAction('pause')) return
  try {
    paused.value = await toggleRegionRecordPause()
  } catch (err) {
    showError(t('screenshot.record.failed', { message: String(err) }))
  }
}

async function stop() {
  if (mode.value !== 'record' || working.value || stopping) return
  stopping = true
  working.value = true
  try {
    await stopRegionRecord()
    window.setTimeout(closeSelf, 1500)
  } catch (err) {
    stopping = false
    working.value = false
    showError(t('screenshot.record.failed', { message: String(err) }))
  }
}

function onKeyDown(event: KeyboardEvent) {
  // 仅在本窗口持有焦点时生效（点击底层应用后焦点离开，全局不再吞键）
  if (mode.value === 'scroll') {
    if (event.code === 'Space') {
      event.preventDefault()
      captureFrame()
    } else if (event.key === 'Enter') {
      event.preventDefault()
      finish()
    } else if (event.key === 'Escape') {
      event.preventDefault()
      cancel()
    }
  } else if (mode.value === 'record' && event.key === 'Escape') {
    event.preventDefault()
    stop()
  }
}

/** 按当前后端会话初始化/重置 UI（新窗口挂载或复用窗口被重新激活时） */
async function initFromBackend() {
  try {
    const info = await screenshotToolbarInfo()
    mode.value = info.mode === 'scroll' || info.mode === 'record' ? info.mode : ''
    frameCount.value = info.frameCount
    maxDurationMs = info.maxDurationMs
    paused.value = info.paused
    stopping = false
    autoStopFired = false
    working.value = false
    if (tickTimer) {
      window.clearInterval(tickTimer)
      tickTimer = null
    }
    if (info.mode === 'record') {
      // 用后端起始时刻对表，避免窗口复用导致计时漂移
      recordStartAt = info.startedAtMs > 0 ? info.startedAtMs : Date.now()
      pausedMs = 0
      pauseAnchor = 0
      tickTimer = window.setInterval(() => {
        const now = Date.now()
        // 有效录制时长 = 墙钟时长 - 暂停累计（与后端视频时间轴一致）
        if (paused.value) {
          if (pauseAnchor === 0) pauseAnchor = now
        } else if (pauseAnchor > 0) {
          pausedMs += now - pauseAnchor
          pauseAnchor = 0
        }
        elapsedMs.value = Math.max(0, now - recordStartAt - pausedMs)
        if (!autoStopFired && maxDurationMs > 0 && elapsedMs.value >= maxDurationMs) {
          // 到达最长时长：由这里触发 stop 命令统一收尾（后端只兜底停线程）
          autoStopFired = true
          stop()
        }
      }, 250)
    } else {
      elapsedMs.value = 0
    }
  } catch {
    // 没有活动会话：窗口是残留的，直接关闭
    await closeSelf()
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown)
  await initFromBackend()
  if (mode.value === '') return

  unlisteners.push(
    await listen<{ frameCount: number; stitchedHeight: number; matchOk: boolean }>(
      'scroll-frame',
      event => {
        frameCount.value = event.payload.frameCount
        stitchedHeight.value = event.payload.stitchedHeight
        matchOk.value = event.payload.matchOk
      }
    )
  )
  unlisteners.push(
    await listen<{ ok: boolean; path?: string; message?: string }>('record-finished', event => {
      if (!event.payload.ok) {
        // 后端会在显示错误后清理会话，即使控制条尚未挂载。
        showError(t('screenshot.record.failed', { message: event.payload.message ?? '' }), true)
      }
    })
  )
  // 后端复用本窗口开启新一轮会话时重新初始化
  unlisteners.push(await listen('toolbar-reactivate', () => initFromBackend()))
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
  if (tickTimer) window.clearInterval(tickTimer)
  if (errorTimer) window.clearTimeout(errorTimer)
  for (const unlisten of unlisteners) unlisten()
})
</script>

<style scoped>
.ctrl-root {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  background: transparent;
  user-select: none;
  overflow: hidden;
  font-family: var(--jedi-font-ui, sans-serif);
}

.ctrl-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 46px;
  padding: 0 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 13px;
  background: rgba(15, 18, 24, 0.94);
  box-shadow: 0 20px 46px rgba(0, 0, 0, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.045);
  backdrop-filter: blur(22px) saturate(145%);
  white-space: nowrap;
}

/* 帧计数器：蓝环呼吸点 + 等宽数字 */
.frame-counter {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px 0 6px;
}

.fc-ring {
  display: grid;
  place-items: center;
  width: 17px;
  height: 17px;
  border-radius: 999px;
  background: rgba(53, 116, 240, 0.2);
  border: 1px solid rgba(53, 116, 240, 0.55);
}

.fc-ring.lost {
  background: rgba(211, 47, 47, 0.2);
  border-color: rgba(211, 47, 47, 0.6);
}

.fc-dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: #539bf5;
}

.fc-ring.lost .fc-dot {
  background: #ff5252;
}

.fc-label {
  font: 400 11px var(--jedi-font-mono, monospace);
  color: rgba(255, 255, 255, 0.55);
}

.fc-label b {
  color: #fff;
  font-weight: 700;
}

.fc-lost {
  color: #ff8a80;
  font-weight: 600;
}

.cb-divider {
  width: 1px;
  height: 20px;
  background: rgba(255, 255, 255, 0.16);
}

.cb-btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  height: 32px;
  padding: 0 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: rgba(255, 255, 255, 0.6);
  font-size: 12px;
  font-weight: 500;
  line-height: 1.5;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease, box-shadow 140ms ease;
}

.cb-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.cb-btn:focus-visible {
  outline: 2px solid #69b8ff;
  outline-offset: 1px;
}

.cb-btn:disabled {
  opacity: 0.55;
  cursor: default;
}

.cb-primary {
  background: #3574f0;
  color: #fff;
  font-weight: 600;
  box-shadow: 0 4px 12px rgba(53, 116, 240, 0.4);
}

.cb-primary:hover:not(:disabled) {
  background: #2f68e0;
  color: #fff;
}

.cb-tonal {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.cb-stop {
  background: linear-gradient(135deg, rgba(211, 47, 47, 0.95), rgba(183, 28, 28, 0.95));
  color: #fff;
  box-shadow: 0 5px 14px rgba(183, 28, 28, 0.32);
}

.cb-stop:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(229, 57, 53, 1), rgba(198, 40, 40, 1));
}

.rec-indicator {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 0 9px 0 8px;
  color: #ff8a80;
}

.rec-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #ff5252;
  box-shadow: 0 0 8px rgba(255, 82, 82, 0.8);
  animation: rec-pulse 1.2s ease-in-out infinite;
}

.rec-dot.paused {
  background: #ffc107;
  box-shadow: 0 0 8px rgba(255, 193, 7, 0.8);
  animation: none;
}

.rec-label {
  font: 700 11px/1 var(--jedi-font-mono, monospace);
  letter-spacing: 1.2px;
}

.rec-elapsed {
  min-width: 46px;
  color: rgba(255, 255, 255, 0.88);
  font: 600 12px/1 var(--jedi-font-mono, monospace);
  font-variant-numeric: tabular-nums;
}

.ctrl-error {
  position: absolute;
  top: 54px;
  left: 50%;
  transform: translateX(-50%);
  padding: 7px 13px;
  border: 1px solid rgba(255, 160, 160, 0.2);
  border-radius: 8px;
  background: rgba(127, 29, 29, 0.95);
  color: #fff;
  font-size: 12px;
  white-space: nowrap;
  max-width: 90vw;
  overflow: hidden;
  text-overflow: ellipsis;
}

@keyframes rec-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.35; }
}

@media (prefers-reduced-motion: reduce) {
  .rec-dot { animation: none; }
  .cb-btn { transition: none; }
}
</style>
