<template>
  <div
    class="overlay-root"
    :class="{ 'is-selecting': drag && drag.mode === 'new', 'has-selection': !!selection }"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @dblclick="confirmCopy"
  >
    <!-- 冻结帧：窗口物理尺寸 = 显示器物理尺寸，CSS 视口按 scaleFactor 缩放后 1:1 铺满 -->
    <img v-if="frameSrc" class="frozen-frame" :src="frameSrc" @load="revealOverlay" @error="cancel" draggable="false" alt="" />

    <!-- 未开始框选时的十字引导线 -->
    <template v-if="!selection && !drag">
      <div v-if="cursor" class="guide guide-v" :style="{ left: `${cursor.x}px` }"></div>
      <div v-if="cursor" class="guide guide-h" :style="{ top: `${cursor.y}px` }"></div>
    </template>

    <!-- 选区：box-shadow 大扩散实现外部压暗 -->
    <div
      v-if="selection"
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

    <!-- 操作条：阻止 mousedown 冒泡到根节点，否则按下按钮会先清空选区 -->
    <div
      v-if="selection && !busy"
      ref="toolbarRef"
      class="toolbar"
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
  mdiClose,
  mdiContentCopy,
  mdiContentSaveOutline,
  mdiRobotOutline,
  mdiTextRecognition,
} from '@mdi/js'
import { convertFileSrc } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { screenshotCancel, screenshotFinish, screenshotFrameInfo } from '@/api/screenshot'

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

async function finish(action: 'copy' | 'save' | 'chat' | 'ocr') {
  if (!selection.value || busy.value) return
  busy.value = true
  const { x, y, w, h } = selection.value
  const scale = scaleFactor.value
  try {
    await screenshotFinish(
      monitorIndex,
      {
        x: Math.round(x * scale),
        y: Math.round(y * scale),
        width: Math.round(w * scale),
        height: Math.round(h * scale),
      },
      action
    )
    // 成功后 Rust 会关闭所有覆盖窗口；窗口若仍在（异常路径）则兜底取消
    window.setTimeout(() => {
      busy.value = false
    }, 2000)
  } catch (err) {
    busy.value = false
    showError(t('screenshot.captureFailed', { message: String(err) }))
  }
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
})
</script>

<style scoped>
.overlay-root {
  position: fixed;
  inset: 0;
  overflow: hidden;
  user-select: none;
  cursor: crosshair;
  background: #000;
}

.frozen-frame {
  position: absolute;
  inset: 0;
  width: 100vw;
  height: 100vh;
  pointer-events: none;
}

.guide {
  position: absolute;
  background: rgba(80, 200, 255, 0.55);
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
  border: 1px solid rgba(80, 200, 255, 0.95);
  /* 大扩散阴影把选区之外整体压暗 */
  box-shadow: 0 0 0 100000px rgba(0, 0, 0, 0.5);
  cursor: move;
  z-index: 2;
}

.handle {
  position: absolute;
  width: 9px;
  height: 9px;
  background: #fff;
  border: 1px solid rgba(30, 30, 30, 0.8);
  border-radius: 1px;
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
  top: -26px;
  right: 0;
  padding: 2px 8px;
  border-radius: 3px;
  background: rgba(20, 20, 20, 0.85);
  color: #fff;
  font-size: 12px;
  line-height: 1.4;
  white-space: nowrap;
  pointer-events: none;
}

.size-badge.flipped {
  top: 4px;
  right: 4px;
}

.toolbar {
  position: absolute;
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px;
  border-radius: 6px;
  background: rgba(28, 28, 30, 0.96);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  z-index: 3;
  max-width: calc(100vw - 16px);
  overflow-x: auto;
  white-space: nowrap;
}

.tb-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(255, 255, 255, 0.85);
  font-size: 13px;
  line-height: 1.5;
  cursor: pointer;
}

.tb-icon {
  justify-content: center;
  width: 32px;
  padding-inline: 6px;
}

.tb-ocr {
  color: #9ee4ff;
}

.tb-ocr kbd {
  padding: 1px 5px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.07);
  color: rgba(255, 255, 255, 0.62);
  font: 10px/1.4 var(--jedi-font-mono, monospace);
}

.tb-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
}

.tb-primary {
  background: rgba(80, 200, 255, 0.18);
  color: #7fd8ff;
}

.tb-primary:hover {
  background: rgba(80, 200, 255, 0.3);
}

.tb-divider {
  width: 1px;
  height: 18px;
  margin: 0 4px;
  background: rgba(255, 255, 255, 0.15);
}

.hint {
  position: absolute;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 6px 16px;
  border-radius: 999px;
  background: rgba(20, 20, 20, 0.7);
  color: rgba(255, 255, 255, 0.85);
  font-size: 12px;
  white-space: nowrap;
  pointer-events: none;
  z-index: 1;
}

.error-toast {
  position: absolute;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  padding: 6px 16px;
  border-radius: 6px;
  background: rgba(140, 30, 30, 0.92);
  color: #fff;
  font-size: 13px;
  z-index: 4;
}
</style>
