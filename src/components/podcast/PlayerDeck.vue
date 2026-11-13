<template>
  <div class="deck">
    <!-- 头部：旋转卷盘 + 标题 + 时钟 -->
    <header class="deck-head">
      <span class="deck-reel" :class="{ spinning: !isPaused }" aria-hidden="true" />
      <span class="deck-title">{{ currentPlaying?.title || '' }}</span>
      <span class="deck-clock">
        {{ formatTime(displaySeconds) }}<span class="clock-sep">/</span>{{ formatTime(duration) }}
      </span>
    </header>

    <!-- 进度：章节刻度 + 缓冲 + 拖拽气泡 -->
    <div
      ref="scrubEl"
      class="deck-scrub"
      role="slider"
      tabindex="0"
      :aria-label="t('podcast.progress')"
      aria-valuemin="0"
      :aria-valuemax="Math.round(duration)"
      :aria-valuenow="Math.round(displaySeconds)"
      :aria-valuetext="formatTime(displaySeconds)"
      @pointerdown="onScrubDown"
      @pointermove="onScrubMove"
      @pointerup="onScrubUp"
      @pointercancel="onScrubUp"
      @pointerleave="hoverPercent = null"
      @keydown.left.prevent="seek(-5)"
      @keydown.right.prevent="seek(5)"
    >
      <div class="scrub-rail">
        <div class="scrub-buffered" :style="{ width: `${bufferedPercent}%` }" />
        <div class="scrub-played" :style="{ width: `${progressPercent}%` }" />
        <span
          v-for="m in chapterMarks"
          :key="m.time"
          class="scrub-tick"
          :style="{ left: `${pctOf(Math.max(0, m.time))}%` }"
          :title="m.label"
        />
        <span class="scrub-thumb" :class="{ dragging }" :style="{ left: `${progressPercent}%` }" />
      </div>
      <div
        v-if="hoverPercent !== null"
        class="scrub-bubble"
        :style="{ left: `${hoverPercent}%` }"
      >
        {{ formatTime(hoverSeconds) }}
      </div>
    </div>

    <!-- 操作区 -->
    <div class="deck-row">
      <div class="deck-jumps">
        <button
          type="button"
          class="jump"
          :aria-label="t('podcast.seekBack', { n: 15 })"
          @click="seek(-15)"
        >
          −15s
        </button>
        <button
          type="button"
          class="jump"
          :aria-label="t('podcast.seekForward', { n: 15 })"
          @click="seek(15)"
        >
          +15s
        </button>
        <button
          type="button"
          class="jump"
          :aria-label="t('podcast.seekForward', { n: 30 })"
          @click="seek(30)"
        >
          +30s
        </button>
      </div>

      <span class="deck-remain">−{{ formatTime(remainingSeconds) }}</span>

      <div class="deck-volume">
        <button
          type="button"
          class="vol-btn"
          :title="volumeTitle"
          :aria-label="volumeTitle"
          @click="toggleMute"
        >
          <v-icon :icon="volumeIcon" size="16" aria-hidden="true" />
        </button>
        <div
          ref="ledEl"
          class="led-meter"
          role="slider"
          tabindex="0"
          :aria-label="t('podcast.volume')"
          aria-valuemin="0"
          aria-valuemax="100"
          :aria-valuenow="volumePercent"
          @pointerdown="onLedDown"
          @pointermove="onLedMove"
          @pointerup="ledDragging = false"
          @pointercancel="ledDragging = false"
          @pointerleave="ledDragging = false"
          @keydown.up.prevent="setVolume(volume + 0.1)"
          @keydown.right.prevent="setVolume(volume + 0.1)"
          @keydown.down.prevent="setVolume(volume - 0.1)"
          @keydown.left.prevent="setVolume(volume - 0.1)"
        >
          <span
            v-for="i in LED_COUNT"
            :key="i"
            class="led"
            :class="[ledTone(i), { on: volumePercent >= (i * 100) / LED_COUNT }]"
          />
        </div>
        <span class="vol-value">{{ isMuted ? 'MUTE' : `${volumePercent}%` }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { mdiVolumeHigh, mdiVolumeLow, mdiVolumeMedium, mdiVolumeOff } from '@mdi/js'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { formatTime, parseTimestamps } from '@/utils/timestampParser'

const { t } = useI18n()

const emit = defineEmits<{
  (e: 'drag-state', dragging: boolean): void
}>()

const {
  currentPlaying,
  isPaused,
  currentTime,
  duration,
  bufferedPercent,
  volume,
  isMuted,
  seek,
  seekToAbsolute,
  setVolume,
  toggleMute,
} = useAudioPlayer()

// ===== 进度 =====
const scrubEl = ref<HTMLElement | null>(null)
const dragging = ref(false)
const dragPercent = ref(0)
const hoverPercent = ref<number | null>(null)

const progressPercent = computed(() => {
  const base = duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0
  const value = dragging.value ? dragPercent.value : base
  return Math.min(100, Math.max(0, value))
})

const displaySeconds = computed(() => {
  if (!dragging.value) return currentTime.value
  return (dragPercent.value / 100) * duration.value
})

const hoverSeconds = computed(() => ((hoverPercent.value ?? 0) / 100) * duration.value)

const remainingSeconds = computed(() => {
  const base = dragging.value ? displaySeconds.value : currentTime.value
  return Math.max(0, duration.value - base)
})

function pctOf(seconds: number) {
  if (duration.value <= 0) return 0
  return Math.min(100, Math.max(0, (seconds / duration.value) * 100))
}

function percentFromEvent(event: PointerEvent) {
  const el = scrubEl.value
  if (!el) return 0
  const rect = el.getBoundingClientRect()
  return Math.min(100, Math.max(0, ((event.clientX - rect.left) / rect.width) * 100))
}

function onScrubDown(event: PointerEvent) {
  if (!duration.value) return
  dragging.value = true
  dragPercent.value = percentFromEvent(event)
  emit('drag-state', true)
  ;(event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId)
}

function onScrubMove(event: PointerEvent) {
  if (!duration.value) return
  const percent = percentFromEvent(event)
  hoverPercent.value = percent
  if (dragging.value) dragPercent.value = percent
}

function onScrubUp() {
  if (!dragging.value) return
  dragging.value = false
  emit('drag-state', false)
  seekToAbsolute((dragPercent.value / 100) * duration.value)
}

// 章节/时间戳刻度（数据可能是空数组或纯字符串，做一次健壮过滤）
const chapterMarks = computed(() => {
  const ep = currentPlaying.value
  if (!ep) return [] as Array<{ time: number; label: string }>
  const fromChapters = (ep.chapters || [])
    .filter(c => typeof (c as { start_time?: unknown })?.start_time === 'number')
    .map(c => ({ time: c.start_time, label: c.title }))
  if (fromChapters.length) return fromChapters.slice(0, 40)
  return parseTimestamps(ep.show_notes || '').slice(0, 40)
})

// ===== 音量 =====
const LED_COUNT = 12
const ledEl = ref<HTMLElement | null>(null)
const ledDragging = ref(false)
const volumePercent = computed(() => Math.round(volume.value * 100))

const volumeIcon = computed(() => {
  if (isMuted.value || volume.value === 0) return mdiVolumeOff
  if (volume.value < 0.34) return mdiVolumeLow
  if (volume.value < 0.67) return mdiVolumeMedium
  return mdiVolumeHigh
})

const volumeTitle = computed(() =>
  isMuted.value || volume.value === 0 ? t('podcast.unmute') : t('podcast.mute')
)

function ledTone(index: number) {
  if (index <= 4) return 'low'
  if (index <= 8) return 'mid'
  return 'high'
}

function volumeFromEvent(event: PointerEvent) {
  const el = ledEl.value
  if (!el) return volume.value
  const rect = el.getBoundingClientRect()
  return Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width))
}

function onLedDown(event: PointerEvent) {
  ledDragging.value = true
  setVolume(volumeFromEvent(event))
  ;(event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId)
}

function onLedMove(event: PointerEvent) {
  if (ledDragging.value) setVolume(volumeFromEvent(event))
}
</script>

<style scoped>
.deck {
  width: 420px;
  max-width: calc(100vw - 24px);
  padding: 12px 14px 13px;
  border: 1px solid var(--border-strong);
  border-radius: 12px;
  background: var(--surface-raised);
  color: var(--text);
  box-shadow: var(--shadow-lg, 0 18px 48px rgb(var(--ink-rgb) / 0.28));
  animation: deck-in 160ms ease-out;
}

/* 头部 */
.deck-head {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-bottom: 10px;
}

.deck-reel {
  position: relative;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  border-radius: 50%;
  border: 1px solid rgb(var(--accent-rgb) / 0.55);
  background: rgb(var(--accent-rgb) / 0.06);
  animation: reel-spin 2.4s linear infinite;
  animation-play-state: paused;
}

.deck-reel.spinning {
  animation-play-state: running;
}

.deck-reel::before {
  content: '';
  position: absolute;
  inset: 3px;
  border-radius: 50%;
  background: conic-gradient(
    rgb(var(--accent-rgb) / 0.85) 0 12%,
    transparent 12% 33%,
    rgb(var(--accent-rgb) / 0.85) 33% 45%,
    transparent 45% 66%,
    rgb(var(--accent-rgb) / 0.85) 66% 78%,
    transparent 78% 100%
  );
}

.deck-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  font-weight: 600;
}

.deck-clock {
  flex-shrink: 0;
  font-family: var(--jedi-font-mono);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  color: var(--text-muted);
}

.clock-sep {
  margin: 0 4px;
  opacity: 0.4;
}

/* 进度 */
.deck-scrub {
  position: relative;
  padding: 9px 0;
  cursor: pointer;
  touch-action: none;
  outline: none;
}

.deck-scrub:focus-visible .scrub-rail {
  box-shadow: 0 0 0 2px rgb(var(--accent-rgb) / 0.4);
}

.scrub-rail {
  position: relative;
  height: 8px;
  border-radius: 4px;
  background: rgb(var(--text-rgb) / 0.12);
}

.scrub-buffered {
  position: absolute;
  inset: 0 auto 0 0;
  border-radius: 4px;
  background: rgb(var(--text-rgb) / 0.18);
}

.scrub-played {
  position: absolute;
  inset: 0 auto 0 0;
  border-radius: 4px;
  background: linear-gradient(90deg, var(--accent), var(--accent-hover));
  box-shadow: 0 0 10px rgb(var(--accent-rgb) / 0.4);
}

.scrub-tick {
  position: absolute;
  top: -3px;
  width: 2px;
  height: 14px;
  margin-left: -1px;
  border-radius: 1px;
  background: rgb(var(--text-rgb) / 0.35);
}

.scrub-thumb {
  position: absolute;
  top: 50%;
  width: 14px;
  height: 14px;
  margin-left: -7px;
  border-radius: 50%;
  background: #fff;
  border: 2px solid var(--accent);
  box-shadow: 0 2px 10px rgb(var(--accent-rgb) / 0.5);
  transform: translateY(-50%);
  transition:
    transform 120ms ease,
    left 60ms linear;
  pointer-events: none;
}

.scrub-thumb.dragging {
  transform: translateY(-50%) scale(1.3);
}

.scrub-bubble {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  padding: 2px 7px;
  border: 1px solid var(--border-strong);
  border-radius: 6px;
  background: var(--surface-raised);
  box-shadow: var(--shadow-md);
  color: var(--text);
  font-family: var(--jedi-font-mono);
  font-size: 10px;
  font-weight: 600;
  white-space: nowrap;
  pointer-events: none;
}

/* 操作区 */
.deck-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-top: 8px;
}

.deck-jumps {
  display: flex;
  gap: 6px;
}

.deck-remain {
  flex: 1;
  text-align: center;
  font-family: var(--jedi-font-mono);
  font-size: 10px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--text-muted);
  opacity: 0.8;
}

.jump {
  height: 24px;
  padding: 0 9px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  color: var(--text-muted);
  font-family: var(--jedi-font-mono);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.3px;
  cursor: pointer;
  transition:
    color 0.12s ease,
    border-color 0.12s ease,
    background 0.12s ease;
}

.jump:hover {
  color: var(--text);
  border-color: var(--accent);
  background: rgb(var(--accent-rgb) / 0.08);
}

.jump:active {
  transform: translateY(1px);
}

.deck-volume {
  display: flex;
  align-items: center;
  gap: 7px;
}

.vol-btn {
  display: grid;
  width: 22px;
  height: 22px;
  place-items: center;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
}

.vol-btn:hover {
  color: var(--text);
  background: rgb(var(--text-rgb) / 0.06);
}

.led-meter {
  display: flex;
  align-items: flex-end;
  gap: 2px;
  height: 16px;
  cursor: pointer;
  touch-action: none;
  outline: none;
}

.led-meter:focus-visible {
  border-radius: 3px;
  box-shadow: 0 0 0 2px rgb(var(--accent-rgb) / 0.4);
}

.led {
  width: 4px;
  border-radius: 1px;
  background: rgb(var(--text-rgb) / 0.13);
  transition:
    background 0.1s ease,
    box-shadow 0.1s ease;
}

.led.low {
  height: 8px;
}

.led.mid {
  height: 11px;
}

.led.high {
  height: 14px;
}

.led.on.low {
  background: var(--success);
  box-shadow: 0 0 6px rgb(var(--success-rgb, 32 138 60) / 0.5);
}

.led.on.mid {
  background: var(--warning);
  box-shadow: 0 0 6px rgb(var(--warning-rgb, 164 103 4) / 0.5);
}

.led.on.high {
  background: var(--danger);
  box-shadow: 0 0 6px rgb(var(--danger-rgb) / 0.5);
}

.vol-value {
  min-width: 32px;
  text-align: right;
  font-family: var(--jedi-font-mono);
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
}

@keyframes reel-spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes deck-in {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

@media (prefers-reduced-motion: reduce) {
  .deck,
  .deck-reel {
    animation: none;
  }
}
</style>
