<template>
  <div class="ctrl-root" :class="`mode-${mode}`">
    <!-- 滚动截长图控制条 -->
    <div v-if="mode === 'scroll'" class="ctrl-bar" role="toolbar" :aria-label="t('screenshot.scrolling.title')">
      <div class="frame-counter" :title="t('screenshot.scrolling.frames', { count: frameCount })">
        <span class="fc-ring"><span class="fc-dot" aria-hidden="true"></span></span>
        <span class="fc-label"><b>{{ paddedCount }}</b>&nbsp;{{ t('screenshot.scrolling.unit') }}</span>
      </div>

      <span class="cb-divider" aria-hidden="true"></span>

      <button class="cb-btn cb-primary" type="button" :disabled="working" @click="captureFrame">
        {{ t('screenshot.scrolling.capture') }}
        <kbd>SPACE</kbd>
      </button>
      <button class="cb-btn cb-tonal" type="button" :disabled="working" @click="finish">
        {{ t('screenshot.scrolling.finish') }}
        <kbd>ENTER</kbd>
      </button>
      <button class="cb-btn cb-ghost" type="button" :aria-label="t('screenshot.scrolling.cancel')" :disabled="working" @click="cancel">
        {{ t('common.cancel') }}
        <kbd>ESC</kbd>
      </button>

      <span class="cb-divider" aria-hidden="true"></span>

      <div class="cb-progress" aria-hidden="true">
        <span v-for="i in 3" :key="i" class="seg" :class="{ filled: i <= Math.min(frameCount, 3) }"></span>
        <span class="seg-label">{{ Math.min(frameCount, 3) }}/3</span>
      </div>
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
import { mdiStopCircleOutline } from '@mdi/js'
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

const paddedCount = computed(() => frameCount.value.toString().padStart(2, '0'))

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
    mode.value = info.mode === 'scroll' || info.mode === 'record' ? info.mode : ''
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

.fc-dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: #539bf5;
}

.fc-label {
  font: 400 11px var(--jedi-font-mono, monospace);
  color: rgba(255, 255, 255, 0.55);
}

.fc-label b {
  color: #fff;
  font-weight: 700;
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

.cb-btn kbd {
  padding: 1px 5px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.07);
  color: rgba(255, 255, 255, 0.5);
  font: 700 9px/1.5 var(--jedi-font-mono, monospace);
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

.cb-primary kbd {
  background: rgba(255, 255, 255, 0.22);
  color: #fff;
}

.cb-primary:hover:not(:disabled) {
  background: #2f68e0;
  color: #fff;
}

.cb-tonal {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.cb-tonal kbd {
  background: rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.6);
}

.cb-stop {
  background: linear-gradient(135deg, rgba(211, 47, 47, 0.95), rgba(183, 28, 28, 0.95));
  color: #fff;
  box-shadow: 0 5px 14px rgba(183, 28, 28, 0.32);
}

.cb-stop:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(229, 57, 53, 1), rgba(198, 40, 40, 1));
}

/* 拼接进度段：3 段蓝色小条 */
.cb-progress {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 0 8px 0 2px;
}

.seg {
  width: 20px;
  height: 3px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.16);
}

.seg.filled {
  background: #3574f0;
}

.seg-label {
  margin-left: 3px;
  font: 400 9px var(--jedi-font-mono, monospace);
  color: rgba(255, 255, 255, 0.5);
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
