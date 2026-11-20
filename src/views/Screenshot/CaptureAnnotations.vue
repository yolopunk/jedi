<template>
  <canvas
    ref="canvas"
    class="annotation-layer"
    :class="{ drawing: tool && !disabled }"
    :width="crop.width"
    :height="crop.height"
    aria-hidden="true"
    @pointerdown.stop="start"
    @pointermove.stop="move"
    @pointerup.stop="end"
    @pointercancel="abortStroke"
    @lostpointercapture="abortStroke"
  />
  <textarea
    v-if="textLayout"
    ref="textInput"
    v-model="textValue"
    class="capture-text"
    :style="textLayout.style"
    :aria-label="t('screenshot.toolText')"
    :placeholder="t('screenshot.textPlaceholder')"
    rows="2"
    wrap="off"
    maxlength="1000"
    spellcheck="false"
    @pointerdown.stop
    @contextmenu.stop
    @keydown="onTextKey"
    @blur="commitText"
  ></textarea>
</template>

<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, shallowRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { PhysicalRect } from '@/api/screenshot'
import {
  type Annotation,
  type AnnotationTool,
  drawAnnotations,
  MOSAIC_BLOCK,
  type Point,
  type Stroke,
  TEXT_FONT_FAMILY,
  TEXT_LINE_HEIGHT,
} from './capture-annotations'

const props = defineProps<{
  crop: PhysicalRect
  frame: HTMLImageElement | null
  tool: AnnotationTool | null
  color: string
  width: number
  textSize: number
  disabled: boolean
}>()
const emit = defineEmits<{ activate: [] }>()

const { t } = useI18n()
const canvas = ref<HTMLCanvasElement | null>(null)
const textInput = ref<HTMLTextAreaElement | null>(null)
const textValue = ref('')
const textDraft = shallowRef<{
  point: Point
  bounds: DOMRect
  color: string
  size: number
} | null>(null)
const textLayout = computed(() => {
  const draft = textDraft.value
  if (!draft) return null
  const { bounds, point, color, size } = draft
  const crop = props.crop
  const scaleX = crop.width / bounds.width
  const scaleY = crop.height / bounds.height
  const lines = textValue.value.split('\n')
  const font = `600 ${size}px ${TEXT_FONT_FAMILY}`
  const ctx = canvas.value?.getContext('2d')
  let textWidth = 0
  if (ctx) {
    ctx.save()
    ctx.font = font
    for (const line of lines) textWidth = Math.max(textWidth, ctx.measureText(line).width)
    ctx.restore()
  }
  const width = Math.min(bounds.width, Math.max(144, Math.ceil(textWidth) + 4))
  const height = Math.min(bounds.height, Math.max(2, lines.length) * size * TEXT_LINE_HEIGHT)
  // 编辑框和导出的文字共用贴边后的起点。
  const left = Math.max(0, Math.min((point.x - crop.x) / scaleX, bounds.width - width))
  const top = Math.max(0, Math.min((point.y - crop.y) / scaleY, bounds.height - height))
  return {
    point: { x: crop.x + left * scaleX, y: crop.y + top * scaleY },
    size: size * scaleY,
    color,
    style: {
      left: `${left}px`,
      top: `${top}px`,
      width: `${width}px`,
      height: `${height}px`,
      color,
      fontFamily: TEXT_FONT_FAMILY,
      fontSize: `${size}px`,
      lineHeight: TEXT_LINE_HEIGHT,
    },
  }
})
const marks = shallowRef<Annotation[]>([])
const undone = shallowRef<Annotation[]>([])
const gesture = shallowRef<{ pointer: number; stroke: Stroke } | null>(null)
const canUndo = computed(() => marks.value.length > 0)
const canRedo = computed(() => undone.value.length > 0)
const isDrawing = computed(() => !!gesture.value)
const hasEdits = computed(() => canUndo.value || !!textDraft.value || isDrawing.value)
let animation = 0
let mosaicSource: HTMLCanvasElement | undefined

function paint(ctx: CanvasRenderingContext2D, preview: boolean) {
  ctx.save()
  ctx.translate(-props.crop.x, -props.crop.y)
  const activeStroke = gesture.value?.stroke
  const annotations = preview && activeStroke ? [...marks.value, activeStroke] : marks.value
  if (props.frame && !mosaicSource && annotations.some(mark => mark.tool === 'mosaic')) {
    mosaicSource = document.createElement('canvas')
    mosaicSource.width = Math.ceil(props.frame.naturalWidth / MOSAIC_BLOCK)
    mosaicSource.height = Math.ceil(props.frame.naturalHeight / MOSAIC_BLOCK)
    mosaicSource
      .getContext('2d')
      ?.drawImage(
        props.frame,
        0,
        0,
        props.frame.naturalWidth / MOSAIC_BLOCK,
        props.frame.naturalHeight / MOSAIC_BLOCK
      )
  }
  drawAnnotations(ctx, annotations, mosaicSource)
  ctx.restore()
}

function redraw() {
  if (animation) return
  animation = requestAnimationFrame(() => {
    animation = 0
    const layer = canvas.value
    const ctx = layer?.getContext('2d')
    if (!ctx || !layer) return
    ctx.clearRect(0, 0, layer.width, layer.height)
    paint(ctx, true)
  })
}

function add(mark: Annotation) {
  marks.value = [...marks.value, mark]
  undone.value = []
  redraw()
}

function undo() {
  commitText()
  const mark = marks.value[marks.value.length - 1]
  if (!mark) return
  marks.value = marks.value.slice(0, -1)
  undone.value = [...undone.value, mark]
  redraw()
}

function redo() {
  const mark = undone.value[undone.value.length - 1]
  if (!mark) return
  marks.value = [...marks.value, mark]
  undone.value = undone.value.slice(0, -1)
  redraw()
}

function pointAt(e: PointerEvent, bounds: DOMRect): Point {
  const { x, y, width, height } = props.crop
  return {
    x: x + Math.min(Math.max(0, ((e.clientX - bounds.left) * width) / bounds.width), width),
    y: y + Math.min(Math.max(0, ((e.clientY - bounds.top) * height) / bounds.height), height),
  }
}

function start(e: PointerEvent) {
  const { tool, disabled, crop, color } = props
  const layer = canvas.value
  if (!layer || disabled || !tool || e.button !== 0 || gesture.value) return
  e.preventDefault()
  commitText()
  emit('activate')
  const bounds = layer.getBoundingClientRect()
  const point = pointAt(e, bounds)
  if (tool === 'text') {
    textDraft.value = {
      point,
      bounds,
      color,
      size: props.textSize,
    }
    nextTick(() => textInput.value?.focus({ preventScroll: true }))
    return
  }
  const scale = crop.width / bounds.width
  layer.setPointerCapture(e.pointerId)
  gesture.value = {
    pointer: e.pointerId,
    stroke: { tool, color, width: props.width * scale, start: point, end: point, points: [point] },
  }
  redraw()
}

function move(e: PointerEvent) {
  const layer = canvas.value
  const active = gesture.value
  if (!layer || active?.pointer !== e.pointerId) return
  const stroke = active.stroke
  const point = pointAt(e, layer.getBoundingClientRect())
  if (point.x === stroke.end.x && point.y === stroke.end.y) return
  stroke.end = point
  if (stroke.tool === 'pen' || stroke.tool === 'mosaic') stroke.points.push(point)
  redraw()
}

function end(e: PointerEvent) {
  const active = gesture.value
  if (active?.pointer !== e.pointerId) return
  move(e)
  const { stroke } = active
  gesture.value = null
  if (
    stroke.tool === 'pen' ||
    stroke.tool === 'mosaic' ||
    Math.hypot(stroke.end.x - stroke.start.x, stroke.end.y - stroke.start.y) >= 2
  ) {
    add(stroke)
  } else {
    redraw()
  }
}

function abortStroke(e?: PointerEvent) {
  const active = gesture.value
  if (!active || (e && e.pointerId !== active.pointer)) return false
  const { pointer } = active
  gesture.value = null
  if (canvas.value?.hasPointerCapture(pointer)) canvas.value.releasePointerCapture(pointer)
  redraw()
  return true
}

function commitText() {
  const layout = textLayout.value
  if (!layout) return
  const { point, color, size } = layout
  const text = textValue.value
  textDraft.value = null
  textValue.value = ''
  if (text.trim()) add({ tool: 'text', point, color, size, text })
}

function onTextKey(e: KeyboardEvent) {
  if (e.isComposing || e.keyCode === 229) return
  if (e.key === 'Escape') {
    e.preventDefault()
    e.stopPropagation()
    textDraft.value = null
    textValue.value = ''
    emit('activate')
  } else if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    e.stopPropagation()
    commitText()
    emit('activate')
  }
}

async function exportPng(): Promise<string | undefined> {
  commitText()
  if (!marks.value.length) return undefined
  const frame = props.frame
  const { x, y, width, height } = props.crop
  const output = document.createElement('canvas')
  output.width = width
  output.height = height
  const ctx = output.getContext('2d')
  if (!ctx || !frame) throw new Error('PNG canvas unavailable')
  ctx.drawImage(frame, x, y, width, height, 0, 0, width, height)
  paint(ctx, false)
  const blob = await new Promise<Blob>((resolve, reject) => {
    output.toBlob(
      value => (value ? resolve(value) : reject(new Error('PNG encoding failed'))),
      'image/png'
    )
  })
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = () => reject(reader.error)
    reader.readAsDataURL(blob)
  })
}

watch(() => props.crop, redraw, { flush: 'post' })
watch(
  () => props.frame,
  () => {
    mosaicSource = undefined
    redraw()
  }
)
watch(() => props.tool, commitText)
onUnmounted(() => cancelAnimationFrame(animation))

defineExpose({ canUndo, canRedo, hasEdits, isDrawing, undo, redo, abortStroke, exportPng })
</script>

<style scoped>
.annotation-layer {
  position: absolute;
  inset: 0;
  display: block;
  width: 100%;
  height: 100%;
  pointer-events: none;
  touch-action: none;
}

.annotation-layer.drawing {
  cursor: crosshair;
  pointer-events: auto;
}

.capture-text {
  position: absolute;
  box-sizing: border-box;
  z-index: 1;
  padding: 0;
  resize: none;
  border: 0;
  outline: 1px solid #69b8ff;
  border-radius: 5px;
  background: #0f1218;
  font-weight: 600;
  white-space: pre;
  scrollbar-width: none;
  user-select: text;
}

.capture-text::placeholder { color: #c0c9d5; }
</style>
