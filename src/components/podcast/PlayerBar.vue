<template>
  <div class="player-bar" :class="{ 'wheel-open': showWheel }">
    <!-- Play/Pause -->
    <v-btn
      icon
      size="x-small"
      variant="text"
      color="primary"
      class="player-btn"
      @click.stop="togglePlayLocal"
    >
      <v-icon :icon="isPaused ? mdiPlay : mdiPause" size="14" />
    </v-btn>

    <!-- Title -->
    <span class="player-title text-truncate">{{ currentPlaying?.title || '' }}</span>

    <!-- Time & Progress -->
    <div class="player-progress-area">
      <span class="player-time">{{ formatTime(currentTime) }}</span>
      <div class="progress-track" ref="progressTrack"
        @mousedown="onTrackMouseDown"
        @click="seekByClick">
        <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
        <div class="progress-thumb" :style="{ left: progressPercent + '%' }"></div>
      </div>
      <span class="player-time">{{ formatTime(duration) }}</span>
    </div>

    <!-- Time Wheel Toggle -->
    <v-btn
      icon
      size="x-small"
      variant="text"
      class="wheel-btn"
      :class="{ active: showWheel }"
      @click.stop="showWheel = !showWheel"
      :title="'Chapters'"
    >
      <span class="wheel-icon">◐</span>
    </v-btn>

    <!-- Close player -->
    <v-btn
      icon
      size="x-small"
      variant="text"
      class="close-btn"
      @click.stop="stop"
      title="Stop"
    >
      <v-icon icon="mdiClose" size="12" />
    </v-btn>

    <!-- Time Wheel Panel -->
    <Teleport to="body">
      <div v-if="showWheel" class="wheel-backdrop" @click="showWheel = false">
        <div class="wheel-panel" @click.stop>
          <TimeWheel
            :duration="duration"
            :current-time="currentTime"
            :markers="timestampMarkers"
            @seek="seekTo"
          />
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { mdiPause, mdiPlay } from '@mdi/js'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { parseTimestamps, formatTime } from '@/utils/timestampParser'
import TimeWheel from './TimeWheel.vue'

const { currentPlaying, isPaused, currentTime, duration, togglePlay, seek, stop } = useAudioPlayer()

const showWheel = ref(false)
const progressTrack = ref<HTMLElement | null>(null)

const progressPercent = computed(() => {
  if (!duration.value || duration.value <= 0) return 0
  return Math.min((currentTime.value / duration.value) * 100, 100)
})

const timestampMarkers = computed(() => {
  const ep = currentPlaying.value
  if (!ep) return []
  // Prefer chapters from podcast data
  if (ep.chapters?.length) {
    return ep.chapters.map(c => ({ time: c.start_time, label: c.title }))
  }
  // Fall back to parsing show notes
  return parseTimestamps(ep.show_notes || '')
})

function togglePlayLocal() {
  togglePlay()
}

function seekTo(seconds: number) {
  seek(seconds - currentTime.value)
}

function seekByClick(e: MouseEvent) {
  if (!progressTrack.value || !duration.value) return
  const rect = progressTrack.value.getBoundingClientRect()
  const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
  seek(ratio * duration.value - currentTime.value)
}

function onTrackMouseDown(e: MouseEvent) {
  seekByClick(e)
  const onMove = (ev: MouseEvent) => seekByClick(ev)
  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}
</script>

<style scoped>
.player-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 100%;
  flex: 1;
  min-width: 0;
  padding: 0 4px;
}

.player-btn,
.wheel-btn,
.close-btn {
  width: 18px !important;
  height: 18px !important;
  min-width: 18px !important;
  flex-shrink: 0;
}

.player-title {
  font-size: 10px;
  color: rgba(var(--v-theme-on-surface), 0.7);
  max-width: 180px;
  flex-shrink: 1;
  min-width: 0;
}

.player-progress-area {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
  min-width: 80px;
}

.player-time {
  font-size: 9px;
  font-family: 'JetBrains Mono', monospace;
  color: rgba(var(--v-theme-on-surface), 0.5);
  min-width: 32px;
  flex-shrink: 0;
}

.player-time:first-child {
  text-align: right;
}

.progress-track {
  flex: 1;
  height: 3px;
  background: rgba(var(--v-theme-on-surface), 0.12);
  border-radius: 2px;
  cursor: pointer;
  position: relative;
  min-width: 30px;
}

.progress-track:hover {
  height: 5px;
}

.progress-fill {
  height: 100%;
  background: rgb(var(--v-theme-primary));
  border-radius: 2px;
  transition: width 0.1s linear;
}

.progress-thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 8px;
  height: 8px;
  background: rgb(var(--v-theme-primary));
  border-radius: 50%;
  opacity: 0;
  transition: opacity 0.15s;
}

.progress-track:hover .progress-thumb {
  opacity: 1;
}

.wheel-btn .wheel-icon {
  font-size: 12px;
  line-height: 1;
}

.wheel-btn.active {
  color: rgb(var(--v-theme-primary));
}

/* Time Wheel Panel */
.wheel-backdrop {
  position: fixed;
  inset: 0;
  z-index: 3000;
  background:rgb(var(--ink-rgb) / 0.3);
  display: flex;
  align-items: flex-end;
  justify-content: center;
}

.wheel-panel {
  width: 100%;
  max-width: 500px;
  margin-bottom: 32px;
}
</style>
