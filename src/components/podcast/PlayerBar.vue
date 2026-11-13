<template>
  <div
    ref="barEl"
    class="player-bar"
    @mouseenter="scheduleOpen"
    @mouseleave="scheduleClose"
    @focusin="scheduleOpen"
  >
    <!-- Play/Pause -->
    <v-btn
      icon
      size="x-small"
      variant="text"
      color="primary"
      class="player-btn"
      :class="{ playing: !isPaused }"
      :aria-label="isPaused ? t('podcast.play') : t('podcast.pause')"
      @click.stop="togglePlay"
    >
      <v-icon :icon="isPaused ? mdiPlay : mdiPause" size="14" aria-hidden="true" />
    </v-btn>

    <!-- Title -->
    <span class="player-title text-truncate">{{ currentPlaying?.title || '' }}</span>

    <!-- 紧凑进度：细轨 + 章节刻度，悬停向上展开播放台 -->
    <div class="player-progress-area">
      <span class="player-time">{{ formatTime(currentTime) }}</span>
      <div
        ref="miniRailEl"
        class="mini-rail"
        role="slider"
        tabindex="0"
        :aria-label="t('podcast.progress')"
        aria-valuemin="0"
        :aria-valuemax="Math.round(duration)"
        :aria-valuenow="Math.round(currentTime)"
        :aria-valuetext="`${formatTime(currentTime)} / ${formatTime(duration)}`"
        @click.stop="onMiniClick"
        @keydown.left.prevent="seek(-5)"
        @keydown.right.prevent="seek(5)"
      >
        <div class="mini-track">
          <div class="mini-buffered" :style="{ width: `${bufferedPercent}%` }" />
          <div class="mini-played" :style="{ width: `${progressPercent}%` }" />
          <span
            v-for="m in miniTicks"
            :key="m.time"
            class="mini-tick"
            :style="{ left: `${pctOf(m.time)}%` }"
          />
          <span class="mini-head" :class="{ paused: isPaused }" :style="{ left: `${progressPercent}%` }" />
        </div>
      </div>
      <span class="player-time">{{ formatTime(duration) }}</span>
    </div>

    <!-- 音量（图标切静音，详细调节在展开面板里） -->
    <v-btn
      icon
      size="x-small"
      variant="text"
      class="volume-btn"
      :title="volumeTitle"
      :aria-label="volumeTitle"
      @click.stop="toggleMute"
    >
      <v-icon :icon="volumeIcon" size="14" aria-hidden="true" />
    </v-btn>

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

    <!-- 悬浮播放台 -->
    <Teleport to="body">
      <div
        v-if="panelOpen"
        class="player-deck-host"
        :style="deckStyle"
        @mouseenter="cancelClose"
        @mouseleave="scheduleClose"
      >
        <PlayerDeck @drag-state="onDeckDragState" />
      </div>
    </Teleport>

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
import {
  mdiPause,
  mdiPlay,
  mdiVolumeHigh,
  mdiVolumeLow,
  mdiVolumeMedium,
  mdiVolumeOff,
} from '@mdi/js'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { formatTime, parseTimestamps } from '@/utils/timestampParser'
import PlayerDeck from './PlayerDeck.vue'
import TimeWheel from './TimeWheel.vue'

const { t } = useI18n()
const {
  currentPlaying,
  isPaused,
  currentTime,
  duration,
  bufferedPercent,
  volume,
  isMuted,
  togglePlay,
  seek,
  seekToAbsolute,
  stop,
  toggleMute,
} = useAudioPlayer()

const showWheel = ref(false)
const barEl = ref<HTMLElement | null>(null)
const miniRailEl = ref<HTMLElement | null>(null)

// ===== 悬浮播放台的开合与定位 =====
const panelOpen = ref(false)
const anchor = ref({ left: 0, top: 0 })
let openTimer: number | null = null
let closeTimer: number | null = null
// 拖拽进度条期间保持展开
let holding = false

const DECK_WIDTH = 420
const DECK_GAP = 8

const deckStyle = computed(() => {
  const maxLeft = Math.max(8, window.innerWidth - DECK_WIDTH - 12)
  const left = Math.min(Math.max(8, anchor.value.left), maxLeft)
  return {
    position: 'fixed' as const,
    left: `${left}px`,
    bottom: `${window.innerHeight - anchor.value.top + DECK_GAP}px`,
  }
})

function updateAnchor() {
  const el = barEl.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  anchor.value = { left: rect.left, top: rect.top }
}

function scheduleOpen() {
  if (closeTimer) {
    clearTimeout(closeTimer)
    closeTimer = null
  }
  if (panelOpen.value || openTimer) return
  openTimer = window.setTimeout(() => {
    openTimer = null
    updateAnchor()
    panelOpen.value = true
  }, 140)
}

function scheduleClose() {
  // 正在拖拽进度条时不要收起，否则 pointerup 会丢
  if (holding) return
  if (openTimer) {
    clearTimeout(openTimer)
    openTimer = null
  }
  if (closeTimer) clearTimeout(closeTimer)
  closeTimer = window.setTimeout(() => {
    closeTimer = null
    panelOpen.value = false
  }, 240)
}

function onDeckDragState(dragging: boolean) {
  holding = dragging
  if (dragging) cancelClose()
}

function cancelClose() {
  if (closeTimer) {
    clearTimeout(closeTimer)
    closeTimer = null
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && panelOpen.value) {
    cancelClose()
    panelOpen.value = false
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  window.addEventListener('resize', updateAnchor)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
  window.removeEventListener('resize', updateAnchor)
  if (openTimer) clearTimeout(openTimer)
  if (closeTimer) clearTimeout(closeTimer)
})

// ===== 紧凑进度 =====
const progressPercent = computed(() => {
  if (duration.value <= 0) return 0
  return Math.min(100, Math.max(0, (currentTime.value / duration.value) * 100))
})

function pctOf(seconds: number) {
  if (duration.value <= 0) return 0
  return Math.min(100, Math.max(0, (seconds / duration.value) * 100))
}

// 状态栏太窄，刻度只取前若干个，避免糊成一片
const miniTicks = computed(() => chapterMarks.value.slice(0, 14))

const chapterMarks = computed(() => {
  const ep = currentPlaying.value
  if (!ep) return [] as Array<{ time: number; label: string }>
  const fromChapters = (ep.chapters || [])
    .filter(c => typeof (c as { start_time?: unknown })?.start_time === 'number')
    .map(c => ({ time: c.start_time, label: c.title }))
  if (fromChapters.length) return fromChapters
  return parseTimestamps(ep.show_notes || '')
})

const timestampMarkers = computed(() => {
  const ep = currentPlaying.value
  if (!ep) return []
  if (ep.chapters?.length) {
    return ep.chapters.map(c => ({ time: c.start_time, label: c.title }))
  }
  return parseTimestamps(ep.show_notes || '')
})

const volumeIcon = computed(() => {
  if (isMuted.value || volume.value === 0) return mdiVolumeOff
  if (volume.value < 0.34) return mdiVolumeLow
  if (volume.value < 0.67) return mdiVolumeMedium
  return mdiVolumeHigh
})

const volumeTitle = computed(() =>
  isMuted.value || volume.value === 0 ? t('podcast.unmute') : t('podcast.mute')
)

function onMiniClick(event: MouseEvent) {
  const el = miniRailEl.value
  if (!el || duration.value <= 0) return
  const rect = el.getBoundingClientRect()
  const percent = Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width))
  seekToAbsolute(percent * duration.value)
}

function seekTo(seconds: number) {
  seek(seconds - currentTime.value)
}
</script>

<style scoped>
/* 浮层挂在 body 上：需要高于应用内容（.main-content 等带 z-index: 1），
   但要低于抽屉/对话框（--z-scrim: 3000） */
.player-deck-host {
  z-index: 400;
}

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
.volume-btn,
.wheel-btn,
.close-btn {
  width: 18px !important;
  height: 18px !important;
  min-width: 18px !important;
  flex-shrink: 0;
}

/* 播放时给个轻微的呼吸光 */
.player-btn.playing :deep(.v-icon) {
  animation: play-pulse 2.4s ease-in-out infinite;
}

.player-title {
  font-size: 10px;
  color: rgba(var(--v-theme-on-surface), 0.7);
  width: 140px;
  max-width: 140px;
  flex-shrink: 0;
  min-width: 0;
}

.player-progress-area {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
  min-width: 104px;
}

.player-time {
  font-size: 9px;
  font-family: 'JetBrains Mono', monospace;
  color: rgba(var(--v-theme-on-surface), 0.5);
  min-width: 32px;
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}

.player-time:first-child {
  text-align: right;
}

/* 紧凑进度轨 */
.mini-rail {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 40px;
  height: 16px;
  cursor: pointer;
  outline: none;
}

.mini-track {
  position: relative;
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: rgba(var(--v-theme-on-surface), 0.12);
  transition: height 0.12s ease;
}

.mini-rail:hover .mini-track {
  height: 6px;
}

.mini-rail:focus-visible .mini-track {
  box-shadow: 0 0 0 2px rgb(var(--accent-rgb) / 0.45);
}

.mini-buffered {
  position: absolute;
  inset: 0 auto 0 0;
  border-radius: 2px;
  background: rgba(var(--v-theme-on-surface), 0.18);
}

.mini-played {
  position: absolute;
  inset: 0 auto 0 0;
  border-radius: 2px;
  background: linear-gradient(90deg, var(--accent), var(--accent-hover));
}

.mini-tick {
  position: absolute;
  top: -2px;
  bottom: -2px;
  width: 1px;
  margin-left: -0.5px;
  background: rgba(var(--v-theme-on-surface), 0.3);
  pointer-events: none;
}

.mini-head {
  position: absolute;
  top: 50%;
  width: 6px;
  height: 6px;
  margin-left: -3px;
  border-radius: 50%;
  background: var(--accent);
  box-shadow: 0 0 0 2px rgb(var(--accent-rgb) / 0.25);
  transform: translateY(-50%);
  transition:
    width 0.12s ease,
    height 0.12s ease,
    margin-left 0.12s ease;
  pointer-events: none;
}

.mini-rail:hover .mini-head {
  width: 9px;
  height: 9px;
  margin-left: -4.5px;
}

.mini-head.paused {
  opacity: 0.75;
}

/* Time Wheel Panel */
.wheel-backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--z-scrim);
  background: rgb(var(--ink-rgb) / 0.3);
  display: flex;
  align-items: flex-end;
  justify-content: center;
}

.wheel-panel {
  width: 100%;
  max-width: 500px;
  margin-bottom: 32px;
}

@keyframes play-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.65;
  }
}

@media (prefers-reduced-motion: reduce) {
  .player-btn.playing :deep(.v-icon) {
    animation: none;
  }
}
</style>
