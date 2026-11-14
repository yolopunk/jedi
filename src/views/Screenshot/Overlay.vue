<template>
  <div
    class="overlay-root"
    :class="{
      'is-selecting': drag && drag.mode === 'new',
      'has-selection': !!selection && mode === 'select',
      'is-recording': mode === 'record',
      'is-scrolling': mode === 'scroll',
    }"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @dblclick="confirmCopy"
  >
    <!-- 冻结帧：窗口物理尺寸 = 显示器物理尺寸，CSS 视口按 scaleFactor 缩放后 1:1 铺满 -->
    <img v-if="frameSrc && mode !== 'record' && mode !== 'scroll'" class="frozen-frame" :src="frameSrc" @load="revealOverlay" @error="cancel" draggable="false" alt="" />

    <!-- 未开始框选时的十字引导线 -->
    <template v-if="!selection && !drag">
      <div v-if="cursor" class="guide guide-v" :style="{ left: `${cursor.x}px` }"></div>
      <div v-if="cursor" class="guide guide-h" :style="{ top: `${cursor.y}px` }"></div>
    </template>

    <!-- 选区：box-shadow 大扩散实现外部压暗 -->
    <div
      v-if="selection && mode === 'select'"
      class="selection"
      :style="selectionStyle"
      @mousedown.stop="onSelectionMouseDown"
      @dblclick.stop="confirmCopy"
    >
      <div
        v-for="handle in HANDLES"
        :key="handle.id"
        class="handle"
        :class="[`handle-${handle.id}`]"
        :style="handleStyle(handle)"
        @mousedown.stop="onHandleMouseDown(handle.id, $event)"
      ></div>
      <div class="size-badge" :class="{ flipped: badgeFlipped }">{{ sizeLabel }}</div>
    </div>

    <!-- 录制模式：保留选区边框 + REC 徽标与已录时长（窗口已点击穿透且排除出捕获） -->
    <div v-if="mode === 'record' && selection" class="record-frame" :style="selectionStyle">
      <div class="rec-badge" :class="{ flipped: badgeFlipped }">
        <span class="rec-dot"></span>REC {{ recordElapsedLabel }}
      </div>
    </div>

    <!-- 滚动截长图：保留选区边框（画面实时可见、可自由滚动），窗口已点击穿透并排除出捕获 -->
    <div v-if="mode === 'scroll' && selection" class="scroll-frame" :style="selectionStyle">
      <span class="scroll-badge" :class="{ flipped: badgeFlipped }">
        <span class="scroll-dot" aria-hidden="true"></span>
        {{ t('screenshot.scrolling.frameHint') }}
      </span>
      <span class="scroll-corner tl" aria-hidden="true"></span>
      <span class="scroll-corner tr" aria-hidden="true"></span>
      <span class="scroll-corner bl" aria-hidden="true"></span>
      <span class="scroll-corner br" aria-hidden="true"></span>
    </div>

    <!-- 操作条：阻止 mousedown 冒泡到根节点，否则按下按钮会先清空选区 -->
    <div
      v-if="selection && !busy && mode === 'select'"
      ref="toolbarRef"
      class="toolbar"
      role="toolbar"
      :aria-label="t('screenshot.title')"
      :style="toolbarStyle"
      @mousedown.stop
    >
      <button class="tb-btn" type="button" @click.stop="finish('copy')">
        <v-icon :icon="mdiContentCopy" size="16" aria-hidden="true" />
        {{ t('screenshot.copy') }}
      </button>
      <button class="tb-btn" type="button" @click.stop="finish('save')">
        <v-icon :icon="mdiContentSaveOutline" size="16" aria-hidden="true" />
        {{ t('screenshot.save') }}
      </button>
      <button
        class="tb-btn tb-ocr"
        type="button"
        :title="t('screenshot.ocr.shortcut')"
        @click.stop="finish('ocr')"
      >
        <v-icon :icon="mdiTextRecognition" size="17" aria-hidden="true" />
        {{ t('screenshot.ocr.title') }}
        <kbd>⇧C</kbd>
      </button>
      <button class="tb-btn" type="button" @click.stop="startScrolling">
        <v-icon :icon="mdiArrowCollapseDown" size="17" aria-hidden="true" />
        {{ t('screenshot.scrolling.action') }}
      </button>
      <button class="tb-btn" type="button" @click.stop="startRecording">
        <v-icon :icon="mdiVideoOutline" size="17" aria-hidden="true" />
        {{ t('screenshot.record.action') }}
      </button>
      <button class="tb-btn tb-primary" type="button" @click.stop="finish('chat')">
        <v-icon :icon="mdiRobotOutline" size="16" aria-hidden="true" />
        {{ t('screenshot.sendToChat') }}
      </button>
      <span class="tb-divider"></span>
      <button class="tb-btn tb-icon" type="button" :aria-label="t('screenshot.cancel')" @click.stop="cancel">
        <v-icon :icon="mdiClose" size="18" aria-hidden="true" />
      </button>
    </div>

    <!-- 底部提示（无选区时） -->
    <div v-if="!selection && !drag" class="hint">{{ t('screenshot.overlayHint') }}</div>

    <div v-if="errorText" class="error-toast" role="alert">{{ errorText }}</div>
  </div>
</template>

<script setup lang="ts">
import {
  mdiArrowCollapseDown,
  mdiClose,
  mdiContentCopy,
  mdiContentSaveOutline,
  mdiRobotOutline,
  mdiTextRecognition,
  mdiVideoOutline,
} from '@mdi/js'
import { convertFileSrc } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  screenshotCancel,
  screenshotFinish,
  screenshotFrameInfo,
  screenshotScrollBegin,
  startRegionRecord,
} from '@/api/screenshot'

const { t } = useI18n()

interface Rect {
  x: number
  y: number
  w: number
  h: number
}

interface DragState {
  mode: 'new' | 'move' | 'resize'
  anchorX: number
  anchorY: number
  origin: Rect
  handle: string | null
}

const HANDLES = [
  { id: 'nw', x: 0, y: 0 },
  { id: 'n', x: 0.5, y: 0 },
  { id: 'ne', x: 1, y: 0 },
  { id: 'e', x: 1, y: 0.5 },
  { id: 'se', x: 1, y: 1 },
  { id: 's', x: 0.5, y: 1 },
  { id: 'sw', x: 0, y: 1 },
  { id: 'w', x: 0, y: 0.5 },
] as const

const TOOLBAR_HEIGHT = 40
const TOOLBAR_MARGIN = 8
const toolbarRef = ref<HTMLElement | null>(null)
const toolbarWidth = ref(360)
let toolbarObserver: ResizeObserver | null = null
const MIN_SELECTION = 3

const window_ = getCurrentWindow()
// 显示器编号编码在窗口 label 后缀（screenshot-overlay-<n>）
const monitorIndex = Number.parseInt(window_.label.split('-').pop() ?? '0', 10) || 0

const frameSrc = ref('')
const scaleFactor = ref(1)
const selection = ref<Rect | null>(null)
const drag = ref<DragState | null>(null)
const cursor = ref<{ x: number; y: number } | null>(null)
const busy = ref(false)
const errorText = ref('')
let errorTimer: number | null = null
// 录制模式：覆盖窗切换为透明 + 点击穿透（由后端设置），仅渲染边框与 REC 徽标
const mode = ref<'select' | 'record' | 'scroll'>('select')
const recordElapsedMs = ref(0)
let recordStartAt = 0
let recordTimer: number | null = null

const recordElapsedLabel = computed(() => {
  const total = Math.floor(recordElapsedMs.value / 1000)
  const m = Math.floor(total / 60)
  const s = total % 60
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
})

const selectionStyle = computed(() =>
  selection.value
    ? {
        left: `${selection.value.x}px`,
        top: `${selection.value.y}px`,
        width: `${selection.value.w}px`,
        height: `${selection.value.h}px`,
      }
    : undefined
)

const sizeLabel = computed(() => {
  if (!selection.value || !scaleFactor.value) return ''
  const { w, h } = selection.value
  return `${Math.round(w * scaleFactor.value)} × ${Math.round(h * scaleFactor.value)}`
})

// 尺寸角标默认悬在选区上方，贴近屏幕顶时翻到选区内
const badgeFlipped = computed(() => (selection.value?.y ?? 0) < 28)

const toolbarStyle = computed(() => {
  if (!selection.value) return undefined
  const { x, y, w, h } = selection.value
  const vw = window.innerWidth
  const vh = window.innerHeight
  const maxLeft = Math.max(8, vw - toolbarWidth.value - 8)
  const left = Math.min(Math.max(x + w / 2 - toolbarWidth.value / 2, 8), maxLeft)
  const below = y + h + TOOLBAR_MARGIN + TOOLBAR_HEIGHT
  const top =
    below <= vh ? y + h + TOOLBAR_MARGIN : Math.max(y - TOOLBAR_MARGIN - TOOLBAR_HEIGHT, 8)
  return { left: `${left}px`, top: `${top}px` }
})

function showError(message: string) {
  errorText.value = message
  if (errorTimer) window.clearTimeout(errorTimer)
  errorTimer = window.setTimeout(() => {
    errorText.value = ''
  }, 3000)
}

function clampRect(rect: Rect): Rect {
  const vw = window.innerWidth
  const vh = window.innerHeight
  const w = Math.min(Math.max(rect.w, 0), vw)
  const h = Math.min(Math.max(rect.h, 0), vh)
  const x = Math.min(Math.max(rect.x, 0), vw - w)
  const y = Math.min(Math.max(rect.y, 0), vh - h)
  return { x, y, w, h }
}

function onMouseDown(e: MouseEvent) {
  if (e.button !== 0 || busy.value) return
  selection.value = null
  drag.value = {
    mode: 'new',
    anchorX: e.clientX,
    anchorY: e.clientY,
    origin: { x: e.clientX, y: e.clientY, w: 0, h: 0 },
    handle: null,
  }
}

function onSelectionMouseDown(e: MouseEvent) {
  if (e.button !== 0 || busy.value || !selection.value) return
  const rect = selection.value
  drag.value = {
    mode: 'move',
    anchorX: e.clientX,
    anchorY: e.clientY,
    origin: { ...rect },
    handle: null,
  }
}

function onHandleMouseDown(handleId: string, e: MouseEvent) {
  if (e.button !== 0 || busy.value || !selection.value) return
  drag.value = {
    mode: 'resize',
    anchorX: e.clientX,
    anchorY: e.clientY,
    origin: { ...selection.value },
    handle: handleId,
  }
}

function onMouseMove(e: MouseEvent) {
  cursor.value = { x: e.clientX, y: e.clientY }
  const state = drag.value
  if (!state) return
  const dx = e.clientX - state.anchorX
  const dy = e.clientY - state.anchorY

  if (state.mode === 'new') {
    selection.value = clampRect({
      x: Math.min(state.anchorX, e.clientX),
      y: Math.min(state.anchorY, e.clientY),
      w: Math.abs(dx),
      h: Math.abs(dy),
    })
    return
  }
  if (!selection.value) return

  if (state.mode === 'move') {
    selection.value = clampRect({
      x: state.origin.x + dx,
      y: state.origin.y + dy,
      w: state.origin.w,
      h: state.origin.h,
    })
    return
  }

  // resize：按手柄方向单侧/双侧跟随，允许跨过对边（等价于换边拖拽）
  const handle = state.handle ?? ''
  let { x, y, w, h } = state.origin
  if (handle.includes('w')) {
    x = state.origin.x + dx
    w = state.origin.w - dx
  }
  if (handle.includes('e')) {
    w = state.origin.w + dx
  }
  if (handle.includes('n')) {
    y = state.origin.y + dy
    h = state.origin.h - dy
  }
  if (handle.includes('s')) {
    h = state.origin.h + dy
  }
  selection.value = clampRect({
    x: Math.min(x, x + w),
    y: Math.min(y, y + h),
    w: Math.abs(w),
    h: Math.abs(h),
  })
}

function onMouseUp() {
  const state = drag.value
  drag.value = null
  if (!state || state.mode !== 'new') return
  const rect = selection.value
  if (!rect || rect.w < MIN_SELECTION || rect.h < MIN_SELECTION) {
    selection.value = null
  }
}

function physicalRect() {
  const { x, y, w, h } = selection.value!
  const scale = scaleFactor.value
  return {
    x: Math.round(x * scale),
    y: Math.round(y * scale),
    width: Math.round(w * scale),
    height: Math.round(h * scale),
  }
}

async function finish(action: 'copy' | 'save' | 'chat' | 'ocr') {
  if (!selection.value || busy.value) return
  busy.value = true
  try {
    await screenshotFinish(monitorIndex, physicalRect(), action)
    // 成功后 Rust 会关闭所有覆盖窗口；窗口若仍在（异常路径）则兜底取消
    window.setTimeout(() => {
      busy.value = false
    }, 2000)
  } catch (err) {
    busy.value = false
    showError(t('screenshot.captureFailed', { message: String(err) }))
  }
}

/**
 * 进入滚动截长图：后端把本覆窗切成「点击穿透 + 排除捕获」，只保留选区边框；
 * 画面实时可见，用户可以直接在框内滚动页面。
 */
async function startScrolling() {
  if (!selection.value || busy.value) return
  busy.value = true
  try {
    await screenshotScrollBegin(monitorIndex, physicalRect())
    mode.value = 'scroll'
    // 后端已接管窗口；失败兜底恢复交互
    window.setTimeout(() => {
      busy.value = false
    }, 2000)
  } catch (err) {
    mode.value = 'select'
    busy.value = false
    showError(t('screenshot.scrolling.failed', { message: String(err) }))
  }
}

/** 进入选区录屏：本窗口切为透明穿透（后端设置），仅渲染边框与 REC 徽标 */
async function startRecording() {
  if (!selection.value || busy.value) return
  busy.value = true
  const rect = physicalRect()
  mode.value = 'record'
  recordStartAt = Date.now()
  recordElapsedMs.value = 0
  if (recordTimer) window.clearInterval(recordTimer)
  recordTimer = window.setInterval(() => {
    recordElapsedMs.value = Date.now() - recordStartAt
  }, 250)
  try {
    await startRegionRecord(monitorIndex, rect)
    busy.value = false
  } catch (err) {
    stopRecordUi()
    showError(t('screenshot.record.failed', { message: String(err) }))
  }
}

function stopRecordUi() {
  if (recordTimer) {
    window.clearInterval(recordTimer)
    recordTimer = null
  }
  mode.value = 'select'
  busy.value = false
}

function confirmCopy() {
  finish('copy')
}

async function cancel() {
  if (busy.value) return
  busy.value = true
  try {
    await screenshotCancel()
  } catch (err) {
    busy.value = false
    showError(t('screenshot.captureFailed', { message: String(err) }))
  }
}

function onKeyDown(e: KeyboardEvent) {
  // 录制模式由悬浮控制条接管键盘，覆盖窗此时也不持有焦点
  if (mode.value === 'record') {
    if (e.key === 'Escape' || e.key === 'Enter' || e.code === 'Space') e.preventDefault()
    return
  }
  if (e.key === 'Escape') {
    e.preventDefault()
    cancel()
    return
  }
  if (e.key === 'Enter') {
    e.preventDefault()
    if (selection.value) confirmCopy()
    return
  }
  if (selection.value && e.shiftKey && e.key.toLowerCase() === 'c') {
    e.preventDefault()
    finish('ocr')
    return
  }
  if (!selection.value) return
  const step = e.shiftKey ? 10 : 1
  let dx = 0
  let dy = 0
  if (e.key === 'ArrowLeft') dx = -step
  else if (e.key === 'ArrowRight') dx = step
  else if (e.key === 'ArrowUp') dy = -step
  else if (e.key === 'ArrowDown') dy = step
  else return
  e.preventDefault()
  const rect = selection.value
  selection.value = clampRect({ ...rect, x: rect.x + dx, y: rect.y + dy })
}

function handleStyle(handle: (typeof HANDLES)[number]) {
  return {
    left: `calc(${handle.x * 100}% - ${handle.x === 0.5 ? 4 : 5}px)`,
    top: `calc(${handle.y * 100}% - ${handle.y === 0.5 ? 4 : 5}px)`,
  }
}

async function revealOverlay(event: Event) {
  try {
    await (event.target as HTMLImageElement).decode()
    await nextTick()
    await window_.show()
    if (monitorIndex === 0) await window_.setFocus()
  } catch (err) {
    console.error(err)
    await cancel()
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown)
  try {
    const [info, scale] = await Promise.all([
      screenshotFrameInfo(monitorIndex),
      window_.scaleFactor(),
    ])
    scaleFactor.value = scale
    frameSrc.value = convertFileSrc(info.framePath)
  } catch (err) {
    console.error(err)
    await cancel()
  }
})

watch(toolbarRef, element => {
  toolbarObserver?.disconnect()
  toolbarObserver = null
  if (!element) return
  toolbarObserver = new ResizeObserver(entries => {
    const width = entries[0]?.borderBoxSize?.[0]?.inlineSize ?? element.offsetWidth
    if (width > 0) toolbarWidth.value = width
  })
  toolbarObserver.observe(element)
})

onUnmounted(() => {
  toolbarObserver?.disconnect()
  window.removeEventListener('keydown', onKeyDown)
  if (errorTimer) window.clearTimeout(errorTimer)
  if (recordTimer) window.clearInterval(recordTimer)
})
</script>

<style scoped>
.overlay-root {
  position: fixed;
  inset: 0;
  overflow: hidden;
  user-select: none;
  cursor: crosshair;
  background: #05070a;
}

/* 录制模式：窗口透明化（后端同时开启点击穿透与捕获排除），只留边框与徽标 */
.overlay-root.is-recording {
  cursor: default;
  background: transparent;
}

/* 滚动截长图：画面实时可见，只画一个「取景框」 */
.overlay-root.is-scrolling {
  cursor: default;
  background: transparent;
}

.scroll-frame {
  position: absolute;
  border: 1px dashed rgb(var(--accent-rgb) / 0.9);
  border-radius: 2px;
  pointer-events: none;
  z-index: 2;
  box-shadow:
    0 0 0 1px rgb(0 0 0 / 0.28),
    0 0 22px rgb(var(--accent-rgb) / 0.18);
}

.scroll-corner {
  position: absolute;
  width: 16px;
  height: 16px;
  border: 2px solid rgb(var(--accent-rgb));
  filter: drop-shadow(0 0 6px rgb(var(--accent-rgb) / 0.55));
}

.scroll-corner.tl { top: -2px; left: -2px; border-right: 0; border-bottom: 0; border-radius: 3px 0 0 0; }
.scroll-corner.tr { top: -2px; right: -2px; border-left: 0; border-bottom: 0; border-radius: 0 3px 0 0; }
.scroll-corner.bl { bottom: -2px; left: -2px; border-right: 0; border-top: 0; border-radius: 0 0 0 3px; }
.scroll-corner.br { bottom: -2px; right: -2px; border-left: 0; border-top: 0; border-radius: 0 0 3px 0; }

.scroll-badge {
  position: absolute;
  top: -30px;
  left: 0;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 9px;
  border-radius: 6px;
  background: rgb(var(--accent-rgb));
  color: #fff;
  font-family: var(--jedi-font-mono);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.3px;
  white-space: nowrap;
  box-shadow: 0 6px 16px rgb(var(--accent-rgb) / 0.45);
}

.scroll-badge.flipped {
  top: auto;
  bottom: -30px;
}

.scroll-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #fff;
  animation: scroll-dot-pulse 1.3s ease-in-out infinite;
}

@keyframes scroll-dot-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.3;
  }
}

@media (prefers-reduced-motion: reduce) {
  .scroll-dot {
    animation: none;
  }
}

.record-frame {
  position: absolute;
  border: 2px solid rgba(255, 82, 82, 0.95);
  outline: 1px solid rgba(255, 255, 255, 0.35);
  outline-offset: 1px;
  pointer-events: none;
  z-index: 2;
}

.rec-badge {
  position: absolute;
  top: -34px;
  right: 0;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 8px;
  background: rgba(15, 18, 24, 0.92);
  color: #ff8a80;
  font: 700 12px/1.4 var(--jedi-font-mono, monospace);
  font-variant-numeric: tabular-nums;
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(12px);
  white-space: nowrap;
}

.rec-badge.flipped {
  top: 7px;
  right: 7px;
}

.rec-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #ff5252;
  box-shadow: 0 0 8px rgba(255, 82, 82, 0.8);
  animation: rec-pulse 1.2s ease-in-out infinite;
}

@keyframes rec-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.35; }
}

.frozen-frame {
  position: absolute;
  inset: 0;
  width: 100vw;
  height: 100vh;
  object-fit: fill;
  pointer-events: none;
}

.guide {
  position: absolute;
  background: rgba(91, 174, 255, 0.48);
  box-shadow: 0 0 8px rgba(62, 158, 255, 0.22);
  pointer-events: none;
  z-index: 1;
}

.guide-v {
  top: 0;
  bottom: 0;
  width: 1px;
}

.guide-h {
  left: 0;
  right: 0;
  height: 1px;
}

.selection {
  position: absolute;
  border: 1px solid rgba(115, 192, 255, 1);
  outline: 1px solid rgba(255, 255, 255, 0.38);
  outline-offset: -2px;
  /* 大扩散阴影把选区之外整体压暗 */
  box-shadow: 0 0 0 100000px rgba(3, 6, 12, 0.56), inset 0 0 0 1px rgba(6, 15, 24, 0.28);
  cursor: move;
  z-index: 2;
}

.handle {
  position: absolute;
  width: 10px;
  height: 10px;
  background: #f8fcff;
  border: 2px solid #318ae7;
  border-radius: 50%;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.32);
}

.handle-nw,
.handle-se {
  cursor: nwse-resize;
}

.handle-ne,
.handle-sw {
  cursor: nesw-resize;
}

.handle-n,
.handle-s {
  cursor: ns-resize;
}

.handle-e,
.handle-w {
  cursor: ew-resize;
}

.size-badge {
  position: absolute;
  top: -31px;
  right: 0;
  padding: 4px 9px;
  border: 1px solid rgba(255, 255, 255, 0.13);
  border-radius: 7px;
  background: rgba(12, 16, 22, 0.88);
  color: rgba(255, 255, 255, 0.9);
  font: 600 11px/1.4 var(--jedi-font-mono, monospace);
  box-shadow: 0 5px 16px rgba(0, 0, 0, 0.28);
  backdrop-filter: blur(12px);
  white-space: nowrap;
  pointer-events: none;
}

.size-badge.flipped {
  top: 7px;
  right: 7px;
}

.toolbar {
  position: absolute;
  display: flex;
  align-items: center;
  gap: 3px;
  min-height: 46px;
  padding: 5px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 13px;
  background: rgba(15, 18, 24, 0.91);
  box-shadow: 0 18px 46px rgba(0, 0, 0, 0.48), inset 0 1px 0 rgba(255, 255, 255, 0.045);
  backdrop-filter: blur(22px) saturate(145%);
  z-index: 3;
  max-width: calc(100vw - 16px);
  overflow-x: auto;
  white-space: nowrap;
  animation: toolbar-enter 180ms cubic-bezier(0.2, 0.8, 0.2, 1);
}

.tb-btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  min-height: 34px;
  padding: 0 11px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: rgba(255, 255, 255, 0.78);
  font-size: 12px;
  font-weight: 550;
  line-height: 1.5;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease, box-shadow 140ms ease;
}

.tb-icon {
  justify-content: center;
  width: 34px;
  padding-inline: 6px;
}

.tb-ocr {
  color: #9bd8ff;
}

.tb-ocr kbd {
  padding: 1px 5px;
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.07);
  color: rgba(255, 255, 255, 0.62);
  font: 10px/1.4 var(--jedi-font-mono, monospace);
}

.tb-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.tb-btn:focus-visible {
  outline: 2px solid #69b8ff;
  outline-offset: 1px;
}

.tb-primary {
  background: linear-gradient(135deg, rgba(42, 133, 224, 0.94), rgba(57, 104, 221, 0.94));
  color: #fff;
  box-shadow: 0 5px 14px rgba(36, 112, 210, 0.28);
}

.tb-primary:hover {
  background: linear-gradient(135deg, rgba(53, 148, 240, 1), rgba(68, 117, 231, 1));
}

.tb-divider {
  width: 1px;
  height: 22px;
  margin: 0 4px;
  background: rgba(255, 255, 255, 0.12);
}

.hint {
  position: absolute;
  bottom: 28px;
  left: 50%;
  transform: translateX(-50%);
  padding: 8px 15px;
  border: 1px solid rgba(255, 255, 255, 0.11);
  border-radius: 999px;
  background: rgba(13, 16, 21, 0.78);
  color: rgba(255, 255, 255, 0.78);
  font: 11px var(--jedi-font-ui, sans-serif);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.28);
  backdrop-filter: blur(16px);
  white-space: nowrap;
  pointer-events: none;
  z-index: 1;
}

.error-toast {
  position: absolute;
  top: 22px;
  left: 50%;
  transform: translateX(-50%);
  padding: 9px 15px;
  border: 1px solid rgba(255, 160, 160, 0.2);
  border-radius: 9px;
  background: rgba(127, 29, 29, 0.93);
  color: #fff;
  font-size: 13px;
  z-index: 4;
}

@keyframes toolbar-enter {
  from { opacity: 0; transform: translateY(-4px) scale(0.98); }
  to { opacity: 1; transform: none; }
}

@media (prefers-reduced-motion: reduce) {
  .toolbar { animation: none; }
  .tb-btn { transition: none; }
}
</style>
