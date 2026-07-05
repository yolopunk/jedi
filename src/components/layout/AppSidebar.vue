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
            <div
                class="d-flex flex-column align-center py-2 border-bottom logo-area"
            >
                <LogoShaderBg v-if="!isCollapsed" :is-collapsed="isCollapsed" />
                <div
                    class="grogu-pod-container mb-1"
                    :class="{ 'mini-pod': isCollapsed }"
                >
                    <img src="/icon.png" alt="Jedi Logo" class="app-logo" />
                    <div class="logo-glow"></div>
                </div>

                <!-- Title/Subtitle -->
                <div
                    v-if="!isCollapsed"
                    class="text-center px-2 fade-transition"
                    style="max-width: 100%"
                >
                    <h2 class="text-h6 font-weight-bold sidebar-title">
                        <span class="title-bracket">[</span>
                        {{ $t("sidebar.title") }}
                        <span class="title-bracket">]</span>
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
                        >{{ item.label }}</v-list-item-title
                    >
                    <v-tooltip
                        v-if="isCollapsed"
                        activator="parent"
                        location="right"
                        >{{ $t(item.tooltipKey) }}</v-tooltip
                    >
                    <div v-if="!isCollapsed" class="nav-indicator"></div>
                </v-list-item>
            </v-list>

            <!-- Footer Status -->
            <div v-if="!isCollapsed" class="sidebar-footer">
                <div class="hud-panel">
                    <div class="hud-scanline"></div>
                    <div class="hud-item">
                        <span class="hud-dot online"></span>
                        <span class="hud-label">SYS</span>
                        <v-tooltip activator="parent" location="top">{{
                            $t("header.systemOnline")
                        }}</v-tooltip>
                    </div>
                    <div class="hud-divider"></div>
                    <div class="hud-item">
                        <span class="hud-dot standby"></span>
                        <span class="hud-label">MOD</span>
                        <v-tooltip activator="parent" location="top">{{
                            $t("header.modulesActive")
                        }}</v-tooltip>
                    </div>
                    <div class="hud-divider"></div>
                    <div class="hud-item">
                        <span class="hud-dot scanning"></span>
                        <span class="hud-label">MON</span>
                        <v-tooltip activator="parent" location="top">{{
                            $t("header.monitoring")
                        }}</v-tooltip>
                    </div>
                </div>
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
import { mdiDns, mdiPodcast, mdiRobot, mdiWallpaper } from '@mdi/js'
import { onMounted, onUnmounted, ref, watch } from 'vue'
import LogoShaderBg from '@/components/common/LogoShaderBg.vue'

const props = defineProps<{
  collapsed?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:width', width: number): void
  (e: 'toggle-sidebar'): void
}>()

const navItems = [
  { to: '/chat', icon: mdiRobot, label: 'CHAT', tooltipKey: 'sidebar.chat' },
  { to: '/hosts', icon: mdiDns, label: 'HOSTS', tooltipKey: 'sidebar.hostsManager' },
  { to: '/wallpapers', icon: mdiWallpaper, label: 'WALLPAPER', tooltipKey: 'sidebar.wallpapers' },
  { to: '/podcast', icon: mdiPodcast, label: 'PODCAST', tooltipKey: 'sidebar.podcast' },
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
    border-right: 1px solid rgb(var(--accent-rgb) / 0.15);
    background: linear-gradient(180deg, var(--bg-terminal) 0%, var(--bg-terminal) 100%);
    height: 100%;
    overflow-x: hidden;
    overflow-y: auto;
    font-family: "JetBrains Mono", "Fira Code", "SF Mono", monospace;
}

/* Custom Scrollbar for Sidebar */
.jedi-sidebar::-webkit-scrollbar {
    width: 4px;
}
.jedi-sidebar::-webkit-scrollbar-track {
    background: transparent;
}
.jedi-sidebar::-webkit-scrollbar-thumb {
    background: rgb(var(--accent-rgb) / 0.2);
    border-radius: 2px;
}
.jedi-sidebar::-webkit-scrollbar-thumb:hover {
    background: rgb(var(--accent-rgb) / 0.3);
}

.logo-area {
    border-bottom-color: rgb(var(--accent-rgb) / 0.15) !important;
    padding: 16px 8px !important;
    position: relative;
    overflow: hidden;
    min-height: 160px;
}

/* Collapsed logo area */
.sidebar-collapsed .logo-area {
    min-height: auto;
    padding: 10px 8px !important;
    background: radial-gradient(
        circle at center,
        rgb(var(--accent-rgb) / 0.06) 0%,
        transparent 70%
    );
}

.grogu-pod-container {
    position: relative;
    width: 72px;
    height: 72px;
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

.logo-glow {
    position: absolute;
    inset: -4px;
    border-radius: 50%;
    background: radial-gradient(
        circle,
        rgb(var(--accent-rgb) / 0.3) 0%,
        transparent 70%
    );
    animation: logoPulse 2s ease-in-out infinite;
    z-index: 0;
}

@keyframes logoPulse {
    0%,
    100% {
        opacity: 0.5;
        transform: scale(1);
    }
    50% {
        opacity: 1;
        transform: scale(1.1);
    }
}

.app-logo {
    width: 56px;
    height: 56px;
    object-fit: contain;
    transition: all 0.3s ease;
    z-index: 2;
    border-radius: 50%;
    position: relative;
}

.mini-pod .app-logo {
    width: 40px;
    height: 40px;
}

.sidebar-title {
    font-size: 13px !important;
    font-weight: 700 !important;
    letter-spacing: 2px;
    color: var(--success) !important;
    text-shadow: 0 0 10px rgb(var(--success-rgb) / 0.5);
}

.title-bracket {
    color: var(--accent);
    font-size: 12px;
    text-shadow: 0 0 10px rgb(var(--accent-rgb) / 0.5);
}

.sidebar-subtitle {
    font-size: 9px !important;
    color: var(--text-subtle) !important;
    letter-spacing: 1px;
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

/* Sci-Fi Console Sidebar Item */
.sidebar-item {
    transition: all 0.15s ease;
    position: relative;
    border-radius: 4px !important;
    overflow: hidden;
}

.sidebar-item :deep(.v-list-item) {
    border-radius: 4px !important;
    position: relative;
}

.sidebar-item::before {
    content: "";
    position: absolute;
    left: 0;
    top: 4px;
    bottom: 4px;
    width: 2px;
    background: var(--accent);
    opacity: 0;
    transform: scaleY(0);
    transition: all 0.15s ease;
    box-shadow: 0 0 10px rgb(var(--accent-rgb) / 0.8);
    z-index: 2;
}

.sidebar-item.v-list-item--active::before {
    opacity: 1;
    transform: scaleY(1);
}

.sidebar-item.v-list-item--active :deep(.v-list-item) {
    background: rgb(var(--accent-rgb) / 0.08) !important;
}

.sidebar-item.v-list-item--active .sidebar-icon-container {
    filter: drop-shadow(0 0 6px rgb(var(--accent-rgb) / 0.8));
}

.sidebar-item:hover :deep(.v-list-item) {
    background: rgb(var(--accent-rgb) / 0.05) !important;
}

.nav-text {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 1px;
    color: var(--text-muted);
}

.sidebar-item.v-list-item--active .nav-text {
    color: var(--accent);
    text-shadow: 0 0 8px rgb(var(--accent-rgb) / 0.5);
}

.nav-indicator {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--success);
    box-shadow: 0 0 8px rgb(var(--success-rgb) / 0.6);
    margin-left: auto;
    opacity: 0;
    transition: opacity 0.2s ease;
}

.sidebar-item.v-list-item--active .nav-indicator {
    opacity: 1;
}

/* Sidebar Footer */
.sidebar-footer {
    padding: 8px 12px 10px;
    border-top: 1px solid rgb(var(--accent-rgb) / 0.15);
    background: linear-gradient(0deg, var(--bg-terminal) 0%, transparent 100%);
}

/* Footer Action Buttons */
.footer-actions {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    margin-bottom: 6px;
}

.footer-action-btn {
    width: 28px !important;
    height: 28px !important;
    min-width: 28px !important;
    border-radius: 4px;
    color: rgb(var(--accent-rgb) / 0.5);
    opacity: 0.7;
    transition: all 0.15s ease;
}

.footer-action-btn:hover {
    background-color: rgb(var(--accent-rgb) / 0.1);
    opacity: 1;
    color: var(--accent) !important;
}

/* Collapsed Action Icons */
.collapsed-actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 0;
    border-top: 1px solid rgb(var(--accent-rgb) / 0.15);
}

.collapsed-action-btn {
    width: 32px !important;
    height: 32px !important;
    min-width: 32px !important;
    border-radius: 4px;
    color: rgb(var(--accent-rgb) / 0.5);
    opacity: 0.7;
    transition: all 0.15s ease;
}

.collapsed-action-btn:hover {
    background-color: rgb(var(--accent-rgb) / 0.1);
    opacity: 1;
    color: var(--accent) !important;
}

/* HUD Status Panel */
.hud-panel {
    display: flex;
    align-items: center;
    gap: 0;
    height: 22px;
    padding: 0 8px;
    background: rgb(var(--accent-rgb) / 0.04);
    border: 1px solid rgb(var(--accent-rgb) / 0.2);
    border-radius: 3px;
    position: relative;
    overflow: hidden;
    font-size: 9px;
    letter-spacing: 1px;
    margin-bottom: 6px;
}

.hud-scanline {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: repeating-linear-gradient(
        0deg,
        transparent,
        transparent 2px,
        rgb(var(--accent-rgb) / 0.03) 2px,
        rgb(var(--accent-rgb) / 0.03) 4px
    );
    pointer-events: none;
    animation: scanline-move 3s linear infinite;
}

@keyframes scanline-move {
    0% {
        background-position: 0 0;
    }
    100% {
        background-position: 0 4px;
    }
}

.hud-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 5px;
}

.hud-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
}

.hud-dot.online {
    background: var(--success);
    box-shadow: 0 0 4px var(--success);
}

.hud-dot.standby {
    background: var(--warning);
    box-shadow: 0 0 4px var(--warning);
}

.hud-dot.scanning {
    background: var(--accent);
    box-shadow: 0 0 4px var(--accent);
    animation: hud-blink 1.5s ease-in-out infinite;
}

@keyframes hud-blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
}

.hud-label {
    color: rgb(var(--accent-rgb) / 0.5);
    font-family: "JetBrains Mono", "Fira Code", "SF Mono", monospace;
    font-weight: 500;
    white-space: nowrap;
}

.hud-divider {
    width: 1px;
    height: 12px;
    background: rgb(var(--accent-rgb) / 0.15);
    flex-shrink: 0;
}

.footer-time {
    font-size: 10px;
    color: var(--accent);
    font-family: "JetBrains Mono", monospace;
    text-shadow: 0 0 6px rgb(var(--accent-rgb) / 0.4);
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
    background-color: rgb(var(--accent-rgb) / 0.1);
}

.resize-handle.is-resizing {
    background-color: rgb(var(--accent-rgb) / 0.15);
}

.resize-indicator {
    width: 2px;
    height: 32px;
    border-radius: 2px;
    background-color: rgb(var(--accent-rgb) / 0.2);
    transition: background-color 0.15s ease;
}

.resize-handle:hover .resize-indicator,
.resize-handle.is-resizing .resize-indicator {
    background-color: var(--accent);
    width: 3px;
    box-shadow: 0 0 10px rgb(var(--accent-rgb) / 0.5);
}
</style>
