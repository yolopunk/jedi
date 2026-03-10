<template>
  <div
    class="sidebar-wrapper"
    :class="{ 'sidebar-collapsed': isCollapsed }"
    :style="{ width: isCollapsed ? '64px' : `${width}px` }"
  >
    <aside
      class="jedi-sidebar d-flex flex-column"
      :class="{ 'sidebar-collapsed': isCollapsed }"
      :style="{ width: isCollapsed ? '64px' : `${width}px` }"
    >
      <!-- Logo Area -->
      <div class="d-flex flex-column align-center py-2 border-bottom logo-area">
        <div class="grogu-pod-container mb-1" :class="{ 'mini-pod': isCollapsed }">
          <img src="/icon.png" alt="Jedi Logo" class="app-logo" />
        </div>

        <!-- Title/Subtitle -->
        <div v-if="!isCollapsed" class="text-center px-2 fade-transition" style="max-width: 100%;">
          <h2 class="text-h6 font-weight-bold text-primary" style="font-size: 1rem !important;">{{ $t('sidebar.title') }}</h2>
          <div class="text-caption text-secondary" style="font-size: 0.7rem !important;">{{ $t('sidebar.subtitle') }}</div>
        </div>
      </div>

      <!-- Navigation -->
      <v-list nav class="pa-2 mt-2 flex-grow-1">
        <v-list-item
          to="/chat"
          rounded="lg"
          class="mb-2 sidebar-item"
          color="primary"
          active-class="v-list-item--active"
          :class="{ 'justify-center': isCollapsed }"
        >
          <template v-slot:prepend>
            <div class="sidebar-icon-container" :class="{ 'mr-3': !isCollapsed }">
              <v-icon :icon="mdiRobot" size="20"></v-icon>
            </div>
          </template>
          <v-list-item-title v-if="!isCollapsed" class="font-weight-medium">Chat</v-list-item-title>
          <v-tooltip v-if="isCollapsed" activator="parent" location="right">Chat</v-tooltip>
        </v-list-item>

        <v-list-item
          to="/hosts"
          rounded="lg"
          class="mb-2 sidebar-item"
          color="primary"
          active-class="v-list-item--active"
          :class="{ 'justify-center': isCollapsed }"
        >
          <template v-slot:prepend>
            <div class="sidebar-icon-container" :class="{ 'mr-3': !isCollapsed }">
              <v-icon :icon="mdiDns" size="20"></v-icon>
            </div>
          </template>
          <v-list-item-title v-if="!isCollapsed" class="font-weight-medium">{{ $t('sidebar.hostsManager') }}</v-list-item-title>
          <v-tooltip v-if="isCollapsed" activator="parent" location="right">{{ $t('sidebar.hostsManager') }}</v-tooltip>
        </v-list-item>

        <v-list-item
          to="/wallpapers"
          rounded="lg"
          class="mb-2 sidebar-item"
          color="primary"
          active-class="v-list-item--active"
          :class="{ 'justify-center': isCollapsed }"
        >
          <template v-slot:prepend>
            <div class="sidebar-icon-container" :class="{ 'mr-3': !isCollapsed }">
              <v-icon :icon="mdiWallpaper" size="20"></v-icon>
            </div>
          </template>
          <v-list-item-title v-if="!isCollapsed" class="font-weight-medium">{{ $t('sidebar.wallpapers') }}</v-list-item-title>
          <v-tooltip v-if="isCollapsed" activator="parent" location="right">{{ $t('sidebar.wallpapers') }}</v-tooltip>
        </v-list-item>

        <v-list-item
          to="/podcast"
          rounded="lg"
          class="mb-2 sidebar-item"
          color="primary"
          active-class="v-list-item--active"
          :class="{ 'justify-center': isCollapsed }"
        >
          <template v-slot:prepend>
            <div class="sidebar-icon-container" :class="{ 'mr-3': !isCollapsed }">
              <v-icon :icon="mdiPodcast" size="20"></v-icon>
            </div>
          </template>
          <v-list-item-title v-if="!isCollapsed" class="font-weight-medium">{{ $t('sidebar.podcast') }}</v-list-item-title>
          <v-tooltip v-if="isCollapsed" activator="parent" location="right">{{ $t('sidebar.podcast') }}</v-tooltip>
        </v-list-item>
      </v-list>

      <!-- Footer Area: Jedi Control Deck -->
      <!-- Removed as controls moved to Top Header -->
    </aside>

    <!-- Resize Handle -->
    <div
      class="resize-handle"
      :class="{ 'is-resizing': isResizing }"
      @mousedown="startResize"
    >
      <div class="resize-indicator"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  mdiDns, mdiWallpaper, mdiPodcast, mdiRobot, mdiCog, mdiGithub,
  mdiWeatherNight, mdiWeatherSunny, mdiThemeLightDark,
  mdiPlay, mdiPause, mdiRewind15, mdiFastForward15, mdiStop
} from '@mdi/js'
import { useTheme } from '@/composables/useTheme'
import { useAudioPlayer } from '@/composables/useAudioPlayer'

const props = defineProps<{
  collapsed?: boolean
}>()

const emit = defineEmits<{
  (e: 'show-settings'): void
  (e: 'open-github'): void
  (e: 'update:width', width: number): void
  (e: 'toggle-sidebar'): void
}>()

const { t } = useI18n()
const { isDark, themeMode, setTheme } = useTheme()
const { currentPlaying, isPaused, currentTime, duration, togglePlay, seek, stop } = useAudioPlayer()

// Sidebar width and resize
const width = ref(180)
const minWidth = 64 // Collapsed icon width
const expandThreshold = 140
const maxWidth = 180 // Default recommended width as max
const isResizing = ref(false)
const isCollapsed = ref(false)

let startX = 0
let startWidth = 0

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

// Watch for external collapse prop changes
watch(() => props.collapsed, (newVal) => {
  if (newVal !== undefined) {
    isCollapsed.value = newVal
  }
})

// Resize functions
function startResize(e: MouseEvent) {
  isResizing.value = true
  startX = e.clientX
  startWidth = isCollapsed.value ? minWidth : width.value
  document.addEventListener('mousemove', doResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'ew-resize'
  document.body.style.userSelect = 'none'
  document.body.classList.add('no-transition') // Add global no-transition
}

function doResize(e: MouseEvent) {
  if (!isResizing.value) return
  const delta = e.clientX - startX
  let newWidth = startWidth + delta

  // Logic: 
  // 1. If width < expandThreshold, snap to minWidth (collapse)
  // 2. If width > expandThreshold, allow resize up to maxWidth
  
  if (newWidth < expandThreshold) {
    isCollapsed.value = true
    newWidth = minWidth
  } else {
    isCollapsed.value = false
    newWidth = Math.min(maxWidth, Math.max(expandThreshold, newWidth))
    width.value = newWidth
  }
  
  emit('update:width', newWidth)
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', doResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  document.body.classList.remove('no-transition') // Remove global no-transition
  saveWidth(width.value)
}

onMounted(() => {
  // Load saved width from localStorage if available
  const savedWidth = localStorage.getItem('jedi-sidebar-width')
  if (savedWidth) {
    const parsed = parseInt(savedWidth, 10)
    if (!isNaN(parsed) && parsed >= expandThreshold && parsed <= maxWidth) {
      width.value = parsed
      isCollapsed.value = false
    } else {
      isCollapsed.value = true
    }
  }
})

// Watch width changes and save to localStorage
function saveWidth(newWidth: number) {
  localStorage.setItem('jedi-sidebar-width', newWidth.toString())
}

onUnmounted(() => {
  document.removeEventListener('mousemove', doResize)
  document.removeEventListener('mouseup', stopResize)
})
</script>

<style scoped>
.sidebar-wrapper {
  position: relative;
  height: 100%;
  transition: width 0.2s ease;
  overflow: visible; /* Allow resize handle to be visible/grabbable */
  flex-shrink: 0;
  z-index: 90; /* High but below header/footer if needed, but here we want it to be distinct */
}

.jedi-sidebar {
  border-right: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  background: linear-gradient(
    180deg,
    rgba(var(--v-theme-surface), 1) 0%,
    rgba(var(--v-theme-surface), 0.98) 100%
  );
  height: 100%;
  overflow-x: hidden;
  overflow-y: auto;
}

/* Custom Scrollbar for Sidebar */
.jedi-sidebar::-webkit-scrollbar {
  width: 4px;
}
.jedi-sidebar::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-on-surface), 0.1);
  border-radius: 2px;
}

.grogu-pod-container {
  position: relative;
  width: 80px;
  height: 80px;
  display: flex;
  justify-content: center;
  align-items: center;
  transition: all 0.3s ease;
}

.grogu-pod-container.mini-pod {
  width: 48px;
  height: 48px;
  margin-bottom: 4px !important;
}

.app-logo {
  width: 64px;
  height: 64px;
  object-fit: contain;
  transition: all 0.3s ease;
  filter: drop-shadow(0 4px 6px rgba(0,0,0,0.1));
  z-index: 2;
  border-radius: 50%; /* Ensure round for rotation */
}

.mini-pod .app-logo {
  width: 40px;
  height: 40px;
}

.logo-spin {
  animation: spin 20s linear infinite;
}

/* Visualizer Animation */
.visualizer-ring {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 140%;
  height: 140%;
  transform: translate(-50%, -50%);
  border-radius: 50%;
  border: 2px dashed rgba(var(--v-theme-primary), 0.3);
  pointer-events: none;
  animation: visualizer-spin 12s linear infinite;
}

.visualizer-ring::after {
  content: '';
  position: absolute;
  inset: -15%;
  border-radius: 50%;
  border: 2px dotted rgba(var(--v-theme-primary), 0.15);
  animation: visualizer-spin 18s linear infinite reverse;
}

@keyframes visualizer-spin {
  from { transform: translate(-50%, -50%) rotate(0deg); }
  to { transform: translate(-50%, -50%) rotate(360deg); }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Ensure icons center in collapsed mode */
.sidebar-item.justify-center :deep(.v-list-item__content) {
  display: flex;
  justify-content: center;
  margin-left: 0 !important; /* Reset margin */
  width: 100%; /* Ensure full width for centering */
}

.sidebar-item.justify-center :deep(.v-list-item__prepend) {
  margin-inline-end: 0 !important;
  display: flex;
  justify-content: center;
  width: 100%;
}

.sidebar-item.justify-center .sidebar-icon-container {
  margin-right: 0 !important; /* Force remove margin */
}

/* Resize Handle */
.resize-handle {
  position: absolute;
  top: 0;
  right: 0;
  width: 6px;
  height: 100%;
  cursor: ew-resize;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s ease;
}

.resize-handle:hover {
  background-color: rgba(var(--v-theme-primary), 0.1);
}

.resize-handle.is-resizing {
  background-color: rgba(var(--v-theme-primary), 0.15);
}

.resize-indicator {
  width: 2px;
  height: 32px;
  border-radius: 2px;
  background-color: rgba(var(--v-theme-on-surface), 0.15);
  transition: background-color 0.15s ease;
}

.resize-handle:hover .resize-indicator,
.resize-handle.is-resizing .resize-indicator {
  background-color: rgb(var(--v-theme-primary));
  width: 3px;
}
</style>
