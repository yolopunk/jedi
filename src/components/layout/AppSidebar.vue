<template>
  <v-navigation-drawer permanent :width="220" class="jedi-sidebar">
    <!-- Logo Area -->
    <div class="d-flex flex-column align-center py-6 border-bottom">
      <div class="grogu-pod-container mb-4" :class="{ 'is-playing': currentPlaying && !isPaused, 'is-paused': currentPlaying && isPaused }">
        <!-- Floating Pod Effect -->
        <div class="pod-chassis">
          <div class="model-wrapper" @click="handleModelClick">
            <!-- Playback Progress Ring -->
            <v-progress-circular
              v-if="currentPlaying"
              :model-value="progressPercentage"
              :rotate="-90"
              :size="92"
              :width="4"
              color="primary"
              class="progress-ring"
              bg-color="rgba(255, 255, 255, 0.1)"
            >
            </v-progress-circular>
            
            <!-- The 3D Model -->
            <KoalaModel />

            <!-- Play/Pause Overlay -->
            <div v-if="currentPlaying" class="player-overlay d-flex align-center justify-center">
              <v-icon 
                :icon="isPaused ? mdiPlay : mdiPause" 
                size="32" 
                color="white"
                class="control-icon"
              ></v-icon>
            </div>
          </div>
        </div>

        <!-- Satellite Controls -->
        <div v-if="currentPlaying" class="pod-controls">
          <v-btn 
            icon 
            size="x-small" 
            variant="tonal" 
            color="primary" 
            class="pod-ctrl-btn rewind-btn"
            @click.stop="seek(-15)"
          >
            <v-icon :icon="mdiRewind15" size="16"></v-icon>
            <v-tooltip activator="parent" location="bottom">Rewind 15s</v-tooltip>
          </v-btn>

          <v-btn 
            icon 
            size="x-small" 
            variant="tonal" 
            color="error" 
            class="pod-ctrl-btn stop-btn"
            @click.stop="stop"
          >
            <v-icon :icon="mdiStop" size="16"></v-icon>
            <v-tooltip activator="parent" location="bottom">Stop Playback</v-tooltip>
          </v-btn>
          
          <v-btn 
            icon 
            size="x-small" 
            variant="tonal" 
            color="primary" 
            class="pod-ctrl-btn forward-btn"
            @click.stop="seek(15)"
          >
            <v-icon :icon="mdiFastForward15" size="16"></v-icon>
            <v-tooltip activator="parent" location="bottom">Forward 15s</v-tooltip>
          </v-btn>
        </div>
      </div>
      
      <!-- Title/Subtitle or Now Playing Info -->
      <div v-if="currentPlaying" class="text-center px-2" style="max-width: 100%;">
        <div class="text-caption font-weight-bold text-primary text-truncate">{{ currentPlaying.title }}</div>
        <div class="text-caption text-secondary text-truncate" style="font-size: 0.7rem !important;">{{ formatTime(currentTime) }} / {{ formatTime(duration) }}</div>
      </div>
      <div v-else class="text-center">
        <h2 class="text-h6 font-weight-bold text-primary">{{ $t('sidebar.title') }}</h2>
        <div class="text-caption text-secondary">{{ $t('sidebar.subtitle') }}</div>
      </div>
    </div>

    <!-- Navigation -->
    <v-list nav class="pa-2 mt-2">
      <v-list-item
        to="/hosts"
        rounded="lg"
        class="mb-2 sidebar-item"
        color="primary"
        active-class="v-list-item--active"
      >
        <template v-slot:prepend>
          <div class="sidebar-icon-container mr-3">
            <v-icon :icon="mdiDns" size="20"></v-icon>
          </div>
        </template>
        <v-list-item-title class="font-weight-medium">{{ $t('sidebar.hostsManager') }}</v-list-item-title>
      </v-list-item>

      <v-list-item
        to="/wallpapers"
        rounded="lg"
        class="mb-2 sidebar-item"
        color="primary"
        active-class="v-list-item--active"
      >
        <template v-slot:prepend>
          <div class="sidebar-icon-container mr-3">
            <v-icon :icon="mdiWallpaper" size="20"></v-icon>
          </div>
        </template>
        <v-list-item-title class="font-weight-medium">{{ $t('sidebar.wallpapers') }}</v-list-item-title>
      </v-list-item>

      <v-list-item
        to="/podcast"
        rounded="lg"
        class="mb-2 sidebar-item"
        color="primary"
        active-class="v-list-item--active"
      >
        <template v-slot:prepend>
          <div class="sidebar-icon-container mr-3">
            <v-icon :icon="mdiPodcast" size="20"></v-icon>
          </div>
        </template>
        <v-list-item-title class="font-weight-medium">{{ $t('sidebar.podcast') }}</v-list-item-title>
      </v-list-item>
    </v-list>

    <!-- Footer Area: Jedi Control Deck -->
    <template v-slot:append>
      <div class="pa-4">
        <div class="jedi-control-deck d-flex align-center justify-space-between px-1 py-1">
          
          <!-- Power Core (Theme) -->
          <div class="power-core-wrapper">
            <v-btn 
              icon 
              density="compact" 
              variant="text" 
              @click="toggleTheme" 
              class="power-core-btn"
              :class="{ 'core-active': isDark }"
            >
              <v-icon :icon="themeIcon" size="20" class="core-icon"></v-icon>
              <div class="core-glow"></div>
              <v-tooltip activator="parent" location="top">{{ themeTooltip }}</v-tooltip>
            </v-btn>
          </div>
          
          <div class="deck-divider mx-1"></div>
          
          <!-- Control Module (Settings & Github) -->
          <div class="d-flex align-center control-module">
            <v-btn 
              icon 
              density="compact" 
              variant="text" 
              size="small" 
              @click="$emit('show-settings')" 
              class="deck-btn"
            >
              <v-icon :icon="mdiCog" size="18"></v-icon>
              <v-tooltip activator="parent" location="top">{{ $t('settings.title') }}</v-tooltip>
            </v-btn>
            
            <v-btn 
              icon 
              density="compact" 
              variant="text" 
              size="small" 
              @click="$emit('open-github')" 
              class="deck-btn"
            >
              <v-icon :icon="mdiGithub" size="18"></v-icon>
              <v-tooltip activator="parent" location="top">{{ $t('sidebar.github') }}</v-tooltip>
            </v-btn>
          </div>
        </div>
        
        <div class="text-center mt-3 text-caption jedi-status-text">
          <span class="status-dot"></span> {{ $t('sidebar.status_connected') }}
        </div>
      </div>
    </template>
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import KoalaModel from '../common/KoalaModel.vue'
import { mdiDns, mdiWallpaper, mdiPodcast, mdiCog, mdiGithub, mdiWeatherNight, mdiWeatherSunny, mdiThemeLightDark, mdiPlay, mdiPause, mdiRewind15, mdiFastForward15, mdiStop } from '@mdi/js'
import { useTheme } from '@/composables/useTheme'
import { useAudioPlayer } from '@/composables/useAudioPlayer'

const { t } = useI18n()
const { isDark, themeMode, setTheme } = useTheme()
const { currentPlaying, isPaused, currentTime, duration, togglePlay, seek, stop } = useAudioPlayer()

// defineProps<{
//   activeItem?: string
// }>()

const emit = defineEmits(['show-settings', 'open-github'])

// function selectItem(item: string) {
//   emit('update:activeItem', item)
// }

const progressPercentage = computed(() => {
  if (!duration.value) return 0
  return (currentTime.value / duration.value) * 100
})

function formatTime(seconds: number): string {
  if (!seconds || isNaN(seconds)) return '00:00'
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
}

function handleModelClick() {
  if (currentPlaying.value) {
    togglePlay()
  }
}

const themeIcon = computed(() => {
  if (themeMode.value === 'dark') return mdiWeatherNight
  if (themeMode.value === 'light') return mdiWeatherSunny
  return mdiThemeLightDark
})

const themeTooltip = computed(() => {
  if (themeMode.value === 'dark') return t('theme.dark')
  if (themeMode.value === 'light') return t('theme.light')
  return t('theme.system')
})

function toggleTheme() {
  if (themeMode.value === 'light') {
    setTheme('dark')
  } else if (themeMode.value === 'dark') {
    setTheme('system')
  } else {
    setTheme('light')
  }
}

</script>

<style scoped>
/* Minimal scoped styles, rely on global theme */
.jedi-sidebar {
  /* Ensure border is visible */
  border-right: 1px solid var(--jedi-border) !important;
}

.border-bottom {
  border-bottom: 1px solid var(--jedi-border);
}

.grogu-pod-container {
  position: relative;
  width: 120px;
  height: 120px;
  display: flex;
  justify-content: center;
  align-items: center;
}

.pod-chassis {
  position: relative;
  width: 100px;
  height: 100px;
  border-radius: 50%;
  background: radial-gradient(circle at 30% 30%, rgba(255, 255, 255, 0.15), rgba(0, 0, 0, 0.4));
  border: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  justify-content: center;
  align-items: center;
  box-shadow: 
    0 10px 20px rgba(0, 0, 0, 0.3),
    inset 0 1px 1px rgba(255, 255, 255, 0.2),
    inset 0 -2px 5px rgba(0, 0, 0, 0.5);
  transition: transform 0.3s ease, box-shadow 0.3s ease;
  z-index: 10;
}

.pod-chassis::before {
  /* Metallic Rim */
  content: '';
  position: absolute;
  top: -4px;
  left: -4px;
  right: -4px;
  bottom: -4px;
  border-radius: 50%;
  background: conic-gradient(
    from 180deg,
    #334155,
    #94a3b8,
    #334155,
    #94a3b8,
    #334155
  );
  z-index: -1;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
}

.pod-chassis::after {
  /* Status Light */
  content: '';
  position: absolute;
  bottom: 8px;
  width: 6px;
  height: 6px;
  background-color: #ef4444; /* Red idle */
  border-radius: 50%;
  box-shadow: 0 0 5px #ef4444;
  transition: background-color 0.3s;
  z-index: 15;
}

.is-playing .pod-chassis::after {
  background-color: #10b981; /* Green playing */
  box-shadow: 0 0 8px #10b981;
  animation: pulse-status 2s infinite;
}

.is-paused .pod-chassis::after {
  background-color: #f59e0b; /* Amber/Yellow paused */
  box-shadow: 0 0 8px #f59e0b;
}

.is-playing .pod-chassis {
  animation: pod-float 4s ease-in-out infinite;
  box-shadow: 
    0 20px 40px rgba(0, 0, 0, 0.4),
    0 0 20px rgba(var(--v-theme-primary), 0.15),
    inset 0 1px 1px rgba(255, 255, 255, 0.2);
}

@keyframes pod-float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}

.model-wrapper {
  position: relative;
  width: 92px;
  height: 92px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 50%;
  background: transparent; /* Removed dark background */
  overflow: hidden; /* Clip contents to circle */
  border: 1px solid rgba(255,255,255,0.05);
}

.player-overlay {
  opacity: 1;
}

.progress-ring {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 1;
  filter: drop-shadow(0 0 4px rgba(var(--v-theme-primary), 0.4));
}

.player-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.4);
  border-radius: 50%;
  opacity: 0;
  transition: all 0.3s;
  z-index: 20;
  backdrop-filter: blur(2px);
}

.control-icon {
  filter: drop-shadow(0 2px 4px rgba(0,0,0,0.6));
  transform: scale(1);
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.model-wrapper:hover .control-icon {
  transform: scale(1.2);
}

.pod-controls {
  position: absolute;
  width: 100%;
  height: 100%;
  pointer-events: none; /* Let clicks pass through to chassis where appropriate */
  z-index: 5;
}

.pod-ctrl-btn {
  position: absolute;
  pointer-events: auto;
  bottom: 0;
  transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  opacity: 0;
  transform: scale(0.8);
  background: rgba(var(--v-theme-surface), 0.9) !important;
  border: 1px solid rgba(var(--v-border-color), 0.5);
  backdrop-filter: blur(4px);
}

.grogu-pod-container:hover .pod-ctrl-btn {
  opacity: 1;
  transform: scale(1);
}

.rewind-btn {
  left: -4px;
  bottom: 4px;
}

.stop-btn {
  left: 50%;
  transform: translateX(-50%) scale(0.8);
  bottom: -12px;
}

.grogu-pod-container:hover .stop-btn {
  transform: translateX(-50%) scale(1);
}

.forward-btn {
  right: -4px;
  bottom: 4px;
}

.pod-ctrl-btn:hover {
  background: rgb(var(--v-theme-primary)) !important;
  color: white !important;
  transform: scale(1.1) !important;
  box-shadow: 0 4px 12px rgba(var(--v-theme-primary), 0.4);
}

.stop-btn:hover {
  transform: translateX(-50%) scale(1.1) !important;
}

.jedi-logo-glow {
  filter: drop-shadow(0 0 10px rgba(59, 130, 246, 0.3));
}

/* Sidebar Item Active State Override for Light Theme */
:global(.light-theme) .sidebar-item.v-list-item--active {
  background-color: #e0f2fe !important; /* Light Blue-50 */
  color: #0369a1 !important; /* Sky-700 */
}

/* Use a subtle gradient for dark mode active state */
:global(.dark-theme) .sidebar-item.v-list-item--active {
  background: linear-gradient(90deg, rgba(59, 130, 246, 0.15), transparent) !important;
  color: #60a5fa !important; /* Blue-400 */
  border-left: 2px solid #60a5fa;
}

/* Jedi Control Deck - Bolder Design */
.jedi-control-deck {
  background: linear-gradient(135deg, rgba(30, 41, 59, 0.9), rgba(15, 23, 42, 0.95));
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 12px;
  position: relative;
  overflow: hidden;
  box-shadow: 
    0 4px 6px -1px rgba(0, 0, 0, 0.1), 
    0 2px 4px -1px rgba(0, 0, 0, 0.06),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* Holographic sheen */
.jedi-control-deck::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(59, 130, 246, 0.8), transparent);
  opacity: 0.5;
}

.jedi-control-deck:hover {
  border-color: rgba(59, 130, 246, 0.6);
  box-shadow: 
    0 0 15px rgba(59, 130, 246, 0.2),
    inset 0 0 20px rgba(59, 130, 246, 0.05);
  transform: translateY(-2px);
}

/* Power Core (Theme Toggle) */
.power-core-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.2);
  margin: 2px;
}

.power-core-btn {
  color: rgba(255, 255, 255, 0.7);
  transition: all 0.3s ease;
  position: relative;
  z-index: 2;
}

.power-core-btn:hover {
  color: #fff;
  text-shadow: 0 0 8px rgba(255, 255, 255, 0.8);
}

.core-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 0;
  height: 0;
  background: radial-gradient(circle, rgba(59, 130, 246, 0.8) 0%, transparent 70%);
  transition: all 0.4s ease;
  border-radius: 50%;
  opacity: 0;
  z-index: 1;
}

.power-core-btn:hover .core-glow {
  width: 40px;
  height: 40px;
  opacity: 0.6;
}

.core-active .core-icon {
  color: #fbbf24; /* Amber for light/sun */
  filter: drop-shadow(0 0 5px rgba(251, 191, 36, 0.5));
}

/* Sidebar Icon Redesign */
.sidebar-icon-container {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(var(--v-theme-on-surface), 0.05);
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  color: rgba(var(--v-theme-on-surface), 0.7);
}

.sidebar-item:hover .sidebar-icon-container {
  background-color: rgba(var(--v-theme-primary), 0.15);
  transform: scale(1.1);
  color: rgb(var(--v-theme-primary));
}

.v-list-item--active .sidebar-icon-container {
  background: linear-gradient(135deg, rgb(var(--v-theme-primary)), rgba(var(--v-theme-primary), 0.8));
  color: white !important;
  box-shadow: 0 4px 12px rgba(var(--v-theme-primary), 0.4);
}

/* Deck Divider */
.deck-divider {
  width: 1px;
  height: 24px;
  background: linear-gradient(to bottom, transparent, rgba(255, 255, 255, 0.2), transparent);
}

/* Deck Buttons */
.deck-btn {
  color: rgba(255, 255, 255, 0.6);
  transition: all 0.2s;
  margin: 0 2px;
}

.deck-btn:hover {
  color: #60a5fa; /* Blue-400 */
  background: rgba(59, 130, 246, 0.1);
  transform: scale(1.1);
}

/* Status Text */
.jedi-status-text {
  font-size: 9px !important;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  opacity: 0.7;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
}

.status-dot {
  width: 4px;
  height: 4px;
  background-color: #10b981; /* Emerald-500 */
  border-radius: 50%;
  margin-right: 6px;
  box-shadow: 0 0 8px #10b981;
  animation: pulse-status 3s infinite;
}

@keyframes pulse-status {
  0%, 100% { opacity: 0.5; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.5); }
}
</style>
