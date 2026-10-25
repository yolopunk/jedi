<template>
  <div class="player-bar" :class="{ 'wheel-open': showWheel }">
    <!-- Play/Pause -->
    <v-btn
      icon
      size="x-small"
      variant="text"
      color="primary"
      class="player-btn"
      :aria-label="isPaused ? t('podcast.play') : t('podcast.pause')"
      @click.stop="togglePlayLocal"
    >
      <v-icon :icon="isPaused ? mdiPlay : mdiPause" size="14" aria-hidden="true" />
    </v-btn>

    <!-- Title -->
    <span class="player-title text-truncate">{{ currentPlaying?.title || '' }}</span>

    <!-- Time & Progress -->
    <div class="player-progress-area">
      <span class="player-time">{{ formatTime(currentTime) }}</span>
      <input
        type="range"
        class="progress-track"
        min="0"
        :max="duration"
        :value="currentTime"
        step="1"
        :aria-label="`${formatTime(currentTime)} / ${formatTime(duration)}`"
        :style="{ '--progress-fill': `${progressPercent}%` }"
        @input="onSeekInput"
      />
      <span class="player-time">{{ formatTime(duration) }}</span>
    </div>

    <!-- Time Wheel Toggle -->
    <v-btn
      icon
      size="x-small"
      variant="text"
      class="wheel-btn"
      :class="{ active: showWheel }"
      :title="t('podcast.chaptersButton')"
      :aria-label="t('podcast.chaptersButton')"
      @click.stop="showWheel = !showWheel"
    >
      <span class="wheel-icon" aria-hidden="true">◐</span>
    </v-btn>

    <!-- Close player -->
    <v-btn
      icon
      size="x-small"
      variant="text"
      class="close-btn"
      :title="t('podcast.stopPlayer')"
      :aria-label="t('podcast.stopPlayer')"
      @click.stop="stop"
    >
      <v-icon icon="mdiClose" size="12" aria-hidden="true" />
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
import { mdiPause, mdiPlay } from '@mdi/js'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { formatTime, parseTimestamps } from '@/utils/timestampParser'
import TimeWheel from './TimeWheel.vue'

const { t } = useI18n()
const { currentPlaying, isPaused, currentTime, duration, togglePlay, seek, stop } = useAudioPlayer()

const showWheel = ref(false)

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

function onSeekInput(event: Event) {
  const target = event.target as HTMLInputElement
  seek(Number(target.value) - currentTime.value)
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
  margin: 0;
  min-width: 30px;
  -webkit-appearance: none;
  appearance: none;
  background: linear-gradient(
    to right,
    rgb(var(--v-theme-primary)) var(--progress-fill, 0%),
    rgba(var(--v-theme-on-surface), 0.12) var(--progress-fill, 0%)
  );
  border-radius: 2px;
  cursor: pointer;
  outline-offset: 2px;
}

.progress-track:hover {
  height: 5px;
}

.progress-track::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: rgb(var(--v-theme-primary));
  opacity: 0;
  transition: opacity 0.15s;
}

.progress-track:hover::-webkit-slider-thumb,
.progress-track:focus-visible::-webkit-slider-thumb {
  opacity: 1;
}

.progress-track:focus-visible {
  outline: 1px solid rgb(var(--v-theme-primary) / 0.6);
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
  z-index: var(--z-scrim);
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
