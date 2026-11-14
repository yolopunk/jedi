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
    <img v-if="frameSrc && mode !== 'record' && mode !== 'scroll'" ref="frameEl" class="frozen-frame" :src="frameSrc" @load="revealOverlay" @error="cancel" draggable="false" alt="" />

    <!-- 未开始框选时的十字引导线 -->
    <template v-if="!selection && !drag">
      <div v-if="cursor" class="guide guide-v" :style="{ left: `${cursor.x}px` }"></div>
      <div v-if="cursor" class="guide guide-h" :style="{ top: `${cursor.y}px` }"></div>
    </template>

    <!-- 取景状态徽标：左上角常驻 -->
    <div v-if="mode === 'select'" class="viewfinder-pill">
      <span class="vp-dot" aria-hidden="true"></span>
      {{ t('screenshot.viewfinder.label', { display: monitorIndex + 1 }) }}
    </div>

    <!-- 选区：白描边 + 蓝色外圈，选区外压暗 -->
    <div
      v-if="selection && mode === 'select'"
      class="selection"
      :style="selectionStyle"
      @mousedown.stop="onSelectionMouseDown"
      @dblclick.stop="confirmCopy"
    >
      <div class="selection-inner" aria-hidden="true"></div>
      <div
        v-for="handle in HANDLES"
        :key="handle.id"
        class="handle"
        :class="[`handle-${handle.id}`]"
        :style="handleStyle(handle)"
        @mousedown.stop="onHandleMouseDown(handle.id, $event)"
      ></div>
      <div class="size-badge" :class="{ flipped: badgeFlipped }">
        {{ sizeLabel }}
        <span class="badge-scale">· {{ scaleLabel }}</span>
      </div>

      <!-- 像素放大镜：挂在选区右下角外侧 -->
      <div v-if="loupeVisible" class="loupe" :class="loupeFlip" :style="loupeStyle" aria-hidden="true">
        <canvas ref="loupeCanvas" class="loupe-canvas" width="4" height="4"></canvas>
        <div class="loupe-meta"><span>8×</span><span>{{ loupeHex }}</span></div>
      </div>
    </div>

    <!-- 录制模式：保留选区边框 + REC 徽标与已录时长（窗口已点击穿透且排除出捕获） -->
    <div v-if="mode === 'record' && selection" class="record-frame" :style="selectionStyle">
      <div class="rec-badge" :class="{ flipped: badgeFlipped }">
        <span class="rec-dot"></span>REC {{ recordElapsedLabel }}
      </div>
    </div>

    <!-- 滚动截长图：保留取景框（画面实时可见、可自由滚动），窗口已点击穿透并排除出捕获 -->
    <div v-if="mode === 'scroll' && selection" class="scroll-frame" :style="selectionStyle">
      <span class="scroll-badge" :class="{ flipped: badgeFlipped }">
        <span class="scroll-dot" aria-hidden="true"></span>
        {{ t('screenshot.scrolling.frameHint', { count: scrollFrameCount }) }}
      </span>
      <span class="scroll-corner tl" aria-hidden="true"></span>
      <span class="scroll-corner tr" aria-hidden="true"></span>
      <span class="scroll-corner bl" aria-hidden="true"></span>
      <span class="scroll-corner br" aria-hidden="true"></span>
      <span class="scroll-hint" :class="{ flipped: badgeFlipped }">{{ t('screenshot.scrolling.frameScrollHint') }}</span>

      <!-- NEG 胶片条：已捕获帧贴在取景框右侧 -->
      <div v-if="scrollThumbs.length" class="neg-strip" :class="{ 'flip-x': negFlipped }">
        <span class="neg-label">NEG {{ String(scrollThumbs.length).padStart(2, '0') }}</span>
        <div v-for="(thumb, i) in scrollThumbs" :key="i" class="neg-frame">
          <img :src="thumb" alt="" draggable="false" />
          <span class="neg-no">{{ String(i + 1).padStart(2, '0') }}</span>
        </div>
      </div>
    </div>

    <!-- 长截图操作指引 -->
    <div v-if="mode === 'scroll'" class="film-instruction">{{ t('screenshot.scrolling.instruction') }}</div>

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
        <v-icon :icon="mdiContentCopy" size="15" aria-hidden="true" />
        {{ t('screenshot.copy') }}
      </button>
      <button class="tb-btn" type="button" @click.stop="finish('save')">
        <v-icon :icon="mdiContentSaveOutline" size="15" aria-hidden="true" />
        {{ t('screenshot.save') }}
      </button>

      <span class="tb-divider" aria-hidden="true"></span>

      <button class="tb-btn" type="button" :title="t('screenshot.ocr.shortcut')" @click.stop="finish('ocr')">
        <v-icon :icon="mdiTextRecognition" size="16" aria-hidden="true" />
        {{ t('screenshot.ocr.title') }}
        <kbd>⇧C</kbd>
      </button>
      <button class="tb-btn tb-highlight" type="button" @click.stop="startScrolling">
        <v-icon :icon="mdiArrowCollapseDown" size="16" aria-hidden="true" />
        {{ t('screenshot.scrolling.action') }}
      </button>
      <button class="tb-btn" type="button" @click.stop="startRecording">
        <v-icon :icon="mdiVideoOutline" size="16" aria-hidden="true" />
        {{ t('screenshot.record.action') }}
      </button>

      <span class="tb-divider" aria-hidden="true"></span>

      <button class="tb-btn tb-primary" type="button" @click.stop="finish('chat')">
        <v-icon :icon="mdiRobotOutline" size="14" aria-hidden="true" />
        {{ t('screenshot.sendToChat') }}
      </button>
      <button class="tb-btn tb-icon" type="button" :aria-label="t('screenshot.cancel')" @click.stop="cancel">
        <v-icon :icon="mdiClose" size="16" aria-hidden="true" />
      </button>
    </div>

    <!-- 快捷键提示：紧跟在操作条下方 -->
    <div v-if="selection && !busy && mode === 'select'" ref="hintRef" class="toolbar-hints" :style="hintStyle">
      <span><b>Enter</b> {{ t('screenshot.copy') }}</span>
      <span><b>Esc</b> {{ t('screenshot.cancel') }}</span>
      <span><b>⇧C</b> {{ t('screenshot.ocr.title') }}</span>
      <span><b>P</b> {{ t('screenshot.pin.action') }}</span>
      <span><b>{{ t('screenshot.viewfinder.doubleClick') }}</b> {{ t('screenshot.viewfinder.copyClose') }}</span>
    </div>

    <!-- 底部提示（无选区时） -->
    <div v-if="!selection && !drag" class="hint">{{ t('screenshot.overlayHint') }}</div>

    <!-- 快门闪白反馈 -->
    <div v-if="flashing" class="flash-overlay" aria-hidden="true"></div>

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
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
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

const TOOLBAR_HEIGHT = 46
const TOOLBAR_HINT_GAP = 10
const TOOLBAR_MARGIN = 8
const toolbarRef = ref<HTMLElement | null>(null)
const hintRef = ref<HTMLElement | null>(null)
const toolbarWidth = ref(360)
let toolbarObserver: ResizeObserver | null = null
const MIN_SELECTION = 3

const window_ = getCurrentWindow()
// 显示器编号编码在窗口 label 后缀（screenshot-overlay-<n>）
const monitorIndex = Number.parseInt(window_.label.split('-').pop() ?? '0', 10) || 0

const frameSrc = ref('')
const frameEl = ref<HTMLImageElement | null>(null)
const scaleFactor = ref(1)
const selection = ref<Rect | null>(null)
const drag = ref<DragState | null>(null)
const cursor = ref<{ x: number; y: number } | null>(null)
const busy = ref(false)
const flashing = ref(false)
const errorText = ref('')
let errorTimer: number | null = null
let flashTimer: number | null = null
// 录制模式：覆盖窗切换为透明 + 点击穿透（由后端设置），仅渲染边框与 REC 徽标
const mode = ref<'select' | 'record' | 'scroll'>('select')
const scrollFrameCount = ref(1)
const scrollThumbs = ref<string[]>([])
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

const scaleLabel = computed(() => {
  const s = scaleFactor.value
  return `${Number.isInteger(s) ? s : s.toFixed(1)}×`
})

// 尺寸角标默认悬在选区上方居中，贴近屏幕顶时翻到选区内
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

const hintStyle = computed(() => {
  if (!selection.value) return undefined
  const { x, y, w, h } = selection.value
  const vw = window.innerWidth
  const vh = window.innerHeight
  const hintWidth = hintRef.value?.offsetWidth ?? 420
  const maxLeft = Math.max(8, vw - hintWidth - 8)
  const left = Math.min(Math.max(x + w / 2 - hintWidth / 2, 8), maxLeft)
  const below = y + h + TOOLBAR_MARGIN + TOOLBAR_HEIGHT + TOOLBAR_HINT_GAP
  const top =
    below <= vh ? below : Math.max(y - TOOLBAR_MARGIN - TOOLBAR_HEIGHT - TOOLBAR_HINT_GAP, 8)
  return { left: `${left}px`, top: `${top}px` }
})

// ---- 像素放大镜 ----
const loupeCanvas = ref<HTMLCanvasElement | null>(null)
const loupeHex = ref('#FFFFFF')
const LOUPE_SIZE = 96

const loupeVisible = computed(
  () => !!selection.value && mode.value === 'select' && !drag.value && !busy.value
)

/** 放大镜挂在选区右下角外侧，越界时翻进屏幕 */
const loupeFlip = computed(() => {
  if (!selection.value) return ''
  const { x, y, w, h } = selection.value
  const vw = window.innerWidth
  const vh = window.innerHeight
  const classes: string[] = []
  if (x + w + LOUPE_SIZE > vw) classes.push('flip-x')
  if (y + h + LOUPE_SIZE > vh) classes.push('flip-y')
  return classes.join(' ')
})

const loupeStyle = computed(() => undefined)

/** NEG 胶片条挂在取景框右侧，贴近屏幕右缘时翻到左侧 */
const negFlipped = computed(() => {
  if (!selection.value) return false
  return selection.value.x + selection.value.w + 96 > window.innerWidth
})

let unlistenScrollFrame: UnlistenFn | undefined

function sampleLoupe() {
  const canvas = loupeCanvas.value
  const frame = frameEl.value
  if (!canvas || !frame || !cursor.value || !scaleFactor.value) return
  const ctx = canvas.getContext('2d', { willReadFrequently: true })
  if (!ctx) return
  const px = Math.max(0, Math.round(cursor.value.x * scaleFactor.value) - 2)
  const py = Math.max(0, Math.round(cursor.value.y * scaleFactor.value) - 2)
  const sx = Math.min(px, Math.max(0, frame.naturalWidth - 4))
  const sy = Math.min(py, Math.max(0, frame.naturalHeight - 4))
  try {
    ctx.drawImage(frame, sx, sy, 4, 4, 0, 0, 4, 4)
    const data = ctx.getImageData(2, 2, 1, 1).data
    loupeHex.value = `#${[data[0], data[1], data[2]]
      .map(v => v.toString(16).padStart(2, '0').toUpperCase())
      .join('')}`
  } catch {
    /* 采样失败时保留上一次颜色 */
  }
}

watch(loupeCanvas, canvas => {
  if (!canvas) return
  const ctx = canvas.getContext('2d', { willReadFrequently: true })
  ctx?.clearRect(0, 0, canvas.width, canvas.height)
  sampleLoupe()
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
  if (loupeVisible.value) sampleLoupe()
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

/** 快门闪白：120ms 淡出，告诉用户「拍到了」 */
function shutterFlash() {
  flashing.value = true
  if (flashTimer) window.clearTimeout(flashTimer)
  flashTimer = window.setTimeout(() => {
    flashing.value = false
  }, 200)
}

async function finish(action: 'copy' | 'save' | 'chat' | 'ocr' | 'pin') {
  if (!selection.value || busy.value) return
  busy.value = true
  shutterFlash()
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
    const status = await screenshotScrollBegin(monitorIndex, physicalRect())
    scrollFrameCount.value = status.frameCount
    scrollThumbs.value = [...status.thumbs]
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
  if (selection.value && !e.shiftKey && !e.ctrlKey && !e.metaKey && e.key.toLowerCase() === 'p') {
    e.preventDefault()
    finish('pin')
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
    left: `calc(${handle.x * 100}% - ${handle.x === 0.5 ? 4.5 : 4.5}px)`,
    top: `calc(${handle.y * 100}% - ${handle.y === 0.5 ? 4.5 : 4.5}px)`,
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
    // 滚动模式：NEG 胶片条跟随帧数增长（事件在滚动会话期间才发出）
    unlistenScrollFrame = await listen<{ frameCount: number; thumbs: string[] }>(
      'scroll-frame',
      event => {
        scrollFrameCount.value = event.payload.frameCount
        if (event.payload.thumbs?.length) scrollThumbs.value = [...event.payload.thumbs]
      }
    )
  } catch {
    /* 事件监听失败不影响框选 */
  }
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
  unlistenScrollFrame?.()
  window.removeEventListener('keydown', onKeyDown)
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

/* 取景状态徽标：左上角 */
.viewfinder-pill {
  position: absolute;
  top: 28px;
  left: 34px;
  z-index: 4;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 999px;
  background: rgba(15, 18, 24, 0.72);
  font: 500 9.5px/1 var(--jedi-font-mono, monospace);
  letter-spacing: 0.12em;
  color: rgba(255, 255, 255, 0.72);
  pointer-events: none;
  white-space: nowrap;
}

.vp-dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: #3574f0;
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

/* NEG 胶片条：已捕获帧贴在取景框右侧 */
.neg-strip {
  position: absolute;
  right: -88px;
  top: 2px;
  width: 72px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  pointer-events: none;
}

.neg-strip.flip-x {
  right: auto;
  left: -88px;
}

.neg-label {
  font: 500 9px var(--jedi-font-mono, monospace);
  letter-spacing: 0.1em;
  color: var(--text-muted);
}

.neg-frame {
  position: relative;
  aspect-ratio: 4 / 3;
  border-radius: 5px;
  background: #fff;
  border: 1px solid var(--border);
  box-shadow: 0 3px 10px rgba(39, 40, 46, 0.1);
  overflow: hidden;
}

.neg-frame img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.neg-no {
  position: absolute;
  right: 3px;
  bottom: 2px;
  font: 700 7.5px var(--jedi-font-mono, monospace);
  color: rgb(var(--accent-rgb));
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
  border: 1px solid rgba(255, 255, 255, 0.92);
  /* 蓝色外圈 + 选区外整体压暗 */
  box-shadow:
    0 0 0 1px rgba(53, 116, 240, 0.9),
    0 24px 60px rgba(0, 0, 0, 0.35),
    0 0 0 100000px rgba(15, 18, 24, 0.56);
  cursor: move;
  z-index: 2;
}

/* 选区内侧蓝线 */
.selection-inner {
  position: absolute;
  inset: 0;
  border: 1px solid rgba(53, 116, 240, 0.55);
  pointer-events: none;
}

.handle {
  position: absolute;
  width: 9px;
  height: 9px;
  background: #fff;
  border: 1.5px solid #3574f0;
  border-radius: 2px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.28);
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
  top: -13px;
  left: 50%;
  transform: translateX(-50%);
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
}

.badge-scale {
  color: rgba(255, 255, 255, 0.42);
  font-weight: 400;
}

/* 像素放大镜：挂在选区右下角外侧，越界翻入屏幕 */
.loupe {
  position: absolute;
  right: -96px;
  bottom: -96px;
  width: 84px;
  padding: 5px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 9px;
  background: #0f1218;
  box-shadow: 0 10px 26px rgba(0, 0, 0, 0.4);
  pointer-events: none;
  z-index: 3;
}

.loupe.flip-x {
  right: auto;
  left: -96px;
}

.loupe.flip-y {
  bottom: auto;
  top: -96px;
}

.loupe-canvas {
  display: block;
  width: 74px;
  height: 74px;
  border-radius: 5px;
  image-rendering: pixelated;
}

.loupe-meta {
  display: flex;
  justify-content: space-between;
  margin-top: 4px;
  font: 400 8px var(--jedi-font-mono, monospace);
  color: rgba(255, 255, 255, 0.5);
}

.toolbar {
  position: absolute;
  display: flex;
  align-items: center;
  gap: 6px;
  height: 46px;
  padding: 0 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 13px;
  background: rgba(15, 18, 24, 0.94);
  box-shadow: 0 20px 46px rgba(0, 0, 0, 0.45), inset 0 1px 0 rgba(255, 255, 255, 0.045);
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
  gap: 6px;
  height: 30px;
  padding: 0 11px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: rgba(255, 255, 255, 0.82);
  font-size: 12px;
  font-weight: 500;
  line-height: 1.5;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease, box-shadow 140ms ease;
}

.tb-icon {
  justify-content: center;
  width: 30px;
  padding-inline: 0;
  color: rgba(255, 255, 255, 0.6);
}

.tb-btn kbd {
  padding: 0 4px;
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.07);
  color: rgba(255, 255, 255, 0.42);
  font: 400 9px/1.5 var(--jedi-font-mono, monospace);
}

.tb-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.tb-btn:focus-visible {
  outline: 2px solid #69b8ff;
  outline-offset: 1px;
}

/* 截长图：取景条中的高亮项 */
.tb-highlight {
  background: rgba(53, 116, 240, 0.18);
  border: 1px solid rgba(53, 116, 240, 0.5);
  color: #9cc2ff;
  font-weight: 600;
}

.tb-highlight:hover {
  background: rgba(53, 116, 240, 0.28);
  color: #b8d2ff;
}

.tb-primary {
  background: #3574f0;
  color: #fff;
  font-weight: 600;
  box-shadow: 0 4px 12px rgba(53, 116, 240, 0.4);
}

.tb-primary:hover {
  background: #2f68e0;
  color: #fff;
}

.tb-divider {
  width: 1px;
  height: 20px;
  margin: 0 3px;
  background: rgba(255, 255, 255, 0.16);
}

/* 快捷键提示行 */
.toolbar-hints {
  position: absolute;
  display: flex;
  align-items: center;
  gap: 14px;
  font: 400 10px var(--jedi-font-mono, monospace);
  color: rgba(255, 255, 255, 0.5);
  white-space: nowrap;
  pointer-events: none;
  z-index: 3;
}

.toolbar-hints b {
  color: rgba(255, 255, 255, 0.82);
  font-weight: 600;
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
