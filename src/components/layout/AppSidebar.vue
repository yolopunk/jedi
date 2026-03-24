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
                    to="/chat"
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
                            <v-icon :icon="mdiRobot" size="20"></v-icon>
                        </div>
                    </template>
                    <v-list-item-title
                        v-if="!isCollapsed"
                        class="font-weight-medium nav-text"
                        >CHAT</v-list-item-title
                    >
                    <v-tooltip
                        v-if="isCollapsed"
                        activator="parent"
                        location="right"
                        >{{ $t("sidebar.chat") }}</v-tooltip
                    >
                    <div v-if="!isCollapsed" class="nav-indicator"></div>
                </v-list-item>

                <v-list-item
                    to="/hosts"
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
                            <v-icon :icon="mdiDns" size="20"></v-icon>
                        </div>
                    </template>
                    <v-list-item-title
                        v-if="!isCollapsed"
                        class="font-weight-medium nav-text"
                        >HOSTS</v-list-item-title
                    >
                    <v-tooltip
                        v-if="isCollapsed"
                        activator="parent"
                        location="right"
                        >{{ $t("sidebar.hostsManager") }}</v-tooltip
                    >
                    <div v-if="!isCollapsed" class="nav-indicator"></div>
                </v-list-item>

                <v-list-item
                    to="/wallpapers"
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
                            <v-icon :icon="mdiWallpaper" size="20"></v-icon>
                        </div>
                    </template>
                    <v-list-item-title
                        v-if="!isCollapsed"
                        class="font-weight-medium nav-text"
                        >WALLPAPER</v-list-item-title
                    >
                    <v-tooltip
                        v-if="isCollapsed"
                        activator="parent"
                        location="right"
                        >{{ $t("sidebar.wallpapers") }}</v-tooltip
                    >
                    <div v-if="!isCollapsed" class="nav-indicator"></div>
                </v-list-item>

                <v-list-item
                    to="/podcast"
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
                            <v-icon :icon="mdiPodcast" size="20"></v-icon>
                        </div>
                    </template>
                    <v-list-item-title
                        v-if="!isCollapsed"
                        class="font-weight-medium nav-text"
                        >PODCAST</v-list-item-title
                    >
                    <v-tooltip
                        v-if="isCollapsed"
                        activator="parent"
                        location="right"
                        >{{ $t("sidebar.podcast") }}</v-tooltip
                    >
                    <div v-if="!isCollapsed" class="nav-indicator"></div>
                </v-list-item>
            </v-list>

            <!-- Footer Status -->
            <div v-if="!isCollapsed" class="sidebar-footer">
                <div class="footer-status">
                    <span class="status-label"
                        >{{ $t("sidebar.system") }}:</span
                    >
                    <span class="status-value online">{{
                        $t("sidebar.online")
                    }}</span>
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
import { ref, onMounted, onUnmounted, watch } from "vue";
import { mdiDns, mdiWallpaper, mdiPodcast, mdiRobot } from "@mdi/js";

const props = defineProps<{
    collapsed?: boolean;
}>();

const emit = defineEmits<{
    (e: "show-settings"): void;
    (e: "open-github"): void;
    (e: "update:width", width: number): void;
    (e: "toggle-sidebar"): void;
}>();

// Sidebar width and resize
const width = ref(200);
const minWidth = 64;
const expandThreshold = 150;
const maxWidth = 220;
const isResizing = ref(false);
const isCollapsed = ref(false);
const currentTime = ref("");

let startX = 0;
let startWidth = 0;
let timeInterval: number | null = null;

// Watch for external collapse prop changes
watch(
    () => props.collapsed,
    (newVal) => {
        if (newVal !== undefined) {
            isCollapsed.value = newVal;
        }
    },
);

// Resize functions
function startResize(e: MouseEvent) {
    isResizing.value = true;
    startX = e.clientX;
    startWidth = isCollapsed.value ? minWidth : width.value;
    document.addEventListener("mousemove", doResize);
    document.addEventListener("mouseup", stopResize);
    document.body.style.cursor = "ew-resize";
    document.body.style.userSelect = "none";
    document.body.classList.add("no-transition");
}

function doResize(e: MouseEvent) {
    if (!isResizing.value) return;
    const delta = e.clientX - startX;
    let newWidth = startWidth + delta;

    if (newWidth < expandThreshold) {
        isCollapsed.value = true;
        newWidth = minWidth;
    } else {
        isCollapsed.value = false;
        newWidth = Math.min(maxWidth, Math.max(expandThreshold, newWidth));
        width.value = newWidth;
    }

    emit("update:width", newWidth);
}

function stopResize() {
    isResizing.value = false;
    document.removeEventListener("mousemove", doResize);
    document.removeEventListener("mouseup", stopResize);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    document.body.classList.remove("no-transition");
    saveWidth(width.value);
}

function updateTime() {
    const now = new Date();
    currentTime.value = now.toLocaleTimeString("en-US", { hour12: false });
}

onMounted(() => {
    const savedWidth = localStorage.getItem("jedi-sidebar-width");
    if (savedWidth) {
        const parsed = parseInt(savedWidth, 10);
        if (!isNaN(parsed) && parsed >= expandThreshold && parsed <= maxWidth) {
            width.value = parsed;
            isCollapsed.value = false;
        } else {
            isCollapsed.value = true;
        }
    }
    updateTime();
    timeInterval = window.setInterval(updateTime, 1000);
});

function saveWidth(newWidth: number) {
    localStorage.setItem("jedi-sidebar-width", newWidth.toString());
}

onUnmounted(() => {
    document.removeEventListener("mousemove", doResize);
    document.removeEventListener("mouseup", stopResize);
    if (timeInterval) {
        clearInterval(timeInterval);
    }
});
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
    border-right: 1px solid rgba(0, 255, 255, 0.15);
    background: linear-gradient(180deg, #0d0d12 0%, #0a0a0f 100%);
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
    background: rgba(0, 255, 255, 0.2);
    border-radius: 2px;
}
.jedi-sidebar::-webkit-scrollbar-thumb:hover {
    background: rgba(0, 255, 255, 0.3);
}

.logo-area {
    border-bottom-color: rgba(0, 255, 255, 0.15) !important;
    padding: 16px 8px !important;
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
        rgba(0, 255, 255, 0.3) 0%,
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
    color: #00ff88 !important;
    text-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
}

.title-bracket {
    color: #00ffff;
    font-size: 12px;
    text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

.sidebar-subtitle {
    font-size: 9px !important;
    color: #52525b !important;
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
    background: #00ffff;
    opacity: 0;
    transform: scaleY(0);
    transition: all 0.15s ease;
    box-shadow: 0 0 10px rgba(0, 255, 255, 0.8);
    z-index: 2;
}

.sidebar-item.v-list-item--active::before {
    opacity: 1;
    transform: scaleY(1);
}

.sidebar-item.v-list-item--active :deep(.v-list-item) {
    background: rgba(0, 255, 255, 0.08) !important;
}

.sidebar-item.v-list-item--active .sidebar-icon-container {
    filter: drop-shadow(0 0 6px rgba(0, 255, 255, 0.8));
}

.sidebar-item:hover :deep(.v-list-item) {
    background: rgba(0, 255, 255, 0.05) !important;
}

.nav-text {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 1px;
    color: #a1a1aa;
}

.sidebar-item.v-list-item--active .nav-text {
    color: #00ffff;
    text-shadow: 0 0 8px rgba(0, 255, 255, 0.5);
}

.nav-indicator {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: #00ff88;
    box-shadow: 0 0 8px rgba(0, 255, 136, 0.6);
    margin-left: auto;
    opacity: 0;
    transition: opacity 0.2s ease;
}

.sidebar-item.v-list-item--active .nav-indicator {
    opacity: 1;
}

/* Sidebar Footer */
.sidebar-footer {
    padding: 12px 16px;
    border-top: 1px solid rgba(0, 255, 255, 0.15);
    background: linear-gradient(0deg, #0f0f1a 0%, transparent 100%);
}

.footer-status {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
}

.status-label {
    font-size: 9px;
    color: #52525b;
    letter-spacing: 1px;
}

.status-value {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 1px;
}

.status-value.online {
    color: #00ff88;
    text-shadow: 0 0 8px rgba(0, 255, 136, 0.5);
}

.footer-time {
    font-size: 10px;
    color: #00ffff;
    font-family: "JetBrains Mono", monospace;
    text-shadow: 0 0 6px rgba(0, 255, 255, 0.4);
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
    background-color: rgba(0, 255, 255, 0.1);
}

.resize-handle.is-resizing {
    background-color: rgba(0, 255, 255, 0.15);
}

.resize-indicator {
    width: 2px;
    height: 32px;
    border-radius: 2px;
    background-color: rgba(0, 255, 255, 0.2);
    transition: background-color 0.15s ease;
}

.resize-handle:hover .resize-indicator,
.resize-handle.is-resizing .resize-indicator {
    background-color: #00ffff;
    width: 3px;
    box-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

/* =========================================
   Light Theme Styles (Tatooine Outpost)
   ========================================= */
.light-theme .jedi-sidebar {
    background: linear-gradient(180deg, #f5e6d3 0%, #efe0cc 100%);
    border-right-color: rgba(184, 134, 11, 0.3);
}

.light-theme .logo-area {
    border-bottom-color: rgba(184, 134, 11, 0.3) !important;
}

.light-theme .logo-glow {
    background: radial-gradient(
        circle,
        rgba(205, 127, 50, 0.2) 0%,
        transparent 70%
    );
}

.light-theme .sidebar-title {
    color: #cd7f32 !important;
    text-shadow: 0 0 8px rgba(205, 127, 50, 0.3);
}

.light-theme .title-bracket {
    color: #6b4423;
}

.light-theme .sidebar-subtitle {
    color: #8b7355 !important;
}

.light-theme .sidebar-item::before {
    background: #cd7f32;
    box-shadow: 0 0 8px rgba(205, 127, 50, 0.3);
}

.light-theme .sidebar-item.v-list-item--active::before {
    opacity: 1;
    transform: scaleY(1);
}

.light-theme .sidebar-item.v-list-item--active .sidebar-icon-container {
    filter: drop-shadow(0 0 6px rgba(205, 127, 50, 0.4));
}

.light-theme .sidebar-item:hover :deep(.v-list-item) {
    background: rgba(205, 127, 50, 0.1) !important;
}

.light-theme .nav-text {
    color: #6b4423;
}

.light-theme .sidebar-item.v-list-item--active .nav-text {
    color: #cd7f32;
}

.light-theme .nav-indicator {
    background: #daa520;
    box-shadow: 0 0 8px rgba(218, 165, 32, 0.4);
}

.light-theme .sidebar-footer {
    border-top-color: rgba(184, 134, 11, 0.3);
    background: linear-gradient(0deg, #f5e6d3 0%, transparent 100%);
}

.light-theme .status-label {
    color: #8b7355;
}

.light-theme .status-value.online {
    color: #daa520;
}

.light-theme .footer-time {
    color: #cd7f32;
}

.light-theme .resize-indicator {
    background-color: rgba(184, 134, 11, 0.3);
}

.light-theme .resize-handle:hover .resize-indicator,
.light-theme .resize-handle.is-resizing .resize-indicator {
    background-color: #cd7f32;
    box-shadow: 0 0 8px rgba(205, 127, 50, 0.4);
}
</style>
