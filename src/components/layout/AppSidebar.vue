<template>
    <div
        class="sidebar-wrapper"
        :class="{ 'sidebar-collapsed': isCollapsed, 'is-resizing': isResizing }"
        :style="{ width: isCollapsed ? '64px' : `${width}px` }"
    >
        <aside
            id="app-navigation"
            class="jedi-sidebar d-flex flex-column"
            :class="{ 'sidebar-collapsed': isCollapsed }"
            style="width: 100%"
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
            role="separator"
            tabindex="0"
            aria-orientation="vertical"
            aria-controls="app-navigation"
            :aria-label="$t('sidebar.resize')"
            :aria-valuemin="collapsedWidth"
            :aria-valuemax="maxWidth"
            :aria-valuenow="isCollapsed ? collapsedWidth : width"
            :title="$t('sidebar.resizeHint')"
            data-tauri-no-drag
            @pointerdown="startResize"
            @pointermove="doResize"
            @pointerup="stopResize"
            @pointercancel="stopResize"
            @lostpointercapture="stopResize"
            @dblclick="resetWidth"
            @keydown="resizeWithKeyboard"
        >
            <div class="resize-indicator"></div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { mdiCrop, mdiDns, mdiPodcast, mdiRobot, mdiToolboxOutline, mdiWallpaper } from '@mdi/js'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'

const props = defineProps<{
  collapsed?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:width', width: number): void
  (e: 'update:collapsed', collapsed: boolean): void
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
  { to: '/screenshot', icon: mdiCrop, tooltipKey: 'sidebar.screenshot' },
]

// Width changes are immediate while dragging; collapse remains an explicit action.
const defaultWidth = 220
const collapsedWidth = 64
const minWidth = 176
const maxWidth = 320
const width = ref(defaultWidth)
const isResizing = ref(false)
const isCollapsed = computed({
  get: () => props.collapsed ?? false,
  set: value => emit('update:collapsed', value),
})
const currentTime = ref('')
let startX = 0
let startWidth = 0
let dragHandle: HTMLElement | null = null
let activePointer: number | null = null
let timeInterval: number | null = null

watch(
  () => props.collapsed,
  collapsed => {
    if (collapsed) stopResize()
  }
)

function startResize(event: PointerEvent) {
  if (event.button !== 0 || isResizing.value) return
  event.preventDefault()
  dragHandle = event.currentTarget as HTMLElement
  dragHandle.focus({ preventScroll: true })
  dragHandle.setPointerCapture(event.pointerId)
  activePointer = event.pointerId
  startX = event.clientX
  startWidth = isCollapsed.value ? collapsedWidth : width.value
  isResizing.value = true
  document.body.classList.add('sidebar-resizing')
}

function doResize(event: PointerEvent) {
  if (!isResizing.value || event.pointerId !== activePointer) return
  const delta = event.clientX - startX
  // A collapsed rail only opens after a deliberate outward drag.
  if (isCollapsed.value && delta <= 12) return
  if (isCollapsed.value) {
    isCollapsed.value = false
    startX = event.clientX
    startWidth = minWidth
  }
  width.value = Math.round(
    Math.min(maxWidth, Math.max(minWidth, startWidth + event.clientX - startX))
  )
  emit('update:width', width.value)
}

function stopResize() {
  if (!isResizing.value) return
  isResizing.value = false
  const handle = dragHandle
  const pointer = activePointer
  dragHandle = null
  activePointer = null
  if (pointer !== null && handle?.hasPointerCapture(pointer)) handle.releasePointerCapture(pointer)
  document.body.classList.remove('sidebar-resizing')
  saveWidth()
}

function resetWidth() {
  stopResize()
  width.value = defaultWidth
  isCollapsed.value = false
  emit('update:width', width.value)
  saveWidth()
}

function resizeWithKeyboard(event: KeyboardEvent) {
  if (!['ArrowLeft', 'ArrowRight', 'Home', 'End', 'Enter'].includes(event.key)) return
  event.preventDefault()
  stopResize()
  if (event.key === 'Enter') {
    isCollapsed.value = !isCollapsed.value
    return
  }
  const step = event.shiftKey ? 32 : 8
  const next =
    event.key === 'Home'
      ? minWidth
      : event.key === 'End'
        ? maxWidth
        : (isCollapsed.value ? minWidth : width.value) + (event.key === 'ArrowRight' ? step : -step)
  width.value = Math.min(maxWidth, Math.max(minWidth, next))
  isCollapsed.value = false
  emit('update:width', width.value)
  saveWidth()
}

function updateTime() {
  const now = new Date()
  currentTime.value = now.toLocaleTimeString('en-US', { hour12: false })
}

onMounted(() => {
  try {
    const savedWidth = Number(localStorage.getItem('jedi-sidebar-width'))
    if (Number.isFinite(savedWidth) && savedWidth > collapsedWidth) {
      width.value = Math.min(maxWidth, Math.max(minWidth, savedWidth))
    }
  } catch {
    /* Storage may be unavailable; keep the default width. */
  }
  window.addEventListener('blur', stopResize)
  updateTime()
  timeInterval = window.setInterval(updateTime, 1000)
})

function saveWidth() {
  try {
    localStorage.setItem('jedi-sidebar-width', width.value.toString())
  } catch {
    /* Resizing remains usable when storage is unavailable. */
  }
}

onUnmounted(() => {
  stopResize()
  window.removeEventListener('blur', stopResize)
  if (timeInterval) {
    clearInterval(timeInterval)
  }
})
</script>

<style scoped>
.sidebar-wrapper {
    position: relative;
    height: 100%;
    transition: width var(--motion-normal) var(--ease-out);
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

/* Wide hit target, quiet hairline and a visible grip in both themes. */
.sidebar-wrapper.is-resizing { transition: none; }
:global(body.sidebar-resizing), :global(body.sidebar-resizing *) {
    cursor: col-resize !important;
    user-select: none !important;
}
.resize-handle {
    position: absolute;
    inset: 0 -6px 0 auto;
    width: 12px;
    cursor: col-resize;
    z-index: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    touch-action: none;
    outline: none;
}
.resize-handle::before {
    content: '';
    position: absolute;
    inset: 12px 5px;
    border-radius: 2px;
    background: var(--border);
    transition: background var(--motion-fast), box-shadow var(--motion-fast);
}
.resize-indicator {
    position: relative;
    width: 4px;
    height: 32px;
    border-radius: 4px;
    background: var(--text-subtle);
    opacity: .45;
    transition: height var(--motion-fast), background var(--motion-fast), opacity var(--motion-fast);
}
.resize-handle:is(:hover, :focus-visible, .is-resizing)::before {
    background: rgb(var(--accent-rgb) / .45);
    box-shadow: 0 0 0 3px rgb(var(--accent-rgb) / .06);
}
.resize-handle:is(:hover, :focus-visible, .is-resizing) .resize-indicator {
    height: 48px;
    opacity: 1;
    background: var(--accent);
}
.resize-handle:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; border-radius: 6px; }
@media (prefers-reduced-motion: reduce) {
    .sidebar-wrapper, .resize-handle::before, .resize-indicator { transition: none; }
}
</style>
