<template>
  <div class="ctrl-root" :class="`mode-${mode}`">
    <!-- 滚动截长图控制条 -->
    <div v-if="mode === 'scroll'" class="ctrl-bar" role="toolbar" :aria-label="t('screenshot.scrolling.title')">
      <span class="mode-chip">
        <v-icon :icon="mdiArrowCollapseDown" size="16" aria-hidden="true" />
        {{ t('screenshot.scrolling.title') }}
      </span>
      <button class="cb-btn cb-primary" type="button" :disabled="working" @click="captureFrame">
        <v-icon :icon="mdiCameraBurst" size="16" aria-hidden="true" />
        {{ t('screenshot.scrolling.capture') }}
        <kbd>Space</kbd>
      </button>
      <button class="cb-btn" type="button" :disabled="working" @click="finish">
        <v-icon :icon="mdiCheck" size="16" aria-hidden="true" />
        {{ t('screenshot.scrolling.finish') }}
        <kbd>Enter</kbd>
      </button>
      <button class="cb-btn cb-icon" type="button" :aria-label="t('screenshot.scrolling.cancel')" :disabled="working" @click="cancel">
        <v-icon :icon="mdiClose" size="18" aria-hidden="true" />
      </button>
      <span class="frame-count">{{ t('screenshot.scrolling.frames', { count: frameCount }) }}</span>
    </div>

    <!-- 录屏控制条 -->
    <div v-else-if="mode === 'record'" class="ctrl-bar" role="toolbar" :aria-label="t('screenshot.record.recording')">
      <span class="rec-indicator" :title="t('screenshot.record.maxDuration')">
        <span class="rec-dot" aria-hidden="true"></span>
        <span class="rec-label">REC</span>
        <span class="rec-elapsed">{{ elapsedLabel }}</span>
      </span>
      <button class="cb-btn cb-stop" type="button" :disabled="working" @click="stop">
        <v-icon :icon="mdiStopCircleOutline" size="17" aria-hidden="true" />
        {{ t('screenshot.record.stop') }}
      </button>
    </div>

    <div v-if="errorText" class="ctrl-error" role="alert">{{ errorText }}</div>
  </div>
</template>

<script setup lang="ts">
import {
  mdiArrowCollapseDown,
  mdiCameraBurst,
  mdiCheck,
  mdiClose,
  mdiStopCircleOutline,
} from '@mdi/js'
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
} from '@/api/screenshot'

const { t } = useI18n()
const window_ = getCurrentWindow()

const mode = ref<'scroll' | 'record' | ''>('')
const frameCount = ref(0)
const working = ref(false)
const errorText = ref('')
const elapsedMs = ref(0)
let maxDurationMs = 0
let recordStartAt = 0
let tickTimer: number | null = null
let errorTimer: number | null = null
let stopping = false
const unlisteners: UnlistenFn[] = []

function showError(message: string) {
  errorText.value = message
  if (errorTimer) window.clearTimeout(errorTimer)
  errorTimer = window.setTimeout(() => {
    errorText.value = ''
  }, 3000)
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

// 同一按键会走两条通路：窗口内 keydown + 后端注册的全局快捷键事件。
// 按动作类型做时间闩锁去重，避免一次 Space 捕两帧（不同动作之间互不影响）。
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
    mode.value = info.mode
    frameCount.value = info.frameCount
    maxDurationMs = info.maxDurationMs
    stopping = false
    working.value = false
    if (tickTimer) {
      window.clearInterval(tickTimer)
      tickTimer = null
    }
    if (info.mode === 'record') {
      recordStartAt = Date.now()
      elapsedMs.value = 0
      tickTimer = window.setInterval(() => {
        elapsedMs.value = Date.now() - recordStartAt
        if (maxDurationMs > 0 && elapsedMs.value >= maxDurationMs) {
          // 后端 10 分钟自动停止；此处兜底触发一次 stop
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
    await listen<string>('scroll-hotkey', event => {
      const action = event.payload
      if (action === 'capture') captureFrame()
      else if (action === 'finish') finish()
      else if (action === 'cancel') cancel()
    })
  )
  unlisteners.push(
    await listen<{ frameCount: number }>('scroll-frame', event => {
      frameCount.value = event.payload.frameCount
    })
  )
  unlisteners.push(
    await listen<{ ok: boolean; path?: string; message?: string }>('record-finished', event => {
      if (!event.payload.ok) {
        showError(t('screenshot.record.failed', { message: event.payload.message ?? '' }))
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
  gap: 4px;
  min-height: 44px;
  padding: 5px 7px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 13px;
  background: rgba(15, 18, 24, 0.94);
  box-shadow: 0 18px 46px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.045);
  backdrop-filter: blur(22px) saturate(145%);
  white-space: nowrap;
}

.mode-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 8px 0 6px;
  color: #9bd8ff;
  font-size: 11px;
  font-weight: 650;
}

.cb-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 32px;
  padding: 0 11px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: rgba(255, 255, 255, 0.8);
  font-size: 12px;
  font-weight: 550;
  line-height: 1.5;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
}

.cb-btn kbd {
  padding: 1px 5px;
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.07);
  color: rgba(255, 255, 255, 0.62);
  font: 10px/1.4 var(--jedi-font-mono, monospace);
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

.cb-icon {
  justify-content: center;
  width: 34px;
  padding-inline: 6px;
}

.cb-primary {
  background: linear-gradient(135deg, rgba(42, 133, 224, 0.94), rgba(57, 104, 221, 0.94));
  color: #fff;
  box-shadow: 0 5px 14px rgba(36, 112, 210, 0.28);
}

.cb-primary:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(53, 148, 240, 1), rgba(68, 117, 231, 1));
}

.cb-stop {
  background: linear-gradient(135deg, rgba(211, 47, 47, 0.95), rgba(183, 28, 28, 0.95));
  color: #fff;
  box-shadow: 0 5px 14px rgba(183, 28, 28, 0.32);
}

.cb-stop:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(229, 57, 53, 1), rgba(198, 40, 40, 1));
}

.frame-count {
  padding: 0 6px;
  color: rgba(255, 255, 255, 0.55);
  font: 600 11px/1.4 var(--jedi-font-mono, monospace);
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
