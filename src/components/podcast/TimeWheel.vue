<template>
  <div class="time-wheel-container">
    <svg
      :viewBox="`0 0 ${svgW} ${svgH}`"
      class="time-wheel-svg"
      ref="svgRef"
      @mousedown="onDragStart"
      @mousemove="onDragMove"
      @mouseup="onDragEnd"
      @mouseleave="onDragEnd"
      @touchstart.prevent="onTouchStart"
      @touchmove.prevent="onTouchMove"
      @touchend="onDragEnd"
    >
      <defs>
        <radialGradient id="wheel-glow" cx="50%" cy="80%" r="50%">
          <stop offset="0%" stop-color="rgba(var(--v-theme-primary), 0.1)" />
          <stop offset="100%" stop-color="transparent" />
        </radialGradient>
        <filter id="dot-glow">
          <feGaussianBlur stdDeviation="2.5" result="blur" />
          <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
        </filter>
      </defs>

      <rect width="100%" height="100%" fill="url(#wheel-glow)" />

      <!-- Arc track (background) -->
      <path
        :d="arcPath"
        fill="none"
        stroke="rgba(var(--v-theme-on-surface), 0.06)"
        stroke-width="3"
        stroke-linecap="round"
      />

      <!-- Progress arc -->
      <path
        v-if="duration > 0"
        :d="arcPath"
        fill="none"
        stroke="rgb(var(--v-theme-primary))"
        stroke-width="2.5"
        stroke-linecap="round"
        :stroke-dasharray="arcLength"
        :stroke-dashoffset="arcLength - (progressPercent / 100) * arcLength"
      />

      <!-- Tick marks (刻度) -->
      <g v-for="tick in tickMarks" :key="'tick-' + tick.time">
        <line
          :x1="tick.x1" :y1="tick.y1"
          :x2="tick.x2" :y2="tick.y2"
          :stroke="tick.major ? 'rgba(var(--v-theme-on-surface), 0.2)' : 'rgba(var(--v-theme-on-surface), 0.08)'"
          :stroke-width="tick.major ? 1.5 : 0.8"
        />
        <text
          v-if="tick.major"
          :x="tick.labelX" :y="tick.labelY"
          text-anchor="middle"
          class="tick-label"
        >{{ tick.label }}</text>
      </g>

      <!-- Chapter markers -->
      <g v-for="(m, i) in positionedMarkers" :key="'ch-' + i">
        <circle
          :cx="m.x" :cy="m.y"
          :r="Math.abs(currentTime - m.time) < 3 ? 6 : 4"
          :fill="Math.abs(currentTime - m.time) < 3 ? 'rgb(var(--v-theme-primary))' : 'rgba(var(--v-theme-on-surface), 0.6)'"
          :stroke="Math.abs(currentTime - m.time) < 3 ? 'rgb(var(--v-theme-primary))' : 'rgba(var(--v-theme-on-surface), 0.2)'"
          stroke-width="1.5"
          class="chapter-dot"
          :class="{ active: Math.abs(currentTime - m.time) < 3 }"
          @click.stop="$emit('seek', m.time)"
        />
        <text
          :x="m.labelX" :y="m.labelY"
          text-anchor="middle"
          class="chapter-label"
          :class="{ active: Math.abs(currentTime - m.time) < 3 }"
          @click.stop="$emit('seek', m.time)"
        >{{ m.label }}</text>
      </g>

      <!-- Drag preview dot -->
      <circle
        v-if="dragPreview.visible"
        :cx="dragPreview.x" :cy="dragPreview.y"
        r="6"
        fill="none"
        stroke="rgb(var(--v-theme-primary))"
        stroke-width="2"
        filter="url(#dot-glow)"
        class="drag-preview"
      />

      <!-- Current position indicator -->
      <circle
        v-if="duration > 0"
        :cx="positionDot.x" :cy="positionDot.y"
        r="5.5"
        fill="rgb(var(--v-theme-primary))"
        stroke="#fff"
        stroke-width="1.5"
        filter="url(#dot-glow)"
        class="position-dot"
      />

      <!-- End time labels -->
      <text :x="arcStart.x - 6" :y="arcStart.y + 14" text-anchor="end" class="end-label">
        {{ formatTime(0) }}
      </text>
      <text :x="arcEnd.x + 6" :y="arcEnd.y + 14" text-anchor="start" class="end-label">
        {{ formatTime(duration) }}
      </text>
    </svg>

    <!-- Center time display -->
    <div class="wheel-center-info">
      <div class="center-time">{{ dragPreview.visible ? formatTime(dragPreview.time) : formatTime(currentTime) }}</div>
      <div class="center-divider"></div>
      <div class="center-total">{{ formatTime(duration) }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { formatTime } from '@/utils/timestampParser'
import type { TimestampMarker } from '@/utils/timestampParser'

const props = defineProps<{
  duration: number
  currentTime: number
  markers: TimestampMarker[]
}>()

const emit = defineEmits<{
  (e: 'seek', time: number): void
}>()

const svgRef = ref<SVGElement | null>(null)

// SVG geometry
const svgW = 500
const svgH = 220
const cx = svgW / 2
const cy = 180
const rx = 210
const ry = 160
const angleStart = Math.PI // left
const angleEnd = 2 * Math.PI // right (clockwise through top)

// ── Angle / point helpers ──────────────────────────────

function angleForTime(t: number): number {
  if (!props.duration || props.duration <= 0) return angleStart
  return angleStart + Math.max(0, Math.min(1, t / props.duration)) * (angleEnd - angleStart)
}

function timeForAngle(theta: number): number {
  const frac = (theta - angleStart) / (angleEnd - angleStart)
  return Math.max(0, Math.min(props.duration, frac * props.duration))
}

function pointForAngle(theta: number) {
  return { x: cx + rx * Math.cos(theta), y: cy + ry * Math.sin(theta), theta }
}

function angleFromPoint(px: number, py: number): number {
  // Angle from center to point, clamped to our arc range
  let a = Math.atan2(py - cy, px - cx)
  // Normalize to [0, 2π)
  if (a < 0) a += 2 * Math.PI
  // Clamp to [π, 2π]
  return Math.max(angleStart, Math.min(angleEnd, a))
}

// ── Arc path ───────────────────────────────────────────

const arcStart = computed(() => pointForAngle(angleStart))
const arcEnd = computed(() => pointForAngle(angleEnd))

const arcPath = computed(() => {
  const s = arcStart.value
  const e = arcEnd.value
  return `M ${s.x.toFixed(1)} ${s.y.toFixed(1)} A ${rx} ${ry} 0 0 1 ${e.x.toFixed(1)} ${e.y.toFixed(1)}`
})

const arcLength = computed(() => {
  const a = Math.max(rx, ry)
  const b = Math.min(rx, ry)
  const h = (a - b) ** 2 / (a + b) ** 2
  return (Math.PI * (a + b) * (1 + (3 * h) / (10 + Math.sqrt(4 - 3 * h)))) / 2
})

const progressPercent = computed(() =>
  props.duration <= 0 ? 0 : Math.min((props.currentTime / props.duration) * 100, 100)
)

const positionDot = computed(() => pointForAngle(angleForTime(props.currentTime)))

// ── Tick marks (刻度) ──────────────────────────────────

interface TickData {
  time: number
  major: boolean
  label: string
  x1: number
  y1: number
  x2: number
  y2: number
  labelX: number
  labelY: number
}

const tickMarks = computed<TickData[]>(() => {
  const dur = props.duration
  if (!dur || dur <= 0) return []

  // Pick interval based on duration
  let interval: number
  if (dur <= 300)
    interval = 30 // ≤ 5 min → every 30s
  else if (dur <= 1800)
    interval = 60 // ≤ 30 min → every 1 min
  else if (dur <= 7200)
    interval = 300 // ≤ 2 h → every 5 min
  else interval = 600 // > 2 h → every 10 min

  const ticks: TickData[] = []
  for (let t = 0; t <= dur + 0.5; t += interval) {
    const pt = pointForAngle(angleForTime(t))
    // Tick line: outward from arc
    const dx = pt.x - cx
    const dy = pt.y - cy
    const dist = Math.sqrt(dx * dx + dy * dy)
    const nx = dist > 0 ? dx / dist : 0
    const ny = dist > 0 ? dy / dist : 0
    const major = t % (interval * 5) === 0 || t === 0 || Math.abs(t - dur) < 0.5
    const outLen = major ? 14 : 7
    const labelOff = major ? 20 : 0

    ticks.push({
      time: t,
      major,
      label: major ? formatTime(t) : '',
      x1: pt.x + nx * 2,
      y1: pt.y + ny * 2,
      x2: pt.x + nx * outLen,
      y2: pt.y + ny * outLen,
      labelX: pt.x + nx * labelOff,
      labelY: pt.y + ny * labelOff + (major ? 5 : 0),
    })
  }
  return ticks
})

// ── Chapter markers ────────────────────────────────────

const positionedMarkers = computed(() => {
  return props.markers.slice(0, 10).map(m => {
    const pt = pointForAngle(angleForTime(m.time))
    const dx = cx - pt.x
    const dy = cy - pt.y
    const dist = Math.sqrt(dx * dx + dy * dy)
    const nx = dist > 0 ? dx / dist : 0
    const ny = dist > 0 ? dy / dist : 0
    const labelOff = 22
    const shortLabel = m.label.length > 16 ? m.label.slice(0, 16) + '…' : m.label
    return {
      ...pt,
      time: m.time,
      label: shortLabel,
      labelX: pt.x + nx * labelOff,
      labelY: pt.y + ny * labelOff + 5,
    }
  })
})

// ── Drag interaction ───────────────────────────────────

const dragPreview = reactive({ visible: false, x: 0, y: 0, time: 0 })
let dragging = false

function clientToSvgAngle(clientX: number, clientY: number): number | null {
  const el = svgRef.value
  if (!el) return null
  const rect = el.getBoundingClientRect()
  // Map client coords -> SVG viewBox coords
  const svgX = ((clientX - rect.left) / rect.width) * svgW
  const svgY = ((clientY - rect.top) / rect.height) * svgH
  return angleFromPoint(svgX, svgY)
}

function onDragStart(e: MouseEvent) {
  dragging = true
  const a = clientToSvgAngle(e.clientX, e.clientY)
  if (a !== null) {
    dragPreview.visible = true
    const pt = pointForAngle(a)
    dragPreview.x = pt.x
    dragPreview.y = pt.y
    dragPreview.time = timeForAngle(a)
  }
}

function onDragMove(e: MouseEvent) {
  if (!dragging) return
  const a = clientToSvgAngle(e.clientX, e.clientY)
  if (a !== null) {
    const pt = pointForAngle(a)
    dragPreview.x = pt.x
    dragPreview.y = pt.y
    dragPreview.time = timeForAngle(a)
  }
}

function onDragEnd() {
  if (!dragging) return
  dragging = false
  if (dragPreview.visible) {
    emit('seek', dragPreview.time)
    dragPreview.visible = false
  }
}

// Touch events
function onTouchStart(e: TouchEvent) {
  const t = e.touches[0]
  dragging = true
  const a = clientToSvgAngle(t.clientX, t.clientY)
  if (a !== null) {
    dragPreview.visible = true
    const pt = pointForAngle(a)
    dragPreview.x = pt.x
    dragPreview.y = pt.y
    dragPreview.time = timeForAngle(a)
  }
}

function onTouchMove(e: TouchEvent) {
  if (!dragging) return
  const t = e.touches[0]
  const a = clientToSvgAngle(t.clientX, t.clientY)
  if (a !== null) {
    const pt = pointForAngle(a)
    dragPreview.x = pt.x
    dragPreview.y = pt.y
    dragPreview.time = timeForAngle(a)
  }
}
</script>

<style scoped>
.time-wheel-container {
  position: relative;
  width: 100%;
  background: rgba(var(--v-theme-surface), 0.95);
  border-radius: 16px 16px 0 0;
  overflow: hidden;
  border: 1px solid rgba(var(--v-theme-on-surface), 0.06);
  user-select: none;
}

.time-wheel-svg {
  width: 100%;
  height: auto;
  display: block;
  cursor: pointer;
}

/* Tick labels */
.tick-label {
  font-size: 8px;
  fill: rgba(var(--v-theme-on-surface), 0.22);
  font-family: 'JetBrains Mono', monospace;
  pointer-events: none;
}

/* Chapter dots & labels */
.chapter-dot {
  cursor: pointer;
  transition: r 0.15s, fill 0.15s;
}
.chapter-dot:hover { r: 6; fill: rgb(var(--v-theme-primary)); }
.chapter-dot.active { filter: url(#dot-glow); }

.chapter-label {
  font-size: 9px;
  fill: rgba(var(--v-theme-on-surface), 0.4);
  cursor: pointer;
  font-family: 'JetBrains Mono', monospace;
  transition: fill 0.15s;
}
.chapter-label:hover,
.chapter-label.active { fill: rgba(var(--v-theme-on-surface), 0.85); }

/* Position dot */
.position-dot { pointer-events: none; }

/* Drag preview */
.drag-preview { pointer-events: none; }

/* End labels */
.end-label {
  font-size: 9px;
  fill: rgba(var(--v-theme-on-surface), 0.25);
  font-family: 'JetBrains Mono', monospace;
}

/* Center time display */
.wheel-center-info {
  position: absolute;
  bottom: 12px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  pointer-events: none;
}
.center-time {
  font-size: 24px;
  font-weight: 700;
  font-family: 'JetBrains Mono', monospace;
  color: rgba(var(--v-theme-on-surface), 0.9);
  letter-spacing: 3px;
  transition: color 0.15s;
}
.center-divider {
  width: 36px;
  height: 1px;
  background: rgba(var(--v-theme-primary), 0.35);
}
.center-total {
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  color: rgba(var(--v-theme-on-surface), 0.35);
}
</style>
