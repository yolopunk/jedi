<template>
  <div class="beautify-editor">
    <div class="canvas-stage">
      <div class="stage-bar">
        <span><span class="live-dot" aria-hidden="true"></span>{{ t('screenshot.livePreview') }}</span>
        <span v-if="ready">{{ exportWidth }} × {{ exportHeight }}</span>
      </div>
      <div class="anno-bar" role="toolbar" :aria-label="t('screenshot.annotation')">
        <div class="anno-group" role="group">
          <button v-for="tl in TOOL_DEFS" :key="tl.id" type="button" class="tool-btn" :class="{ active: tool === tl.id }" :aria-pressed="tool === tl.id" :title="t(`screenshot.${tl.label}`)" :aria-label="t(`screenshot.${tl.label}`)" @click="setTool(tl.id)">
            <v-icon :icon="tl.icon" size="16" aria-hidden="true" />
          </button>
        </div>
        <span class="anno-divider" aria-hidden="true"></span>
        <div class="anno-group" role="group" :aria-label="t('screenshot.annoColor')">
          <button v-for="c in ANNO_COLORS" :key="c" type="button" class="color-dot" :class="{ active: annoColor === c }" :style="{ background: c }" :aria-pressed="annoColor === c" :aria-label="t('screenshot.annoColor')" @click="annoColor = c"></button>
        </div>
        <span class="anno-divider" aria-hidden="true"></span>
        <label class="width-ctrl"><span>{{ t('screenshot.annoWidth') }}</span><input v-model.number="annoWidth" type="range" min="2" max="16" step="1" :aria-label="t('screenshot.annoWidth')" /></label>
        <label v-if="tool === 'text'" class="width-ctrl"><span>{{ t('screenshot.annoFontSize') }}</span><input v-model.number="textSize" type="range" min="14" max="80" step="2" :aria-label="t('screenshot.annoFontSize')" /></label>
        <span class="anno-spacer" aria-hidden="true"></span>
        <button type="button" class="tool-btn" :disabled="!canUndo" :title="t('screenshot.annoUndo')" :aria-label="t('screenshot.annoUndo')" @click="undo">
          <v-icon :icon="mdiUndo" size="16" aria-hidden="true" />
        </button>
        <button type="button" class="tool-btn" :disabled="!annos.length" :title="t('screenshot.annoClear')" :aria-label="t('screenshot.annoClear')" @click="clearAnnos">
          <v-icon :icon="mdiEraser" size="16" aria-hidden="true" />
        </button>
      </div>
      <div ref="viewport" class="canvas-viewport">
        <canvas
          ref="canvas"
          :aria-label="t('screenshot.beautify')"
          :class="`cursor-${cursorKind}`"
          @pointerdown="onPointerDown"
          @pointermove="onPointerMove"
          @pointerup="onPointerUp"
          @pointercancel="onPointerUp"
        />
        <input
          v-if="editing"
          ref="editInput"
          v-model="editValue"
          class="anno-editor"
          :style="editorStyle"
          :placeholder="t('screenshot.textPlaceholder')"
          @keydown.enter.prevent="commitText"
          @keydown.esc.prevent="commitText"
          @blur="commitText"
        />
      </div>
    </div>
    <aside class="controls">
      <div class="controls-scroll">
        <header class="controls-header">
          <span class="controls-icon"><v-icon :icon="mdiAutoFix" size="18" aria-hidden="true" /></span>
          <div><h3>{{ t('screenshot.beautify') }}</h3><p>{{ t('screenshot.beautifyHint') }}</p></div>
        </header>
        <section class="control-section">
          <div class="section-heading"><span>{{ t('screenshot.background') }}</span></div>
          <div class="swatches" :aria-label="t('screenshot.background')">
            <button v-for="preset in presets" :key="preset.id" type="button" :aria-label="t(`screenshot.presetNames.${preset.id}`)" :aria-pressed="from === preset.from && to === preset.to" @click="from = preset.from; to = preset.to">
              <span :style="{ background: `linear-gradient(135deg, ${preset.from}, ${preset.to})` }"></span>
              <small>{{ t(`screenshot.presetNames.${preset.id}`) }}</small>
            </button>
          </div>
          <div class="color-row">
            <label><span>{{ t('screenshot.colorFrom') }}</span><span class="color-field"><input v-model="from" type="color" /><code>{{ from.toUpperCase() }}</code></span></label>
            <label><span>{{ t('screenshot.colorTo') }}</span><span class="color-field"><input v-model="to" type="color" /><code>{{ to.toUpperCase() }}</code></span></label>
          </div>
        </section>
        <section class="control-section geometry-section">
          <div class="section-heading"><span>{{ t('screenshot.appearance') }}</span><button type="button" class="reset-btn" @click="resetParams"><v-icon :icon="mdiRestore" size="15" aria-hidden="true" />{{ t('screenshot.resetParams') }}</button></div>
          <label class="slider-label"><span>{{ t('screenshot.padding') }}</span><span class="value-input"><input v-model.number="padding" class="num-input" type="number" min="0" max="160" step="4" /><small>px</small></span><input v-model.number="padding" type="range" min="0" max="160" step="4" :aria-label="t('screenshot.padding')" /></label>
          <label class="slider-label"><span>{{ t('screenshot.radius') }}</span><span class="value-input"><input v-model.number="radius" class="num-input" type="number" min="0" max="64" step="2" /><small>px</small></span><input v-model.number="radius" type="range" min="0" max="64" step="2" :aria-label="t('screenshot.radius')" /></label>
          <label class="slider-label"><span>{{ t('screenshot.shadow') }}</span><span class="value-input"><input v-model.number="shadow" class="num-input" type="number" min="0" max="80" step="2" /><small>px</small></span><input v-model.number="shadow" type="range" min="0" max="80" step="2" :aria-label="t('screenshot.shadow')" /></label>
        </section>
        <v-alert v-if="error" type="error" variant="tonal" density="compact">{{ error }}</v-alert>
      </div>
      <div class="export-panel">
        <div v-if="ready" class="export-info">
          <strong>{{ t('screenshot.exportSize', { w: exportWidth, h: exportHeight }) }}</strong>
          <span>{{ t('screenshot.saveAsCopy') }}</span>
        </div>
        <v-btn color="primary" rounded="lg" :prepend-icon="mdiContentSaveOutline" :loading="saving" :disabled="!ready" @click="save">{{ t('screenshot.saveStyled') }}</v-btn>
      </div>
    </aside>
  </div>
</template>

<script setup lang="ts">
import {
  mdiArrowTopRight,
  mdiAutoFix,
  mdiContentSaveOutline,
  mdiCursorDefaultOutline,
  mdiEraser,
  mdiFormatText,
  mdiPencilOutline,
  mdiRectangleOutline,
  mdiRestore,
  mdiUndo,
  mdiVectorLine,
} from '@mdi/js'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { readImageDataUrl, saveStyledScreenshot } from '@/api/screenshot'

const props = defineProps<{ path: string }>()
const emit = defineEmits<{ saved: [path: string] }>()
const { t } = useI18n()

// ---------- 背景美化参数 ----------
const presets = [
  { id: 'mist', from: '#a5b4fc', to: '#fbcfe8' },
  { id: 'ocean', from: '#38bdf8', to: '#134e4a' },
  { id: 'sunset', from: '#fb923c', to: '#fce7f3' },
  { id: 'slate', from: '#27272a', to: '#52525b' },
]
const from = ref(presets[0].from)
const to = ref(presets[0].to)
const padding = ref(64)
const radius = ref(16)
const shadow = ref(32)
const canvas = ref<HTMLCanvasElement | null>(null)
const viewport = ref<HTMLElement | null>(null)
const ready = ref(false)
const saving = ref(false)
const error = ref('')
const exportWidth = ref(0)
const exportHeight = ref(0)
let image: HTMLImageElement | null = null
let loadVersion = 0
let draftTimer: number | undefined
let draftTimerPath = ''

// ---------- 标注 ----------
type Pt = { x: number; y: number }
type Tool = 'select' | 'text' | 'line' | 'arrow' | 'rect' | 'pen'
interface AnnoBase {
  id: number
  color: string
  width: number
}
interface TextAnno extends AnnoBase {
  kind: 'text'
  x: number
  y: number
  size: number
  text: string
}
interface SegmentAnno extends AnnoBase {
  kind: 'line' | 'arrow'
  x1: number
  y1: number
  x2: number
  y2: number
}
interface RectAnno extends AnnoBase {
  kind: 'rect'
  x: number
  y: number
  w: number
  h: number
}
interface PenAnno extends AnnoBase {
  kind: 'pen'
  points: Pt[]
}
type Anno = TextAnno | SegmentAnno | RectAnno | PenAnno
type DrawAnno = SegmentAnno | RectAnno | PenAnno
// 标注坐标统一存图片本地坐标（不含留白），渲染时按 padding 平移，参数调整不丢位置。
type Gesture =
  | { kind: 'draw'; anno: DrawAnno; snapshot: Anno[]; significant: boolean }
  | { kind: 'move'; id: number; orig: Anno; start: Pt; snapshot: Anno[] | null; isText: boolean }

const ANNO_COLORS = ['#ff4d4f', '#fbbf24', '#34d399', '#38bdf8', '#111827', '#ffffff']
const ANNO_FONT_STACK = '"Segoe UI", "Microsoft YaHei", "PingFang SC", system-ui, sans-serif'
const TOOL_DEFS: { id: Tool; icon: string; label: string }[] = [
  { id: 'select', icon: mdiCursorDefaultOutline, label: 'toolSelect' },
  { id: 'text', icon: mdiFormatText, label: 'toolText' },
  { id: 'line', icon: mdiVectorLine, label: 'toolLine' },
  { id: 'arrow', icon: mdiArrowTopRight, label: 'toolArrow' },
  { id: 'rect', icon: mdiRectangleOutline, label: 'toolRect' },
  { id: 'pen', icon: mdiPencilOutline, label: 'toolPen' },
]

const tool = ref<Tool>('select')
const annoColor = ref(ANNO_COLORS[0])
const annoWidth = ref(4)
const textSize = ref(28)
const annos = ref<Anno[]>([])
const selectedId = ref<number | null>(null)
const canUndo = ref(false)
const editing = ref(false)
const editId = ref(0)
const editValue = ref('')
const editColor = ref(ANNO_COLORS[0])
const editPos = ref({ left: 0, top: 0, fontSize: 16, width: 160 })
const editInput = ref<HTMLInputElement | null>(null)

let nextAnnoId = 1
let history: Anno[][] = []
let gesture: Gesture | null = null
let creatingText = false
let pendingText: TextAnno | null = null
let editSnapshot: Anno[] | null = null
let imageW = 0
let imageH = 0
let drawScale = 1
let rafId = 0
let resizeObserver: ResizeObserver | null = null
const measureCanvas = document.createElement('canvas')
const measureCtx = measureCanvas.getContext('2d')
const textMeasure = new Map<string, number>()

const cursorKind = computed(() =>
  tool.value === 'select' ? 'default' : tool.value === 'text' ? 'text' : 'crosshair'
)
const editorStyle = computed(() => ({
  left: `${editPos.value.left}px`,
  top: `${editPos.value.top}px`,
  fontSize: `${editPos.value.fontSize}px`,
  width: `${editPos.value.width}px`,
  color: editColor.value,
}))

function annoFont(size: number) {
  return `600 ${size}px ${ANNO_FONT_STACK}`
}

function measureTextWidth(text: string, size: number): number {
  if (!measureCtx) return text.length * size * 0.6
  const key = `${size}|${text}`
  let width = textMeasure.get(key)
  if (width === undefined) {
    measureCtx.font = annoFont(size)
    width = measureCtx.measureText(text).width
    if (textMeasure.size > 400) textMeasure.clear()
    textMeasure.set(key, width)
  }
  return width
}

// 浅色文字加深色描边、深色文字加白色描边，保证在任意截图内容上可读。
function contrastOutline(color: string): string {
  const m = /^#?([0-9a-f]{6})$/i.exec(color.trim())
  if (!m) return 'rgba(255, 255, 255, 0.85)'
  const int = Number.parseInt(m[1], 16)
  const luminance =
    (0.299 * ((int >> 16) & 255) + 0.587 * ((int >> 8) & 255) + 0.114 * (int & 255)) / 255
  return luminance > 0.7 ? 'rgba(17, 20, 28, 0.62)' : 'rgba(255, 255, 255, 0.85)'
}

function cloneAnnos(list: Anno[]): Anno[] {
  return list.map(a =>
    a.kind === 'pen' ? { ...a, points: a.points.map(p => ({ ...p })) } : { ...a }
  )
}

function pushUndo(snapshot?: Anno[]) {
  history.push(snapshot ?? cloneAnnos(annos.value))
  if (history.length > 60) history.shift()
  canUndo.value = true
}

function undo() {
  if (!history.length) return
  annos.value = history.pop() as Anno[]
  selectedId.value = null
  canUndo.value = history.length > 0
}

function clearAnnos() {
  if (!annos.value.length) return
  pushUndo()
  annos.value = []
  selectedId.value = null
}

function setTool(id: Tool) {
  if (editing.value) commitText()
  tool.value = id
}

function deleteSelected() {
  if (selectedId.value === null) return
  pushUndo()
  annos.value = annos.value.filter(a => a.id !== selectedId.value)
  selectedId.value = null
}

function requestDraw() {
  if (rafId) return
  rafId = requestAnimationFrame(() => {
    rafId = 0
    draw()
  })
}

// ---------- 坐标换算（显示像素 <-> 图片本地坐标） ----------
function toImagePt(e: PointerEvent): Pt {
  const cv = canvas.value as HTMLCanvasElement
  const rect = cv.getBoundingClientRect()
  const x = ((e.clientX - rect.left) / rect.width) * (cv.width / drawScale) - padding.value
  const y = ((e.clientY - rect.top) / rect.height) * (cv.height / drawScale) - padding.value
  return { x: Math.min(Math.max(0, x), imageW - 1), y: Math.min(Math.max(0, y), imageH - 1) }
}

// ---------- 命中测试与包围盒 ----------
function distToSeg(px: number, py: number, x1: number, y1: number, x2: number, y2: number) {
  const dx = x2 - x1
  const dy = y2 - y1
  const lenSq = dx * dx + dy * dy
  const t = lenSq === 0 ? 0 : Math.min(1, Math.max(0, ((px - x1) * dx + (py - y1) * dy) / lenSq))
  return Math.hypot(px - (x1 + t * dx), py - (y1 + t * dy))
}

function annoBBox(a: Anno): { x: number; y: number; w: number; h: number } {
  if (a.kind === 'text') {
    return {
      x: a.x - 4,
      y: a.y - a.size * 0.95,
      w: measureTextWidth(a.text, a.size) + 8,
      h: a.size * 1.18,
    }
  }
  if (a.kind === 'line' || a.kind === 'arrow') {
    const pad = a.width / 2
    return {
      x: Math.min(a.x1, a.x2) - pad,
      y: Math.min(a.y1, a.y2) - pad,
      w: Math.abs(a.x2 - a.x1) + pad * 2,
      h: Math.abs(a.y2 - a.y1) + pad * 2,
    }
  }
  if (a.kind === 'rect') {
    return {
      x: Math.min(a.x, a.x + a.w),
      y: Math.min(a.y, a.y + a.h),
      w: Math.abs(a.w),
      h: Math.abs(a.h),
    }
  }
  if (a.kind === 'pen') {
    let minX = Infinity
    let minY = Infinity
    let maxX = -Infinity
    let maxY = -Infinity
    for (const p of a.points) {
      minX = Math.min(minX, p.x)
      minY = Math.min(minY, p.y)
      maxX = Math.max(maxX, p.x)
      maxY = Math.max(maxY, p.y)
    }
    const pad = a.width / 2 + 1
    return { x: minX - pad, y: minY - pad, w: maxX - minX + pad * 2, h: maxY - minY + pad * 2 }
  }
  return { x: 0, y: 0, w: 0, h: 0 }
}

function hitTest(pt: Pt): Anno | null {
  for (let i = annos.value.length - 1; i >= 0; i--) {
    const a = annos.value[i]
    if (a.kind === 'text') {
      if (!a.text) continue
      const box = annoBBox(a)
      if (pt.x >= box.x && pt.x <= box.x + box.w && pt.y >= box.y && pt.y <= box.y + box.h) return a
    } else if (a.kind === 'line' || a.kind === 'arrow') {
      if (distToSeg(pt.x, pt.y, a.x1, a.y1, a.x2, a.y2) <= a.width / 2 + 6) return a
    } else if (a.kind === 'rect') {
      const box = annoBBox(a)
      const pad = a.width / 2 + 5
      if (
        pt.x >= box.x - pad &&
        pt.x <= box.x + box.w + pad &&
        pt.y >= box.y - pad &&
        pt.y <= box.y + box.h + pad
      )
        return a
    } else if (a.kind === 'pen') {
      const pts = a.points
      for (let j = 1; j < pts.length; j++) {
        if (
          distToSeg(pt.x, pt.y, pts[j - 1].x, pts[j - 1].y, pts[j].x, pts[j].y) <=
          a.width / 2 + 6
        )
          return a
      }
    }
  }
  return null
}

function restoreTranslated(target: Anno, orig: Anno, dx: number, dy: number) {
  if (target.kind === 'text' && orig.kind === 'text') {
    target.x = orig.x + dx
    target.y = orig.y + dy
  } else if (
    (target.kind === 'line' || target.kind === 'arrow') &&
    (orig.kind === 'line' || orig.kind === 'arrow')
  ) {
    target.x1 = orig.x1 + dx
    target.y1 = orig.y1 + dy
    target.x2 = orig.x2 + dx
    target.y2 = orig.y2 + dy
  } else if (target.kind === 'rect' && orig.kind === 'rect') {
    target.x = orig.x + dx
    target.y = orig.y + dy
  } else if (target.kind === 'pen' && orig.kind === 'pen') {
    target.points = orig.points.map(p => ({ x: p.x + dx, y: p.y + dy }))
  }
}

// ---------- 文字编辑浮层 ----------
function editingAnno(): TextAnno | null {
  if (!editing.value) return null
  if (creatingText) return pendingText
  const a = annos.value.find(item => item.id === editId.value)
  return a && a.kind === 'text' ? a : null
}

function openTextEditor(a: TextAnno, isNew: boolean) {
  gesture = null
  creatingText = isNew
  pendingText = isNew ? a : null
  editId.value = a.id
  editValue.value = a.text
  editColor.value = a.color
  editSnapshot = cloneAnnos(annos.value)
  editing.value = true
  selectedId.value = a.id
  nextTick(() => {
    syncEditorPos()
    editInput.value?.focus()
  })
}

function syncEditorPos() {
  const a = editingAnno()
  const cv = canvas.value
  const vp = viewport.value
  if (!a || !cv || !vp || !cv.width) return
  const rect = cv.getBoundingClientRect()
  const vpRect = vp.getBoundingClientRect()
  const k = rect.width / cv.width
  const fontSize = Math.max(11, Math.round(a.size * drawScale * k))
  editPos.value = {
    left: rect.left - vpRect.left + (padding.value + a.x) * drawScale * k,
    top:
      rect.top -
      vpRect.top +
      (padding.value + a.y) * drawScale * (rect.height / cv.height) -
      fontSize -
      6,
    fontSize,
    width: Math.max(
      150,
      measureTextWidth(editValue.value || 'M'.repeat(10), a.size) * drawScale * k + 30
    ),
  }
}

function commitText() {
  if (!editing.value) return
  const value = editValue.value
  if (creatingText) {
    if (value.trim() && pendingText) {
      annos.value.push({ ...pendingText, text: value })
      if (editSnapshot) pushUndo(editSnapshot)
    }
  } else {
    const a = annos.value.find(item => item.id === editId.value)
    if (a && a.kind === 'text') {
      if (!value.trim()) {
        pushUndo(editSnapshot ?? undefined)
        annos.value = annos.value.filter(item => item.id !== a.id)
        if (selectedId.value === a.id) selectedId.value = null
      } else if (a.text !== value) {
        pushUndo(editSnapshot ?? undefined)
        a.text = value
      }
    }
  }
  editing.value = false
  creatingText = false
  pendingText = null
  editSnapshot = null
  requestDraw()
}

// ---------- 指针交互 ----------
function onPointerDown(e: PointerEvent) {
  // 阻止 mousedown 默认抢焦点，否则刚打开的文字输入框会立刻被 blur 提交关闭
  e.preventDefault()
  if (!image || e.button !== 0) return
  if (editing.value) commitText()
  const cv = canvas.value as HTMLCanvasElement
  try {
    cv.setPointerCapture(e.pointerId)
  } catch {
    // 指针已失效（如合成事件）时忽略，元素级监听仍可工作
  }
  const pt = toImagePt(e)
  if (tool.value === 'text') {
    const hit = hitTest(pt)
    if (hit && hit.kind === 'text') {
      openTextEditor(hit, false)
      return
    }
    openTextEditor(
      {
        id: nextAnnoId++,
        kind: 'text',
        x: pt.x,
        y: pt.y,
        size: textSize.value,
        color: annoColor.value,
        width: annoWidth.value,
        text: '',
      },
      true
    )
    return
  }
  if (tool.value === 'select') {
    const hit = hitTest(pt)
    selectedId.value = hit ? hit.id : null
    if (hit)
      gesture = {
        kind: 'move',
        id: hit.id,
        orig: cloneAnnos([hit])[0],
        start: pt,
        snapshot: null,
        isText: hit.kind === 'text',
      }
    requestDraw()
    return
  }
  const base = { id: nextAnnoId++, color: annoColor.value, width: annoWidth.value }
  let anno: DrawAnno
  if (tool.value === 'rect') anno = { ...base, kind: 'rect', x: pt.x, y: pt.y, w: 0, h: 0 }
  else if (tool.value === 'pen') anno = { ...base, kind: 'pen', points: [pt] }
  else anno = { ...base, kind: tool.value, x1: pt.x, y1: pt.y, x2: pt.x, y2: pt.y }
  gesture = { kind: 'draw', anno, snapshot: cloneAnnos(annos.value), significant: false }
}

function onPointerMove(e: PointerEvent) {
  const g = gesture
  if (!g || !image) return
  const pt = toImagePt(e)
  if (g.kind === 'draw') {
    const a = g.anno
    if (a.kind === 'rect') {
      a.w = pt.x - a.x
      a.h = pt.y - a.y
      g.significant ||= Math.abs(a.w) > 3 && Math.abs(a.h) > 3
    } else if (a.kind === 'pen') {
      const last = a.points[a.points.length - 1]
      if (Math.hypot(pt.x - last.x, pt.y - last.y) > 1.5) a.points.push(pt)
      g.significant ||= a.points.length > 2
    } else {
      a.x2 = pt.x
      a.y2 = pt.y
      g.significant ||= Math.hypot(a.x2 - a.x1, a.y2 - a.y1) > 3
    }
  } else {
    const dx = pt.x - g.start.x
    const dy = pt.y - g.start.y
    if (!g.snapshot && (Math.abs(dx) > 2 || Math.abs(dy) > 2)) g.snapshot = cloneAnnos(annos.value)
    if (g.snapshot) {
      const target = annos.value.find(item => item.id === g.id)
      if (target) restoreTranslated(target, g.orig, dx, dy)
    }
  }
  requestDraw()
}

function onPointerUp(e: PointerEvent) {
  const cv = canvas.value
  if (cv?.hasPointerCapture(e.pointerId)) cv.releasePointerCapture(e.pointerId)
  const g = gesture
  gesture = null
  if (!g) return
  if (g.kind === 'draw') {
    if (!g.significant) return
    const a = g.anno
    let ok = false
    if (a.kind === 'rect') ok = Math.abs(a.w) > 3 && Math.abs(a.h) > 3
    else if (a.kind === 'pen') ok = a.points.length >= 1
    else ok = Math.hypot(a.x2 - a.x1, a.y2 - a.y1) > 3
    if (ok) {
      annos.value.push(a)
      pushUndo(g.snapshot)
      selectedId.value = a.id
    }
  } else {
    const target = annos.value.find(item => item.id === g.id)
    if (g.snapshot) {
      if (target) pushUndo(g.snapshot)
    } else if (g.isText && target && target.kind === 'text') {
      openTextEditor(target, false)
    }
  }
  requestDraw()
}

function onKeydown(e: KeyboardEvent) {
  const target = e.target as HTMLElement | null
  const typing =
    !!target &&
    (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)
  if (e.key === 'Escape' && !typing) {
    if (editing.value) commitText()
    selectedId.value = null
    tool.value = 'select'
    return
  }
  if (typing) return
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') {
    e.preventDefault()
    undo()
  } else if (
    (e.key === 'Delete' || e.key === 'Backspace') &&
    selectedId.value !== null &&
    !editing.value
  ) {
    e.preventDefault()
    deleteSelected()
  }
}

// ---------- 渲染 ----------
function renderAnnotations(ctx: CanvasRenderingContext2D) {
  const list = gesture && gesture.kind === 'draw' ? [...annos.value, gesture.anno] : annos.value
  for (const a of list) {
    ctx.save()
    ctx.lineCap = 'round'
    ctx.lineJoin = 'round'
    ctx.strokeStyle = a.color
    ctx.fillStyle = a.color
    ctx.lineWidth = a.width
    if (a.kind === 'text') {
      if (a.text) {
        ctx.font = annoFont(a.size)
        ctx.textBaseline = 'alphabetic'
        ctx.strokeStyle = contrastOutline(a.color)
        ctx.lineWidth = Math.max(2, a.size / 9)
        ctx.strokeText(a.text, a.x, a.y)
        ctx.fillStyle = a.color
        ctx.fillText(a.text, a.x, a.y)
      }
    } else if (a.kind === 'line' || a.kind === 'arrow') {
      const len = Math.hypot(a.x2 - a.x1, a.y2 - a.y1)
      if (a.kind === 'arrow' && len > 1) {
        const head = Math.max(a.width * 3.2, 10)
        const angle = Math.atan2(a.y2 - a.y1, a.x2 - a.x1)
        const ux = Math.cos(angle)
        const uy = Math.sin(angle)
        ctx.beginPath()
        ctx.moveTo(a.x1, a.y1)
        ctx.lineTo(a.x2 - ux * head * 0.72, a.y2 - uy * head * 0.72)
        ctx.stroke()
        ctx.beginPath()
        ctx.moveTo(a.x2, a.y2)
        ctx.lineTo(a.x2 - head * Math.cos(angle - 0.42), a.y2 - head * Math.sin(angle - 0.42))
        ctx.lineTo(a.x2 - head * Math.cos(angle + 0.42), a.y2 - head * Math.sin(angle + 0.42))
        ctx.closePath()
        ctx.fill()
      } else {
        ctx.beginPath()
        ctx.moveTo(a.x1, a.y1)
        ctx.lineTo(a.x2, a.y2)
        ctx.stroke()
      }
    } else if (a.kind === 'rect') {
      const w = Math.abs(a.w)
      const h = Math.abs(a.h)
      if (w > 1 && h > 1) ctx.strokeRect(Math.min(a.x, a.x + a.w), Math.min(a.y, a.y + a.h), w, h)
    } else if (a.kind === 'pen') {
      if (a.points.length === 1) {
        const p = a.points[0]
        ctx.beginPath()
        ctx.arc(p.x, p.y, a.width / 2, 0, Math.PI * 2)
        ctx.fill()
      } else if (a.points.length > 1) {
        ctx.beginPath()
        ctx.moveTo(a.points[0].x, a.points[0].y)
        for (let i = 1; i < a.points.length; i++) ctx.lineTo(a.points[i].x, a.points[i].y)
        ctx.stroke()
      }
    }
    ctx.restore()
  }
  if (
    selectedId.value !== null &&
    tool.value === 'select' &&
    !(editing.value && editId.value === selectedId.value)
  ) {
    const a = annos.value.find(item => item.id === selectedId.value)
    if (a) {
      const box = annoBBox(a)
      ctx.save()
      ctx.setLineDash([5, 4])
      ctx.lineWidth = 1.5
      ctx.strokeStyle = 'rgba(96, 132, 255, 0.95)'
      ctx.strokeRect(box.x - 5, box.y - 5, box.w + 10, box.h + 10)
      ctx.restore()
    }
  }
}

function draw() {
  const target = canvas.value
  if (!target || !image) return
  const ctx = target.getContext('2d')
  if (!ctx) return
  const w = image.naturalWidth
  const h = image.naturalHeight
  // 限制画布尺寸和内存；一般截图按原始像素导出。
  const scale = Math.min(1, 6000 / Math.max(w + padding.value * 2, h + padding.value * 2))
  drawScale = scale
  imageW = w
  imageH = h
  target.width = Math.round((w + padding.value * 2) * scale)
  target.height = Math.round((h + padding.value * 2) * scale)
  exportWidth.value = target.width
  exportHeight.value = target.height
  ctx.scale(scale, scale)
  const gradient = ctx.createLinearGradient(0, 0, w + padding.value * 2, h + padding.value * 2)
  gradient.addColorStop(0, from.value)
  gradient.addColorStop(1, to.value)
  ctx.fillStyle = gradient
  ctx.fillRect(0, 0, target.width / scale, target.height / scale)
  const r = Math.min(radius.value, w / 2, h / 2)
  ctx.save()
  ctx.beginPath()
  ctx.roundRect(padding.value, padding.value, w, h, r)
  ctx.shadowColor = 'rgba(0, 0, 0, 0.35)'
  ctx.shadowBlur = shadow.value
  ctx.shadowOffsetY = shadow.value / 3
  ctx.fillStyle = '#fff'
  ctx.fill()
  ctx.restore()
  ctx.save()
  ctx.beginPath()
  ctx.roundRect(padding.value, padding.value, w, h, r)
  ctx.clip()
  ctx.drawImage(image, padding.value, padding.value, w, h)
  ctx.translate(padding.value, padding.value)
  renderAnnotations(ctx)
  ctx.restore()
  if (editing.value) syncEditorPos()
  ready.value = true
}

// ---------- 草稿 ----------
function draftKey(path: string) {
  return `beautify-draft:${path}`
}

function draftNumber(value: unknown, min: number, max: number, fallback: number) {
  const n = typeof value === 'number' ? value : Number(value)
  return Number.isFinite(n) ? Math.min(max, Math.max(min, n)) : fallback
}

function draftCoord(value: unknown) {
  const n = Number(value)
  return Number.isFinite(n) ? n : 0
}

function sanitizeAnnos(value: unknown): Anno[] {
  if (!Array.isArray(value)) return []
  const out: Anno[] = []
  for (const raw of value) {
    if (!raw || typeof raw !== 'object') continue
    const r = raw as Record<string, unknown>
    const base = {
      id: nextAnnoId++,
      color: typeof r.color === 'string' && r.color ? r.color : ANNO_COLORS[0],
      width: draftNumber(r.width, 1, 40, 4),
    }
    if (r.kind === 'text') {
      out.push({
        ...base,
        kind: 'text',
        x: draftCoord(r.x),
        y: draftCoord(r.y),
        size: draftNumber(r.size, 12, 120, 28),
        text: typeof r.text === 'string' ? r.text : '',
      })
    } else if (r.kind === 'line' || r.kind === 'arrow') {
      out.push({
        ...base,
        kind: r.kind,
        x1: draftCoord(r.x1),
        y1: draftCoord(r.y1),
        x2: draftCoord(r.x2),
        y2: draftCoord(r.y2),
      })
    } else if (r.kind === 'rect') {
      out.push({
        ...base,
        kind: 'rect',
        x: draftCoord(r.x),
        y: draftCoord(r.y),
        w: draftCoord(r.w),
        h: draftCoord(r.h),
      })
    } else if (r.kind === 'pen' && Array.isArray(r.points)) {
      const points = r.points
        .filter(p => p && typeof p === 'object')
        .map(p => ({
          x: draftCoord((p as Record<string, unknown>).x),
          y: draftCoord((p as Record<string, unknown>).y),
        }))
      if (points.length) out.push({ ...base, kind: 'pen', points })
    }
  }
  return out
}

function resetParams() {
  from.value = presets[0].from
  to.value = presets[0].to
  padding.value = 64
  radius.value = 16
  shadow.value = 32
}

function applyDraft(path: string) {
  let draft: unknown = null
  try {
    const raw = sessionStorage.getItem(draftKey(path))
    if (raw) draft = JSON.parse(raw)
  } catch {
    draft = null
  }
  if (!draft || typeof draft !== 'object') {
    resetParams()
    annos.value = []
    return
  }
  const saved = draft as Record<string, unknown>
  from.value = typeof saved.from === 'string' && saved.from ? saved.from : presets[0].from
  to.value = typeof saved.to === 'string' && saved.to ? saved.to : presets[0].to
  padding.value = draftNumber(saved.padding, 0, 160, 64)
  radius.value = draftNumber(saved.radius, 0, 64, 16)
  shadow.value = draftNumber(saved.shadow, 0, 80, 32)
  annos.value = sanitizeAnnos(saved.annos)
}

function writeDraft(path: string) {
  try {
    sessionStorage.setItem(
      draftKey(path),
      JSON.stringify({
        from: from.value,
        to: to.value,
        padding: padding.value,
        radius: radius.value,
        shadow: shadow.value,
        annos: annos.value,
      })
    )
  } catch {
    // 草稿写入失败可忽略（不影响编辑与导出）
  }
}

function cancelDraft() {
  if (draftTimer !== undefined) {
    clearTimeout(draftTimer)
    draftTimer = undefined
  }
}

function flushDraft() {
  if (draftTimer === undefined) return
  cancelDraft()
  writeDraft(draftTimerPath)
}

// ---------- 加载与保存 ----------
async function load() {
  const version = ++loadVersion
  ready.value = false
  image = null
  error.value = ''
  gesture = null
  editing.value = false
  creatingText = false
  pendingText = null
  editSnapshot = null
  selectedId.value = null
  history = []
  canUndo.value = false
  flushDraft()
  applyDraft(props.path)
  const next = new Image()
  try {
    next.src = await readImageDataUrl(props.path)
    await next.decode()
    if (version !== loadVersion) return
    image = next
    draw()
  } catch (err) {
    error.value = t('screenshot.loadFailed', { message: String(err) })
  }
}

async function save() {
  if (!canvas.value || !ready.value || saving.value) return
  if (editing.value) commitText()
  saving.value = true
  error.value = ''
  try {
    const savedPath = await saveStyledScreenshot(canvas.value.toDataURL('image/png'))
    cancelDraft()
    try {
      sessionStorage.removeItem(draftKey(props.path))
    } catch {
      // 草稿清理失败可忽略
    }
    emit('saved', savedPath)
  } catch (err) {
    error.value = t('screenshot.captureFailed', { message: String(err) })
  } finally {
    saving.value = false
  }
}

watch(() => props.path, load)
watch(
  [from, to, padding, radius, shadow, annos],
  () => {
    requestDraw()
    cancelDraft()
    draftTimerPath = props.path
    draftTimer = window.setTimeout(() => {
      draftTimer = undefined
      writeDraft(draftTimerPath)
    }, 300)
  },
  { deep: true }
)
watch(editValue, () => {
  if (editing.value) syncEditorPos()
})

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  if (canvas.value && typeof ResizeObserver !== 'undefined') {
    resizeObserver = new ResizeObserver(() => {
      if (editing.value) syncEditorPos()
    })
    resizeObserver.observe(canvas.value)
  }
  load()
})

onUnmounted(() => {
  if (rafId) cancelAnimationFrame(rafId)
  resizeObserver?.disconnect()
  window.removeEventListener('keydown', onKeydown)
  flushDraft()
})
</script>

<style scoped>
.beautify-editor { min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) 328px; overflow: hidden; }
.canvas-stage { min-width: 0; min-height: 0; display: grid; grid-template-rows: auto auto minmax(0, 1fr); padding: 14px; border-right: 1px solid var(--border); background:
    radial-gradient(circle at 50% 36%, rgb(var(--accent-rgb) / 0.09), transparent 32rem),
    var(--surface-canvas); }
.stage-bar { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 2px 4px 10px; color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.stage-bar > span:first-child { display: flex; align-items: center; gap: 7px; }
.live-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--success); box-shadow: 0 0 0 4px rgb(var(--success-rgb) / 0.12); }
.anno-bar { display: flex; align-items: center; flex-wrap: wrap; gap: 8px 10px; margin-bottom: 12px; padding: 7px 10px; border: 1px solid var(--border); border-radius: 11px; background: var(--surface-canvas); }
.anno-group { display: flex; align-items: center; gap: 3px; }
.tool-btn { display: grid; width: 30px; height: 30px; place-items: center; padding: 0; border: 1px solid transparent; border-radius: 8px; background: transparent; color: var(--text-muted); cursor: pointer; transition: border-color var(--motion-fast), background var(--motion-fast), color var(--motion-fast); }
.tool-btn:hover:not(:disabled) { background: rgb(var(--text-rgb) / 0.06); color: var(--text); }
.tool-btn.active { border-color: rgb(var(--accent-rgb) / 0.5); background: rgb(var(--accent-rgb) / 0.12); color: var(--accent); }
.tool-btn:disabled { opacity: 0.4; cursor: default; }
.tool-btn:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
.anno-divider { width: 1px; height: 20px; background: var(--border); }
.anno-spacer { flex: 1; }
.color-dot { width: 20px; height: 20px; padding: 0; border: 2px solid transparent; border-radius: 50%; box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.28); cursor: pointer; transition: border-color var(--motion-fast), transform var(--motion-fast); }
.color-dot:hover { transform: scale(1.1); }
.color-dot.active { border-color: var(--text); transform: scale(1.08); }
.color-dot:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.width-ctrl { display: flex; align-items: center; gap: 7px; color: var(--text-muted); font-size: 10px; }
.width-ctrl input { width: 78px; margin: 0; accent-color: var(--accent); cursor: pointer; }
.canvas-viewport { position: relative; min-width: 0; min-height: 0; display: grid; place-items: center; overflow: auto; padding: clamp(18px, 3vw, 38px); border: 1px solid rgb(var(--text-rgb) / 0.07); border-radius: 13px; background-image: radial-gradient(rgb(var(--text-rgb) / 0.075) 0.7px, transparent 0.7px); background-size: 12px 12px; }
canvas { max-width: 100%; max-height: 64vh; object-fit: contain; box-shadow: 0 24px 70px rgb(var(--ink-rgb) / 0.24); touch-action: none; }
.cursor-text { cursor: text; }
.cursor-crosshair { cursor: crosshair; }
.anno-editor { position: absolute; z-index: 6; min-width: 150px; padding: 3px 8px; border: 1.5px solid var(--accent); border-radius: 7px; background: rgb(var(--bg-rgb) / 0.93); font-family: "Segoe UI", "Microsoft YaHei", "PingFang SC", system-ui, sans-serif; font-weight: 600; line-height: 1.25; box-shadow: 0 8px 26px rgb(var(--ink-rgb) / 0.3); }
.anno-editor:focus-visible { outline: none; }
.controls { min-width: 0; min-height: 0; display: flex; flex-direction: column; background: var(--bg); }
.controls-scroll { min-height: 0; flex: 1; overflow: auto; padding: 22px; }
.controls-header { display: flex; align-items: flex-start; gap: 12px; padding-bottom: 20px; }
.controls-icon { display: grid; width: 36px; height: 36px; flex: 0 0 auto; place-items: center; border: 1px solid rgb(var(--accent-rgb) / 0.22); border-radius: 10px; background: rgb(var(--accent-rgb) / 0.1); color: var(--accent); }
h3, p { margin: 0; }
h3 { font-size: 15px; font-weight: 650; }
p { margin-top: 5px; color: var(--text-muted); font-size: 11px; line-height: 1.6; }
.control-section { padding: 18px 0; border-top: 1px solid var(--border); }
.section-heading { min-height: 28px; display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 13px; color: var(--text); font-size: 11px; font-weight: 650; }
.swatches { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
.swatches button { min-width: 0; padding: 4px; border: 1px solid transparent; border-radius: 9px; background: transparent; color: var(--text-muted); cursor: pointer; transition: border-color var(--motion-fast), background var(--motion-fast), color var(--motion-fast); }
.swatches button > span { display: block; aspect-ratio: 1.25; border: 1px solid rgb(var(--text-rgb) / 0.12); border-radius: 7px; box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.08); }
.swatches button small { display: block; margin-top: 6px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 9px; }
.swatches button:hover { background: rgb(var(--text-rgb) / 0.045); color: var(--text); }
.swatches button[aria-pressed="true"] { border-color: rgb(var(--accent-rgb) / 0.5); background: rgb(var(--accent-rgb) / 0.1); color: var(--accent); }
button:focus-visible, input:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.color-row { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 14px; }
.color-row label { display: flex; flex-direction: column; gap: 7px; color: var(--text-muted); font-size: 10px; }
.color-field { display: flex; align-items: center; gap: 7px; min-width: 0; padding: 5px 7px; border: 1px solid var(--border); border-radius: 8px; background: var(--surface-canvas); }
input[type="color"] { width: 26px; height: 24px; flex: 0 0 auto; padding: 0; border: 0; border-radius: 5px; background: transparent; cursor: pointer; }
.color-field code { min-width: 0; overflow: hidden; color: var(--text-muted); font: 9px var(--jedi-font-mono); text-overflow: ellipsis; }
.geometry-section { display: flex; flex-direction: column; gap: 16px; }
.geometry-section .section-heading { margin-bottom: -2px; }
.slider-label { display: grid; grid-template-columns: 1fr auto; align-items: center; gap: 8px 12px; color: var(--text-muted); font-size: 11px; }
.slider-label > input[type="range"] { grid-column: 1 / -1; width: 100%; margin: 0; accent-color: var(--accent); cursor: pointer; }
.value-input { display: flex; align-items: center; gap: 2px; padding: 2px 6px; border: 1px solid var(--border); border-radius: 7px; background: var(--surface-canvas); }
.value-input small { color: var(--text-muted); font: 9px var(--jedi-font-mono); }
.num-input { width: 38px; padding: 2px 0; border: 0; outline: 0; background: transparent; color: var(--text); font: 10px var(--jedi-font-mono); text-align: right; }
.reset-btn { display: flex; align-items: center; gap: 5px; padding: 4px 7px; border: 0; border-radius: 6px; background: transparent; color: var(--text-muted); font: 10px var(--jedi-font-ui); cursor: pointer; }
.reset-btn:hover { background: rgb(var(--text-rgb) / 0.06); color: var(--text); }
.export-panel { display: flex; flex-direction: column; gap: 13px; padding: 16px 20px 18px; border-top: 1px solid var(--border); background: rgb(var(--text-rgb) / 0.018); }
.export-info { display: flex; flex-direction: column; gap: 3px; }
.export-info strong { font: 600 10px var(--jedi-font-mono); }
.export-info span { color: var(--text-muted); font-size: 10px; }
@media (max-width: 900px) {
  .beautify-editor { grid-template-columns: 1fr; overflow: auto; }
  .canvas-stage { min-height: 360px; border-right: 0; border-bottom: 1px solid var(--border); }
  canvas { max-height: 48vh; }
  .controls { min-height: 430px; }
}
@media (prefers-reduced-motion: reduce) { .swatches button, .tool-btn, .color-dot { transition: none; } }
</style>
