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
            <div class="d-flex flex-column align-center logo-area">
                <div
                    class="grogu-pod-container mb-1"
                    :class="{ 'mini-pod': isCollapsed }"
                >
                    <img :src="logoSrc" alt="Jedi Logo" class="app-logo" />
                </div>

                <!-- Title/Subtitle -->
                <div
                    v-if="!isCollapsed"
                    class="text-center px-2 fade-transition"
                    style="max-width: 100%"
                >
                    <h2 class="text-h6 font-weight-bold sidebar-title">
                        {{ $t("sidebar.title") }}
                    </h2>
                    <div class="text-caption sidebar-subtitle">
                        {{ $t("sidebar.subtitle") }}
                    </div>
                </div>
            </div>

            <!-- Navigation -->
            <v-list nav class="pa-2 mt-2 flex-grow-1">
                <v-list-item
                    v-for="item in navItems"
                    :key="item.to"
                    :to="item.to"
                    rounded="lg"
                    class="mb-2 sidebar-item"
                    color="primary"
                    active-class="v-list-item--active"
                    :slim="isCollapsed"
                    :class="{ 'justify-center': isCollapsed }"
                >
                    <template v-slot:prepend>
                        <div
                            class="sidebar-icon-container"
                            :class="{ 'mr-3': !isCollapsed }"
                        >
                            <v-icon :icon="item.icon" size="20"></v-icon>
                        </div>
                    </template>
                    <v-list-item-title
                        v-if="!isCollapsed"
                        class="font-weight-medium nav-text"
                        >{{ $t(item.tooltipKey) }}</v-list-item-title
                    >
                    <v-tooltip
                        v-if="isCollapsed"
                        activator="parent"
                        location="right"
                        >{{ $t(item.tooltipKey) }}</v-tooltip
                    >
                </v-list-item>
            </v-list>

            <!-- Footer Clock -->
            <div v-if="!isCollapsed" class="sidebar-footer">
                <div class="footer-time">{{ currentTime }}</div>
            </div>
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
import { mdiDns, mdiPodcast, mdiRobot, mdiToolboxOutline, mdiWallpaper } from '@mdi/js'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'

const props = defineProps<{
  collapsed?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:width', width: number): void
  (e: 'toggle-sidebar'): void
}>()

// Dark theme uses the black-tile koala variant
const { isDark } = useTheme()

const logoSrc = computed(() =>
  isDark.value
    ? '/branding/jedi-app-icon-dark.png?v=jedi-koala-20261003'
    : '/icon.png?v=jedi-koala-20261003'
)

const navItems = [
  { to: '/chat', icon: mdiRobot, tooltipKey: 'sidebar.chat' },
  { to: '/skills', icon: mdiToolboxOutline, tooltipKey: 'sidebar.skills' },
  { to: '/hosts', icon: mdiDns, tooltipKey: 'sidebar.hostsManager' },
  { to: '/wallpapers', icon: mdiWallpaper, tooltipKey: 'sidebar.wallpapers' },
  { to: '/podcast', icon: mdiPodcast, tooltipKey: 'sidebar.podcast' },
]

// Sidebar width and resize
const width = ref(200)
const minWidth = 64
const expandThreshold = 150
const maxWidth = 220
const isResizing = ref(false)
const isCollapsed = ref(false)
const currentTime = ref('')

let startX = 0
let startWidth = 0
let timeInterval: number | null = null

// Watch for external collapse prop changes
watch(
  () => props.collapsed,
  newVal => {
    if (newVal !== undefined) {
      isCollapsed.value = newVal
    }
  }
)

// Resize functions
function startResize(e: MouseEvent) {
  isResizing.value = true
  startX = e.clientX
  startWidth = isCollapsed.value ? minWidth : width.value
  document.addEventListener('mousemove', doResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'ew-resize'
  document.body.style.userSelect = 'none'
  document.body.classList.add('no-transition')
}

function doResize(e: MouseEvent) {
  if (!isResizing.value) return
  const delta = e.clientX - startX
  let newWidth = startWidth + delta

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
  document.body.classList.remove('no-transition')
  saveWidth(width.value)
}

function updateTime() {
  const now = new Date()
  currentTime.value = now.toLocaleTimeString('en-US', { hour12: false })
}

onMounted(() => {
  const savedWidth = localStorage.getItem('jedi-sidebar-width')
  if (savedWidth) {
    const parsed = parseInt(savedWidth, 10)
    if (!Number.isNaN(parsed) && parsed >= expandThreshold && parsed <= maxWidth) {
      width.value = parsed
      isCollapsed.value = false
    } else {
      isCollapsed.value = true
    }
  }
  updateTime()
  timeInterval = window.setInterval(updateTime, 1000)
})

function saveWidth(newWidth: number) {
  localStorage.setItem('jedi-sidebar-width', newWidth.toString())
}

onUnmounted(() => {
  document.removeEventListener('mousemove', doResize)
  document.removeEventListener('mouseup', stopResize)
  if (timeInterval) {
    clearInterval(timeInterval)
  }
})
</script>

<style scoped>
.sidebar-wrapper {
    position: relative;
    height: 100%;
    transition: width 0.2s ease;
    overflow: visible;
    flex-shrink: 0;
    z-index: 90;
}

.jedi-sidebar {
    border-right: 1px solid var(--border);
    background: var(--bg-sidebar);
    height: 100%;
    overflow-x: hidden;
    overflow-y: auto;
    font-family: var(--jedi-font-ui);
}

/* Custom Scrollbar for Sidebar */
.jedi-sidebar::-webkit-scrollbar {
    width: 4px;
}
.jedi-sidebar::-webkit-scrollbar-track {
    background: transparent;
}
.jedi-sidebar::-webkit-scrollbar-thumb {
    background: rgb(var(--text-rgb) / 0.2);
    border-radius: 2px;
}
.jedi-sidebar::-webkit-scrollbar-thumb:hover {
    background: rgb(var(--text-rgb) / 0.32);
}

.logo-area {
    border-bottom: 1px solid var(--border);
    padding: 18px 8px 14px !important;
}

/* Collapsed logo area */
.sidebar-collapsed .logo-area {
    padding: 12px 8px !important;
}

.grogu-pod-container {
    position: relative;
    width: 56px;
    height: 56px;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: all 0.3s ease;
}

.grogu-pod-container.mini-pod {
    width: 36px;
    height: 36px;
    margin-bottom: 4px !important;
}

.app-logo {
    width: 44px;
    height: 44px;
    object-fit: contain;
    transition: all 0.3s ease;
    z-index: 2;
    border-radius: 0;
    position: relative;
}

.mini-pod .app-logo {
    width: 28px;
    height: 28px;
}

.sidebar-title {
    font-size: 13px !important;
    font-weight: 600 !important;
    letter-spacing: 0.5px;
    color: var(--text) !important;
    font-family: var(--jedi-font-ui);
}

.sidebar-subtitle {
    font-size: 10px !important;
    color: var(--text-subtle) !important;
    letter-spacing: 0.5px;
}

/* Ensure icons center in collapsed mode */
.sidebar-item.justify-center :deep(.v-list-item) {
    justify-content: center !important;
    padding-inline: 0 !important;
    padding-left: 0 !important;
    padding-right: 0 !important;
    min-width: 0 !important;
}

.sidebar-item.justify-center :deep(.v-list-item__content) {
    display: none !important;
}

.sidebar-item.justify-center :deep(.v-list-item__spacer) {
    display: none !important;
    width: 0 !important;
    flex: none !important;
}

.sidebar-item.justify-center :deep(.v-list-item__prepend) {
    margin-inline-end: 0 !important;
    margin-inline-start: 0 !important;
    margin-left: auto !important;
    margin-right: auto !important;
    display: flex;
    justify-content: center;
    align-items: center;
    width: auto;
    flex: none !important;
}

.sidebar-item.justify-center .sidebar-icon-container {
    margin-right: 0 !important;
    margin-left: 0 !important;
    display: flex;
    justify-content: center;
    align-items: center;
}

/* Sidebar Item — quiet neutral highlight (ChatGPT/Claude style) */
.sidebar-item {
    transition: background 0.15s ease;
    position: relative;
    border-radius: 6px !important;
    overflow: hidden;
}

.sidebar-item :deep(.v-list-item) {
    border-radius: 6px !important;
    position: relative;
}

.sidebar-item.v-list-item--active :deep(.v-list-item) {
    background: rgb(var(--text-rgb) / 0.08) !important;
}

.sidebar-item:hover :deep(.v-list-item) {
    background: rgb(var(--text-rgb) / 0.05) !important;
}

.nav-text {
    font-size: 13px;
    font-weight: 500;
    letter-spacing: 0;
    color: var(--text-muted);
}

.sidebar-item.v-list-item--active .nav-text {
    color: var(--text);
}

/* Sidebar Footer */
.sidebar-footer {
    padding: 8px 12px 10px;
    border-top: 1px solid var(--border);
}

.footer-time {
    font-size: 10px;
    color: var(--text-subtle);
    font-family: var(--jedi-font-ui);
    text-align: center;
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
    background-color: rgb(var(--text-rgb) / 0.06);
}

.resize-handle.is-resizing {
    background-color: rgb(var(--text-rgb) / 0.1);
}

.resize-indicator {
    width: 2px;
    height: 32px;
    border-radius: 2px;
    background-color: rgb(var(--text-rgb) / 0.2);
    transition: background-color 0.15s ease;
}

.resize-handle:hover .resize-indicator,
.resize-handle.is-resizing .resize-indicator {
    background-color: var(--text-muted);
    width: 3px;
}
</style>
