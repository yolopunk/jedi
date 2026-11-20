<template>
  <div
    ref="rootRef"
    class="overlay-root"
    tabindex="-1"
    :class="{
      'is-recording': mode === 'record',
      'is-scrolling': mode === 'scroll',
    }"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="cancelDrag"
    @lostpointercapture="cancelDrag"
    @pointerleave="leaveSurface"
    @dblclick="confirmDoubleClick"
    @contextmenu.prevent="onContextMenu"
  >
    <img
      v-if="frameSrc && mode === 'select'"
      ref="frameEl"
      class="frozen-frame"
      crossorigin="anonymous"
      :src="frameSrc"
      @load="revealOverlay"
      @error="cancel"
      draggable="false"
      alt=""
    />

    <div v-if="windowTargetStyle" class="window-target" :style="windowTargetStyle" aria-hidden="true"></div>
    <div v-else-if="mode === 'select' && !selection" class="capture-scrim" aria-hidden="true"></div>

    <!-- 选区：白描边 + 蓝色外圈，选区外压暗 -->
    <div
      v-if="selection && mode === 'select'"
      class="selection"
      :style="selectionStyle"
      @pointerdown.stop="startDrag($event, 'move')"
    >
      <CaptureAnnotations
        v-if="drag?.mode !== 'new'"
        ref="editor"
        :crop="selection"
        :frame="frameEl"
        :tool="annotationTool"
        :color="annotationColor"
        :width="annotationWidth"
        :text-size="annotationTextSize"
        :disabled="busy || !!drag"
        @activate="focusOverlay"
      />
      <div
        v-for="handle in HANDLES"
        :key="handle.id"
        class="handle"
        :class="[`handle-${handle.id}`]"
        :style="handleStyle(handle)"
        @pointerdown.stop="startDrag($event, 'resize', handle.id)"
      ></div>
      <div class="size-badge" :class="{ flipped: badgeFlipped }">
        {{ sizeLabel }}
      </div>
    </div>

    <div v-if="loupeStyle" class="loupe" :style="loupeStyle" aria-hidden="true">
      <div class="loupe-grid">
        <canvas ref="loupeCanvas" class="loupe-canvas" :width="LOUPE_PIXELS" :height="LOUPE_PIXELS"></canvas>
      </div>
      <div class="loupe-meta"><span>{{ loupeCoords }}</span><span>{{ loupeHex }}</span></div>
    </div>

    <!-- 录制模式：保留选区边框 + REC 徽标与已录时长（窗口已点击穿透且排除出捕获） -->
    <div v-if="mode === 'record' && selection" class="record-frame" :style="selectionStyle">
      <div class="rec-badge" :class="{ flipped: badgeFlipped }">
        <span class="rec-dot"></span>REC {{ recordElapsedLabel }}
      </div>
    </div>

    <!-- 滚动截长图：保留取景框（画面实时可见、随滚自动捕获），窗口已点击穿透并排除出捕获 -->
    <div v-if="mode === 'scroll' && selection" class="scroll-frame" :style="selectionStyle">
      <span class="scroll-badge" :class="{ flipped: badgeFlipped }">
        <span class="scroll-dot" aria-hidden="true"></span>
        {{ t('screenshot.scrolling.frameHint', { height: scrollStitchedHeight }) }}
      </span>
      <span class="scroll-corner tl" aria-hidden="true"></span>
      <span class="scroll-corner tr" aria-hidden="true"></span>
      <span class="scroll-corner bl" aria-hidden="true"></span>
      <span class="scroll-corner br" aria-hidden="true"></span>
      <span class="scroll-hint" :class="{ flipped: badgeFlipped }">{{ t('screenshot.scrolling.frameScrollHint') }}</span>

      <!-- 匹配失败提示：滚太快或选区含固定元素，回滚一点即可恢复 -->
      <span v-if="!scrollMatchOk" class="match-lost" :class="{ flipped: badgeFlipped }">
        {{ t('screenshot.scrolling.matchLost') }}
      </span>

      <!-- NEG 预览条：累积长图实时预览，贴在取景框右侧 -->
      <div v-if="scrollThumb" class="neg-strip" :class="{ 'flip-x': negFlipped }">
        <span class="neg-label">NEG {{ String(scrollFrameCount).padStart(2, '0') }}</span>
        <div class="neg-frame">
          <img :src="scrollThumb" alt="" draggable="false" />
        </div>
      </div>
    </div>

    <!-- 长截图操作指引 -->
    <div v-if="mode === 'scroll'" class="film-instruction">{{ t('screenshot.scrolling.instruction') }}</div>

    <!-- 整组定位，设置栏展开时一起避开屏幕边缘。 -->
    <div
      v-if="selection && mode === 'select'"
      v-show="!drag && !busy"
      ref="toolbarRef"
      class="capture-controls"
      :style="toolbarStyle"
      @pointerdown.stop
      @pointermove.stop
      @dblclick.stop
      @mousedown="onToolbarMouseDown"
    >
      <div class="toolbar" role="group" :aria-label="t('screenshot.title')">
        <div class="tb-group" role="group" :aria-label="t('screenshot.annotation')">
          <button
            v-for="item in ANNOTATION_TOOLS"
            :key="item.tool"
            class="tb-btn"
            type="button"
            :aria-pressed="annotationTool === item.tool"
            :aria-label="t(`screenshot.${item.label}`)"
            :title="t(`screenshot.${item.label}`)"
            @click="toggleAnnotation(item.tool)"
          ><v-icon :icon="item.icon" size="18" aria-hidden="true" /></button>
          <button class="tb-btn" type="button" :disabled="!editor?.canUndo" :title="t('screenshot.annoUndo')" :aria-label="t('screenshot.annoUndo')" @click="editor?.undo()">
            <v-icon :icon="mdiUndo" size="18" aria-hidden="true" />
          </button>
          <button class="tb-btn" type="button" :disabled="!editor?.canRedo" :title="t('screenshot.annoRedo')" :aria-label="t('screenshot.annoRedo')" @click="editor?.redo()">
            <v-icon :icon="mdiRedo" size="18" aria-hidden="true" />
          </button>
        </div>
        <div class="tb-group tb-outputs">
          <button
            v-for="item in extraActions"
            :key="item.label"
            class="tb-btn"
            type="button"
            :disabled="item.live && !!editor?.hasEdits"
            :title="t(item.live && editor?.hasEdits ? 'screenshot.liveNeedsUnmarked' : item.label)"
            :aria-label="t(item.label)"
            @click="item.run()"
          ><v-icon :icon="item.icon" size="18" aria-hidden="true" /></button>
          <button class="tb-btn" type="button" :title="t('screenshot.saveShortcut')" :aria-label="t('screenshot.save')" @click="finish('save')">
            <v-icon :icon="mdiContentSaveOutline" size="18" aria-hidden="true" />
          </button>
          <button class="tb-btn" type="button" :title="t('screenshot.cancelShortcut')" :aria-label="t('screenshot.cancel')" @click="cancel">
            <v-icon :icon="mdiClose" size="18" aria-hidden="true" />
          </button>
          <button class="tb-btn tb-primary" type="button" :title="t('screenshot.finishShortcut')" @click="finish('copy')">
            <v-icon :icon="mdiCheck" size="18" aria-hidden="true" />
            {{ t('screenshot.finishCapture') }}
          </button>
        </div>
      </div>
      <div v-if="annotationTool" class="annotation-options" role="group" :aria-label="t('screenshot.annotation')">
        <div v-if="annotationTool !== 'mosaic'" class="tb-group" role="group" :aria-label="t('screenshot.annoColor')">
          <button
            v-for="color in ANNOTATION_COLORS"
            :key="color"
            class="color-choice"
            type="button"
            :aria-pressed="annotationColor === color"
            :aria-label="t('screenshot.captureColor', { color })"
            :title="t('screenshot.captureColor', { color })"
            @click="annotationColor = color"
          ><span :style="{ backgroundColor: color }" aria-hidden="true"></span></button>
        </div>
        <label v-if="annotationTool === 'text'" class="annotation-size">
          {{ t('screenshot.annoFontSize') }}
          <input v-model.number="annotationTextSize" type="range" min="14" max="54" step="2" />
        </label>
        <label v-else class="annotation-size">
          {{ t(annotationTool === 'mosaic' ? 'screenshot.brushSize' : 'screenshot.annoWidth') }}
          <input v-model.number="annotationWidth" type="range" min="2" max="12" />
        </label>
      </div>
    </div>

    <!-- 底部提示（无选区时） -->
    <div v-if="mode === 'select' && !selection && !drag" class="hint">{{ t('screenshot.overlayHint') }}</div>

    <!-- 快门闪白反馈 -->
    <div v-if="flashing" class="flash-overlay" aria-hidden="true"></div>

    <div v-if="errorText" class="error-toast" role="alert">{{ errorText }}</div>
  </div>
</template>

<script setup lang="ts">
import {
  mdiArrowCollapseDown,
  mdiArrowTopRight,
  mdiCheck,
  mdiCheckerboard,
  mdiClose,
  mdiContentSaveOutline,
  mdiEllipseOutline,
  mdiFormatText,
  mdiPencilOutline,
  mdiPinOutline,
  mdiRectangleOutline,
  mdiRedo,
  mdiRobotOutline,
  mdiTextRecognition,
  mdiUndo,
  mdiVectorLine,
  mdiVideoOutline,
} from '@mdi/js'
import { convertFileSrc } from '@tauri-apps/api/core'
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { computed, nextTick, onMounted, onUnmounted, ref, shallowRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  copyScreenshotText,
  type MonitorFrameInfo,
  type PhysicalRect,
  screenshotCancel,
  screenshotFinish,
  screenshotFrameInfo,
  screenshotScrollBegin,
  startRegionRecord,
} from '@/api/screenshot'
// biome-ignore lint/style/useImportType: The Vue template renders this component at runtime.
import CaptureAnnotations from './CaptureAnnotations.vue'
import type { AnnotationTool, Point } from './capture-annotations'

const { t } = useI18n()

interface DragState {
  mode: 'new' | 'move' | 'resize'
  pointerId: number
  anchor: Point
  origin: PhysicalRect | null
  handle?: Handle
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
type Handle = (typeof HANDLES)[number]['id']

const ANNOTATION_TOOLS: { tool: AnnotationTool; icon: string; label: string }[] = [
  { tool: 'rect', icon: mdiRectangleOutline, label: 'toolRect' },
  { tool: 'ellipse', icon: mdiEllipseOutline, label: 'toolEllipse' },
  { tool: 'arrow', icon: mdiArrowTopRight, label: 'toolArrow' },
  { tool: 'line', icon: mdiVectorLine, label: 'toolLine' },
  { tool: 'pen', icon: mdiPencilOutline, label: 'toolPen' },
  { tool: 'mosaic', icon: mdiCheckerboard, label: 'toolMosaic' },
  { tool: 'text', icon: mdiFormatText, label: 'toolText' },
]
const extraActions = [
  { label: 'screenshot.ocr.shortcut', icon: mdiTextRecognition, run: () => finish('ocr') },
  { label: 'screenshot.pin.shortcut', icon: mdiPinOutline, run: () => finish('pin') },
  {
    label: 'screenshot.scrolling.action',
    icon: mdiArrowCollapseDown,
    live: true,
    run: startScrolling,
  },
  { label: 'screenshot.record.action', icon: mdiVideoOutline, live: true, run: startRecording },
  { label: 'screenshot.sendToChat', icon: mdiRobotOutline, run: () => finish('chat') },
]
const ANNOTATION_COLORS = ['#ff4d4f', '#ffba27', '#34d399', '#38bdf8', '#17191f', '#ffffff']
const annotationTool = ref<AnnotationTool | null>(null)
const annotationColor = ref(ANNOTATION_COLORS[0])
const annotationWidth = ref(4)
const annotationTextSize = ref(24)
const editor = ref<InstanceType<typeof CaptureAnnotations> | null>(null)
const rootRef = ref<HTMLElement | null>(null)
const toolbarRef = ref<HTMLElement | null>(null)
const toolbarSize = ref({ width: 600, height: 48 })
const viewport = ref({ width: 1, height: 1 })
let toolbarObserver: ResizeObserver | undefined

const window_ = getCurrentWindow()
// 显示器编号编码在窗口 label 后缀（screenshot-overlay-<n>）
const monitorIndex = Number.parseInt(window_.label.split('-').pop() ?? '0', 10) || 0

const frameSrc = ref('')
const frameEl = ref<HTMLImageElement | null>(null)
const frameInfo = shallowRef<MonitorFrameInfo | null>(null)
const frameReady = ref(false)
const selection = shallowRef<PhysicalRect | null>(null)
const drag = shallowRef<DragState | null>(null)
const cursor = shallowRef<Point | null>(null)
const busy = ref(false)
const flashing = ref(false)
const errorText = ref('')
let errorTimer: number | null = null
let flashTimer: number | null = null
// 录制模式：覆盖窗切换为透明 + 点击穿透（由后端设置），仅渲染边框与 REC 徽标
const mode = ref<'select' | 'record' | 'scroll'>('select')
const scrollFrameCount = ref(1)
const scrollStitchedHeight = ref(0)
const scrollThumb = ref('')
const scrollMatchOk = ref(true)
const recordElapsedMs = ref(0)
let recordStartAt = 0
let recordTimer: number | null = null

const recordElapsedLabel = computed(() => {
  const total = Math.floor(recordElapsedMs.value / 1000)
  const m = Math.floor(total / 60)
  const s = total % 60
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
})

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), Math.max(min, max))
}

// 选区始终使用冻结帧像素，CSS 视口只负责显示，避免混合 DPI 下重复取整。
const cssScale = computed(() => ({
  x: viewport.value.width / (frameInfo.value?.width ?? 1),
  y: viewport.value.height / (frameInfo.value?.height ?? 1),
}))

function cssRect(rect: PhysicalRect) {
  const { x, y } = cssScale.value
  return { x: rect.x * x, y: rect.y * y, width: rect.width * x, height: rect.height * y }
}

function rectStyle(rect: PhysicalRect | null) {
  if (!rect) return undefined
  const { x, y, width, height } = cssRect(rect)
  return { left: `${x}px`, top: `${y}px`, width: `${width}px`, height: `${height}px` }
}

const selectionStyle = computed(() => rectStyle(selection.value))
const sizeLabel = computed(() =>
  selection.value ? `${selection.value.width} × ${selection.value.height}` : ''
)
const badgeFlipped = computed(() => (selection.value?.y ?? 0) * cssScale.value.y < 28)

function targetAt(point: Point | null) {
  const info = frameInfo.value
  if (!point || !info) return null
  return (
    info.windowTargets.find(
      rect =>
        point.x >= rect.x &&
        point.x < rect.x + rect.width &&
        point.y >= rect.y &&
        point.y < rect.y + rect.height
    ) ?? { x: 0, y: 0, width: info.width, height: info.height }
  )
}

const windowTargetStyle = computed(() =>
  mode.value === 'select' && !selection.value && !drag.value
    ? rectStyle(targetAt(cursor.value))
    : undefined
)

const toolbarStyle = computed(() => {
  if (!selection.value) return undefined
  const rect = cssRect(selection.value)
  const { width, height } = toolbarSize.value
  const vw = viewport.value.width
  const vh = viewport.value.height
  const below = rect.y + rect.height + 8
  const above = rect.y - 8 - height
  const top = below + height + 8 <= vh ? below : above >= 8 ? above : vh - height - 8
  return {
    left: `${clamp(rect.x + rect.width - width, 8, vw - width - 8)}px`,
    top: `${clamp(top, 8, vh - height - 8)}px`,
  }
})

// ---- 像素放大镜 ----
const loupeCanvas = ref<HTMLCanvasElement | null>(null)
const loupeHex = ref('')
const LOUPE_PIXELS = 11
const LOUPE_CENTER = Math.floor(LOUPE_PIXELS / 2)
const loupeCoords = computed(() =>
  cursor.value ? `${Math.floor(cursor.value.x)}, ${Math.floor(cursor.value.y)}` : ''
)
const loupeStyle = computed(() => {
  if (
    !cursor.value ||
    !frameReady.value ||
    busy.value ||
    mode.value !== 'select' ||
    (selection.value && !drag.value) ||
    drag.value?.mode === 'move'
  )
    return undefined
  const x = cursor.value.x * cssScale.value.x
  const y = cursor.value.y * cssScale.value.y
  const { width, height } = viewport.value
  const left = x + 16 + 126 <= width ? x + 16 : x - 16 - 126
  const top = y + 16 + 124 <= height ? y + 16 : y - 16 - 124
  return {
    left: `${clamp(left, 4, width - 130)}px`,
    top: `${clamp(top, 4, height - 128)}px`,
  }
})

/** NEG 预览条挂在取景框右侧，贴近屏幕右缘时翻到左侧 */
const negFlipped = computed(() => {
  if (!selection.value) return false
  const rect = cssRect(selection.value)
  return rect.x + rect.width + 110 > viewport.value.width
})

let unlistenScrollFrame: UnlistenFn | undefined
let unlistenRecordFinished: UnlistenFn | undefined
let unlistenSelection: UnlistenFn | undefined
let loupeAnimation = 0

function sampleLoupe() {
  const canvas = loupeCanvas.value
  const frame = frameEl.value
  if (!canvas || !frame || !cursor.value || !frameReady.value) return
  const ctx = canvas.getContext('2d', { willReadFrequently: true })
  if (!ctx) return
  const px = clamp(Math.floor(cursor.value.x), 0, frame.naturalWidth - 1) - LOUPE_CENTER
  const py = clamp(Math.floor(cursor.value.y), 0, frame.naturalHeight - 1) - LOUPE_CENTER
  try {
    ctx.clearRect(0, 0, LOUPE_PIXELS, LOUPE_PIXELS)
    ctx.drawImage(frame, px, py, LOUPE_PIXELS, LOUPE_PIXELS, 0, 0, LOUPE_PIXELS, LOUPE_PIXELS)
    const data = ctx.getImageData(LOUPE_CENTER, LOUPE_CENTER, 1, 1).data
    loupeHex.value = `#${[data[0], data[1], data[2]]
      .map(v => v.toString(16).padStart(2, '0').toUpperCase())
      .join('')}`
  } catch {
    loupeHex.value = ''
  }
}

watch(
  [loupeCanvas, cursor],
  () => {
    if (!loupeCanvas.value || loupeAnimation) return
    loupeAnimation = requestAnimationFrame(() => {
      loupeAnimation = 0
      sampleLoupe()
    })
  },
  { flush: 'post' }
)

function showError(message: string) {
  errorText.value = message
  if (errorTimer) window.clearTimeout(errorTimer)
  errorTimer = window.setTimeout(() => {
    errorText.value = ''
  }, 3000)
}

function framePoint(e: PointerEvent): Point {
  const { x, y } = cssScale.value
  return { x: e.clientX / x, y: e.clientY / y }
}

function focusOverlay() {
  rootRef.value?.focus({ preventScroll: true })
}

function onPointerDown(e: PointerEvent) {
  // 选区外直接重画没有标注的选区。已有标注时先用右键或取消按钮撤回选区。
  if (editor.value?.hasEdits) return
  startDrag(e, 'new')
}

function startDrag(e: PointerEvent, mode_: DragState['mode'], handle?: Handle) {
  if (!frameReady.value || mode.value !== 'select' || e.button !== 0 || busy.value || drag.value)
    return
  e.preventDefault()
  focusOverlay()
  rootRef.value?.setPointerCapture(e.pointerId)
  const anchor = framePoint(e)
  drag.value = { mode: mode_, pointerId: e.pointerId, anchor, origin: selection.value, handle }
  if (mode_ === 'new') {
    annotationTool.value = null
    selection.value = null
  }
  cursor.value = anchor
}

function moveSelection(rect: PhysicalRect, dx: number, dy: number): PhysicalRect {
  const info = frameInfo.value!
  return {
    ...rect,
    x: clamp(rect.x + dx, 0, info.width - rect.width),
    y: clamp(rect.y + dy, 0, info.height - rect.height),
  }
}

function rectFromEdges(left: number, top: number, right: number, bottom: number): PhysicalRect {
  const info = frameInfo.value!
  const x = clamp(Math.round(Math.min(left, right)), 0, info.width - 1)
  const y = clamp(Math.round(Math.min(top, bottom)), 0, info.height - 1)
  return {
    x,
    y,
    width: clamp(Math.round(Math.max(left, right)) - x, 1, info.width - x),
    height: clamp(Math.round(Math.max(top, bottom)) - y, 1, info.height - y),
  }
}

function updateDrag(state: DragState, point: Point) {
  const { anchor, origin } = state
  const dx = Math.round(point.x - anchor.x)
  const dy = Math.round(point.y - anchor.y)
  if (state.mode === 'new') {
    const { x, y } = cssScale.value
    if (Math.hypot(dx * x, dy * y) >= 4 || selection.value) {
      selection.value = rectFromEdges(anchor.x, anchor.y, point.x, point.y)
    }
  } else if (origin && state.mode === 'move') {
    selection.value = moveSelection(origin, dx, dy)
  } else if (origin && state.handle) {
    const { x, y, width, height } = origin
    const handle = state.handle
    selection.value = rectFromEdges(
      x + (handle.includes('w') ? dx : 0),
      y + (handle.includes('n') ? dy : 0),
      x + width + (handle.includes('e') ? dx : 0),
      y + height + (handle.includes('s') ? dy : 0)
    )
  }
}

function onPointerMove(e: PointerEvent) {
  if (!frameReady.value || mode.value !== 'select') return
  const point = framePoint(e)
  const info = frameInfo.value!
  cursor.value = { x: clamp(point.x, 0, info.width - 1), y: clamp(point.y, 0, info.height - 1) }
  const state = drag.value
  if (state?.pointerId === e.pointerId) updateDrag(state, point)
}

function onPointerUp(e: PointerEvent) {
  const state = drag.value
  if (state?.pointerId !== e.pointerId) return
  const point = framePoint(e)
  updateDrag(state, point)
  drag.value = null
  if (state.mode === 'new') {
    selection.value ??= targetAt(point)
    if (selection.value) announceSelection()
  }
}

function cancelDrag(e?: PointerEvent) {
  const state = drag.value
  if (!state || (e && state.pointerId !== e.pointerId)) return
  selection.value = state.origin
  drag.value = null
  const root = rootRef.value
  if (root?.hasPointerCapture(state.pointerId)) root.releasePointerCapture(state.pointerId)
}

function leaveSurface() {
  if (!drag.value) cursor.value = null
}

function announceSelection() {
  void emit('capture-selection', window_.label).catch(() => {})
}

function resetSelection() {
  drag.value = null
  selection.value = null
  annotationTool.value = null
  focusOverlay()
}

function onContextMenu() {
  if (busy.value || mode.value !== 'select') return
  if (selection.value) resetSelection()
  else void cancel()
}

function toggleAnnotation(tool: AnnotationTool) {
  annotationTool.value = annotationTool.value === tool ? null : tool
}

function onToolbarMouseDown(e: MouseEvent) {
  if (e.button !== 0 || !(e.target instanceof Element) || !e.target.closest('button')) return
  // 鼠标点工具后仍由 Enter 完成截图；Tab 聚焦按钮时保留原生键盘行为。
  e.preventDefault()
  focusOverlay()
}

/** 快门闪白：120ms 淡出，告诉用户「拍到了」 */
function shutterFlash() {
  flashing.value = true
  if (flashTimer) window.clearTimeout(flashTimer)
  flashTimer = window.setTimeout(() => {
    flashing.value = false
  }, 200)
}

async function finish(action: 'copy' | 'save' | 'chat' | 'ocr' | 'pin') {
  const rect = selection.value
  if (!rect || busy.value || drag.value || editor.value?.isDrawing || mode.value !== 'select')
    return
  busy.value = true
  errorText.value = ''
  try {
    const annotatedPng = await editor.value?.exportPng()
    if (action === 'copy') shutterFlash()
    const result = await screenshotFinish(
      monitorIndex,
      rect,
      action,
      annotatedPng,
      action === 'save' ? t('screenshot.saveCapture') : undefined
    )
    // 保存对话框取消时，覆盖窗保留当前选区与标注。
    if (!result) focusOverlay()
  } catch (err) {
    showError(t('screenshot.captureFailed', { message: String(err) }))
  } finally {
    busy.value = false
  }
}

/**
 * 进入滚动截长图：后端把本覆窗切成「点击穿透 + 排除捕获」，只保留选区边框；
 * 画面实时可见，用户可以直接在框内滚动页面。
 */
async function startScrolling() {
  if (!selection.value || busy.value || editor.value?.hasEdits) return
  busy.value = true
  try {
    const status = await screenshotScrollBegin(monitorIndex, selection.value)
    scrollFrameCount.value = status.frameCount
    scrollStitchedHeight.value = status.stitchedHeight
    scrollThumb.value = status.stitchedThumb
    scrollMatchOk.value = status.matchOk
    mode.value = 'scroll'
  } catch (err) {
    showError(t('screenshot.scrolling.failed', { message: String(err) }))
  } finally {
    busy.value = false
  }
}

/** 进入选区录屏：本窗口切为透明穿透（后端设置），仅渲染边框与 REC 徽标 */
async function startRecording() {
  if (!selection.value || busy.value || editor.value?.hasEdits) return
  busy.value = true
  const rect = selection.value
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

function confirmDoubleClick(e: MouseEvent) {
  if (!annotationTool.value && !(e.target instanceof HTMLTextAreaElement)) void finish('copy')
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
  if (
    mode.value !== 'select' ||
    !frameReady.value ||
    busy.value ||
    e.defaultPrevented ||
    e.isComposing ||
    e.keyCode === 229
  )
    return
  if (e.target instanceof HTMLTextAreaElement) return

  if (e.key === 'Escape') {
    e.preventDefault()
    if (drag.value) cancelDrag()
    else if (!editor.value?.abortStroke()) void cancel()
    return
  }

  const key = e.key.toLowerCase()
  const shortcut = (e.ctrlKey || e.metaKey) && !e.altKey
  const rect = selection.value
  if (editor.value?.isDrawing) {
    e.preventDefault()
    return
  }
  if (shortcut && key === 'a' && !rect && frameInfo.value && !drag.value) {
    e.preventDefault()
    selection.value = { x: 0, y: 0, width: frameInfo.value.width, height: frameInfo.value.height }
    announceSelection()
    return
  }
  if (!rect) {
    if (shortcut && key === 'c' && loupeStyle.value && loupeHex.value && !drag.value) {
      e.preventDefault()
      void copyScreenshotText(loupeHex.value).catch(err =>
        showError(t('screenshot.captureFailed', { message: String(err) }))
      )
    } else if (e.key === 'Enter' && !drag.value) {
      selection.value = targetAt(cursor.value)
      if (selection.value) announceSelection()
    }
    return
  }
  if (drag.value) return

  if (shortcut && (key === 'z' || key === 'y')) {
    e.preventDefault()
    if (key === 'y' || e.shiftKey) editor.value?.redo()
    else editor.value?.undo()
    return
  }
  if (shortcut && key === 's') {
    e.preventDefault()
    void finish('save')
    return
  }
  if (
    (shortcut && key === 'c') ||
    (e.key === 'Enter' &&
      !(e.target instanceof HTMLButtonElement) &&
      !(e.target instanceof HTMLInputElement))
  ) {
    e.preventDefault()
    void finish('copy')
    return
  }
  if (!shortcut && !e.altKey && e.shiftKey && key === 'c') {
    e.preventDefault()
    void finish('ocr')
    return
  }
  if (!shortcut && !e.altKey && !e.shiftKey && key === 'p') {
    e.preventDefault()
    void finish('pin')
    return
  }
  if (e.target instanceof HTMLInputElement || shortcut) return
  const step = e.shiftKey ? 10 : 1
  let dx = 0
  let dy = 0
  if (e.key === 'ArrowLeft') dx = -step
  else if (e.key === 'ArrowRight') dx = step
  else if (e.key === 'ArrowUp') dy = -step
  else if (e.key === 'ArrowDown') dy = step
  else return
  e.preventDefault()
  if (e.altKey) {
    selection.value = rectFromEdges(
      rect.x,
      rect.y,
      rect.x + Math.max(1, rect.width + dx),
      rect.y + Math.max(1, rect.height + dy)
    )
  } else {
    selection.value = moveSelection(rect, dx, dy)
  }
}

function handleStyle(handle: (typeof HANDLES)[number]) {
  return {
    left: `${handle.x * 100}%`,
    top: `${handle.y * 100}%`,
  }
}

function syncViewport() {
  viewport.value = { width: window.innerWidth, height: window.innerHeight }
}

async function revealOverlay(event: Event) {
  try {
    await (event.target as HTMLImageElement).decode()
    if (frameReady.value) return
    syncViewport()
    frameReady.value = true
    await nextTick()
    await window_.show()
    if (frameInfo.value?.focused) {
      await window_.setFocus()
      if (!selection.value) focusOverlay()
    }
  } catch (err) {
    console.error(err)
    await cancel()
  }
}

onMounted(async () => {
  syncViewport()
  window.addEventListener('resize', syncViewport)
  window.addEventListener('keydown', onKeyDown)
  try {
    unlistenSelection = await listen<string>('capture-selection', event => {
      if (
        event.payload !== window_.label &&
        mode.value === 'select' &&
        !busy.value &&
        !drag.value
      ) {
        selection.value = null
        annotationTool.value = null
        cursor.value = null
      }
    })
  } catch {
    /* 可继续在单个屏幕选取 */
  }
  try {
    // 滚动模式：累积长图实时预览跟随增长（事件在滚动会话期间才发出）
    unlistenScrollFrame = await listen<{
      frameCount: number
      stitchedHeight: number
      stitchedThumb: string
      matchOk: boolean
    }>('scroll-frame', event => {
      scrollFrameCount.value = event.payload.frameCount
      scrollStitchedHeight.value = event.payload.stitchedHeight
      if (event.payload.stitchedThumb) scrollThumb.value = event.payload.stitchedThumb
      scrollMatchOk.value = event.payload.matchOk
    })
  } catch {
    /* 事件监听失败不影响框选 */
  }
  try {
    // 录屏失败：悬浮条可能已在关闭中，覆盖窗兜底展示错误
    unlistenRecordFinished = await listen<{ ok: boolean; message?: string }>(
      'record-finished',
      event => {
        if (!event.payload.ok) {
          showError(t('screenshot.record.failed', { message: event.payload.message ?? '' }))
        }
      }
    )
  } catch {
    /* 事件监听失败不影响框选 */
  }
  try {
    const info = await screenshotFrameInfo(monitorIndex)
    frameInfo.value = info
    if (info.cursor) cursor.value = { x: info.cursor[0], y: info.cursor[1] }
    frameSrc.value = convertFileSrc(info.framePath)
  } catch (err) {
    console.error(err)
    await cancel()
  }
})

watch(toolbarRef, element => {
  toolbarObserver?.disconnect()
  if (!element) return
  toolbarObserver = new ResizeObserver(() => {
    const { offsetWidth: width, offsetHeight: height } = element
    if (width && height) toolbarSize.value = { width, height }
  })
  toolbarObserver.observe(element)
})

onUnmounted(() => {
  toolbarObserver?.disconnect()
  unlistenScrollFrame?.()
  unlistenRecordFinished?.()
  unlistenSelection?.()
  window.removeEventListener('resize', syncViewport)
  window.removeEventListener('keydown', onKeyDown)
  cancelAnimationFrame(loupeAnimation)
  if (errorTimer) window.clearTimeout(errorTimer)
  if (flashTimer) window.clearTimeout(flashTimer)
  if (recordTimer) window.clearInterval(recordTimer)
})
</script>

<style scoped>
.overlay-root {
  position: fixed;
  inset: 0;
  overflow: hidden;
  user-select: none;
  touch-action: none;
  cursor: crosshair;
  background: #05070a;
  outline: none;
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

/* 快门闪白 */
.flash-overlay {
  position: absolute;
  inset: 0;
  background: #fff;
  animation: shutter-flash 120ms ease-out forwards;
  pointer-events: none;
  z-index: 10;
}

@keyframes shutter-flash {
  from {
    opacity: 0.86;
  }
  to {
    opacity: 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  .flash-overlay {
    animation: none;
    opacity: 0;
  }
}

.scroll-frame {
  position: absolute;
  border: 1.5px dashed rgb(var(--accent-rgb));
  border-radius: 2px;
  pointer-events: none;
  z-index: 2;
  box-shadow:
    0 0 0 1px rgba(255, 255, 255, 0.9),
    0 10px 30px rgb(var(--accent-rgb) / 0.12);
}

.scroll-corner {
  position: absolute;
  width: 18px;
  height: 18px;
  border: 3px solid rgb(var(--accent-rgb));
}

.scroll-corner.tl { top: -3px; left: -3px; border-right: 0; border-bottom: 0; border-radius: 3px 0 0 0; }
.scroll-corner.tr { top: -3px; right: -3px; border-left: 0; border-bottom: 0; border-radius: 0 3px 0 0; }
.scroll-corner.bl { bottom: -3px; left: -3px; border-right: 0; border-top: 0; border-radius: 0 0 0 3px; }
.scroll-corner.br { bottom: -3px; right: -3px; border-left: 0; border-top: 0; border-radius: 0 0 3px 0; }

.scroll-badge {
  position: absolute;
  top: -30px;
  left: -1px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 24px;
  padding: 0 9px;
  border-radius: 6px;
  background: rgb(var(--accent-rgb));
  color: #fff;
  font-size: 10.5px;
  font-weight: 600;
  white-space: nowrap;
  box-shadow: 0 6px 16px rgb(var(--accent-rgb) / 0.35);
}

.scroll-badge.flipped {
  top: auto;
  bottom: -30px;
}

.scroll-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: #fff;
  animation: scroll-dot-pulse 1.3s ease-in-out infinite;
}

/* 框内滚动提示：挂在取景框右下角外侧 */
.scroll-hint {
  position: absolute;
  right: 0;
  bottom: -27px;
  display: inline-flex;
  align-items: center;
  height: 20px;
  padding: 0 7px;
  border: 1px solid var(--border);
  border-radius: 5px;
  background: rgb(var(--bg-rgb) / 0.95);
  color: var(--text-muted);
  font: 500 9.5px var(--jedi-font-mono, monospace);
  white-space: nowrap;
}

.scroll-hint.flipped {
  bottom: auto;
  top: -27px;
}

/* 匹配失败提示：取景框上方居中，回滚后自动消失 */
.match-lost {
  position: absolute;
  top: -30px;
  left: 50%;
  transform: translateX(-50%);
  display: inline-flex;
  align-items: center;
  height: 22px;
  padding: 0 9px;
  border-radius: 6px;
  background: #d32f2f;
  color: #fff;
  font-size: 10.5px;
  font-weight: 600;
  white-space: nowrap;
  box-shadow: 0 6px 16px rgba(211, 47, 47, 0.35);
}

.match-lost.flipped {
  top: auto;
  bottom: 4px;
}

/* NEG 预览条：累积长图实时预览，贴在取景框右侧 */
.neg-strip {
  position: absolute;
  right: -104px;
  top: 2px;
  width: 88px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  pointer-events: none;
}

.neg-strip.flip-x {
  right: auto;
  left: -104px;
}

.neg-label {
  font: 500 9px var(--jedi-font-mono, monospace);
  letter-spacing: 0.1em;
  color: var(--text-muted);
}

.neg-frame {
  position: relative;
  max-height: 420px;
  border-radius: 5px;
  background: #fff;
  border: 1px solid var(--border);
  box-shadow: 0 3px 10px rgba(39, 40, 46, 0.1);
  overflow: hidden;
}

.neg-frame img {
  display: block;
  width: 100%;
  height: auto;
}

/* 长截图操作指引：画面底部居中 */
.film-instruction {
  position: absolute;
  left: 50%;
  bottom: 20px;
  transform: translateX(-50%);
  font: 400 10px var(--jedi-font-mono, monospace);
  color: rgb(var(--ink-rgb) / 0.45);
  white-space: nowrap;
  pointer-events: none;
  z-index: 2;
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

.capture-scrim {
  position: absolute;
  inset: 0;
  background: rgba(15, 18, 24, 0.56);
  pointer-events: none;
}

.selection,
.window-target {
  position: absolute;
  /* 描边不占布局空间，标注画布可直接使用选区的完整尺寸。 */
  box-shadow:
    inset 0 0 0 1px #fff,
    0 0 0 1px var(--accent),
    0 0 0 100000px rgba(15, 18, 24, 0.56);
  z-index: 2;
}

.window-target {
  pointer-events: none;
}

.selection { cursor: move; }

.handle {
  position: absolute;
  width: 10px;
  height: 10px;
  transform: translate(-50%, -50%);
  background: #fff;
  border: 1.5px solid var(--accent);
  border-radius: 2px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.28);
}

.handle::before {
  position: absolute;
  inset: -3px;
  content: '';
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
  bottom: calc(100% + 6px);
  left: 0;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding: 0 10px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 7px;
  background: #0f1218;
  color: #fff;
  font: 700 11px/1.4 var(--jedi-font-mono, monospace);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.35);
  white-space: nowrap;
  pointer-events: none;
}

.size-badge.flipped {
  top: 7px;
  bottom: auto;
}

.loupe {
  position: absolute;
  width: 126px;
  padding: 6px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 10px;
  background: #0f1218;
  box-shadow: 0 10px 26px rgba(0, 0, 0, 0.4);
  pointer-events: none;
  z-index: 3;
}

.loupe-grid {
  position: relative;
  display: grid;
  place-items: center;
}

.loupe-grid::after {
  content: '';
  position: absolute;
  inset: calc(50% - 4px);
  outline: 1px solid #fff;
  box-shadow: 0 0 0 2px #101216;
}

.loupe-canvas {
  display: block;
  width: 99px;
  height: 99px;
  border-radius: 5px;
  image-rendering: pixelated;
}

.loupe-meta {
  display: flex;
  justify-content: space-between;
  gap: 4px;
  margin-top: 5px;
  font: 500 10px/1.2 var(--jedi-font-mono, monospace);
  color: #f0f2f6;
}

.capture-controls {
  position: absolute;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 5px;
  max-width: calc(100vw - 16px);
  cursor: default;
  z-index: 4;
  animation: toolbar-enter 140ms ease-out;
}

.toolbar,
.annotation-options {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 4px 6px;
  max-width: 100%;
  padding: 5px 6px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 11px;
  background: rgba(15, 18, 24, 0.97);
  box-shadow: 0 20px 46px rgba(0, 0, 0, 0.45), inset 0 1px 0 rgba(255, 255, 255, 0.045);
  color: #f3f5f9;
}

.tb-group {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 2px;
  min-width: 0;
}

.tb-outputs {
  border-inline-start: 1px solid rgba(255, 255, 255, 0.16);
  padding-inline-start: 6px;
}

.tb-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  min-width: 32px;
  height: 34px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: inherit;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background var(--motion-fast) ease;
}

.tb-btn:enabled:hover {
  background: rgba(255, 255, 255, 0.1);
}

.tb-btn:focus-visible,
.color-choice:focus-visible,
.annotation-size input:focus-visible {
  outline: 2px solid #69b8ff;
  outline-offset: 1px;
}

.tb-btn[aria-pressed='true'] {
  background: rgba(83, 155, 245, 0.22);
  color: #b8d8ff;
  box-shadow: inset 0 0 0 1px rgba(83, 155, 245, 0.6);
}

.tb-btn:disabled {
  opacity: 0.42;
  cursor: default;
}

.tb-primary {
  padding: 0 10px;
  background: #2f68e0;
  color: #fff;
  font-weight: 600;
  white-space: nowrap;
}

.tb-primary:enabled:hover {
  background: #2456bf;
}

.color-choice {
  display: grid;
  place-items: center;
  width: 26px;
  height: 28px;
  border-radius: 6px;
  cursor: pointer;
}

.color-choice span {
  width: 15px;
  height: 15px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.35);
}

.color-choice:hover { background: rgba(255, 255, 255, 0.1); }
.color-choice[aria-pressed='true'] span { outline: 2px solid #fff; outline-offset: 2px; }

.annotation-size {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 28px;
  padding: 0 4px;
  color: #e1e6ee;
  font-size: 11px;
}

.annotation-size input {
  width: 84px;
  accent-color: #69b8ff;
  cursor: pointer;
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
  z-index: 3;
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
  z-index: 5;
}

@keyframes toolbar-enter {
  from { opacity: 0; }
  to { opacity: 1; }
}

@media (prefers-reduced-motion: reduce) {
  .capture-controls, .rec-dot { animation: none; }
  .tb-btn { transition: none; }
}
</style>
