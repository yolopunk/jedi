<template>
    <div
        class="app-header"
        data-tauri-drag-region
        @dblclick="handleDoubleClick"
        @click="handleClick"
    >
        <!-- Status Lights -->
        <div class="status-indicators" data-tauri-no-drag @click.stop>
            <div
                class="status-light online"
                :title="$t('header.systemOnline')"
            ></div>
            <div
                class="status-light standby"
                :title="$t('header.modulesActive')"
            ></div>
            <div
                class="status-light scanning"
                :title="$t('header.monitoring')"
            ></div>
        </div>

        <!-- macOS Traffic Lights / Window Controls -->
        <template v-if="isMacOS">
            <div class="header-left" data-tauri-no-drag>
                <div class="window-controls macos-controls">
                    <div class="traffic-light close" @click="closeWindow">
                        <svg viewBox="0 0 10 10" class="traffic-icon">
                            <path
                                d="M1.5 1.5 L8.5 8.5 M8.5 1.5 L1.5 8.5"
                                stroke="currentColor"
                                stroke-width="1.2"
                                stroke-linecap="round"
                            />
                        </svg>
                    </div>
                    <div class="traffic-light minimize" @click="minimizeWindow">
                        <svg viewBox="0 0 10 10" class="traffic-icon">
                            <path
                                d="M1.5 5 L8.5 5"
                                stroke="currentColor"
                                stroke-width="1.2"
                                stroke-linecap="round"
                            />
                        </svg>
                    </div>
                    <div class="traffic-light maximize" @click="toggleMaximize">
                        <svg viewBox="0 0 10 10" class="traffic-icon">
                            <path
                                d="M2 2 L2 8 L8 8 L8 2 Z"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="1.2"
                            />
                            <path
                                d="M1.5 8.5 L8.5 1.5"
                                stroke="currentColor"
                                stroke-width="1.2"
                                stroke-linecap="round"
                            />
                        </svg>
                    </div>
                </div>
            </div>
        </template>

        <!-- Right: Controls & Window Buttons -->
        <div class="header-right" data-tauri-no-drag>
            <!-- Podcast Mini Player -->
            <div
                v-if="currentPlaying"
                class="mini-player d-flex align-center mr-3 fade-in"
            >
                <v-btn
                    icon
                    size="x-small"
                    variant="text"
                    color="primary"
                    class="player-btn"
                    @click="seek(-15)"
                >
                    <v-icon :icon="mdiRewind15" size="14" />
                    <v-tooltip activator="parent" location="bottom">{{
                        $t("header.rewind15s")
                    }}</v-tooltip>
                </v-btn>

                <v-btn
                    icon
                    size="x-small"
                    variant="text"
                    :color="isPaused ? 'medium-emphasis' : 'primary'"
                    class="player-btn mx-1"
                    @click="togglePlay"
                >
                    <v-icon :icon="isPaused ? mdiPlay : mdiPause" size="18" />
                    <v-tooltip activator="parent" location="bottom">{{
                        isPaused ? $t("header.play") : $t("header.pause")
                    }}</v-tooltip>
                </v-btn>

                <v-btn
                    icon
                    size="x-small"
                    variant="text"
                    color="primary"
                    class="player-btn"
                    @click="seek(15)"
                >
                    <v-icon :icon="mdiFastForward15" size="14" />
                    <v-tooltip activator="parent" location="bottom">{{
                        $t("header.forward15s")
                    }}</v-tooltip>
                </v-btn>

                <div
                    class="text-caption text-truncate ml-2"
                    style="max-width: 120px; font-size: 11px"
                >
                    {{ currentPlaying.title }}
                </div>
            </div>

            <!-- App Controls -->
            <div class="app-controls d-flex align-center mr-2">
                <v-btn
                    icon
                    size="x-small"
                    variant="text"
                    color="medium-emphasis"
                    class="control-btn"
                    @click="toggleTheme"
                >
                    <v-icon :icon="themeIcon" size="16" />
                    <v-tooltip activator="parent" location="bottom">{{
                        themeTooltip
                    }}</v-tooltip>
                </v-btn>

                <v-btn
                    icon
                    size="x-small"
                    variant="text"
                    color="medium-emphasis"
                    class="control-btn"
                    @click="$emit('open-github')"
                >
                    <v-icon :icon="mdiGithub" size="16" />
                    <v-tooltip activator="parent" location="bottom"
                        >GitHub</v-tooltip
                    >
                </v-btn>

                <v-btn
                    icon
                    size="x-small"
                    variant="text"
                    color="medium-emphasis"
                    class="control-btn"
                    @click="$emit('show-settings')"
                >
                    <v-icon :icon="mdiCog" size="16" />
                    <v-tooltip activator="parent" location="bottom">{{
                        $t("header.settings")
                    }}</v-tooltip>
                </v-btn>
            </div>

            <div v-if="!isMacOS" class="divider-vertical mx-1"></div>

            <template v-if="!isMacOS">
                <div class="window-controls windows-controls">
                    <v-btn
                        icon
                        size="x-small"
                        variant="text"
                        @click="minimizeWindow"
                        class="window-btn minimize-btn"
                    >
                        <v-icon :icon="mdiWindowMinimize" size="14" />
                    </v-btn>
                    <v-btn
                        icon
                        size="x-small"
                        variant="text"
                        @click="toggleMaximize"
                        class="window-btn maximize-btn"
                    >
                        <v-icon
                            :icon="
                                isMaximized
                                    ? mdiWindowRestore
                                    : mdiWindowMaximize
                            "
                            size="14"
                        />
                    </v-btn>
                    <v-btn
                        icon
                        size="x-small"
                        variant="text"
                        @click="closeWindow"
                        class="window-btn close-btn"
                    >
                        <v-icon :icon="mdiClose" size="14" />
                    </v-btn>
                </div>
            </template>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import {
    mdiClose,
    mdiWindowMinimize,
    mdiWindowMaximize,
    mdiWindowRestore,
    mdiCog,
    mdiGithub,
    mdiWeatherNight,
    mdiWeatherSunny,
    mdiThemeLightDark,
    mdiPlay,
    mdiPause,
    mdiRewind15,
    mdiFastForward15,
} from "@mdi/js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTheme } from "@/composables/useTheme";
import { useAudioPlayer } from "@/composables/useAudioPlayer";

defineProps<{
    sidebarCollapsed?: boolean;
}>();

defineEmits<{
    (e: "toggle-sidebar"): void;
    (e: "show-settings"): void;
    (e: "open-github"): void;
}>();

const { t } = useI18n();
const appWindow = getCurrentWindow();
const { themeMode, setTheme } = useTheme();
const { currentPlaying, isPaused, togglePlay, seek } = useAudioPlayer();

const isMaximized = ref(false);
const isMacOS = ref(false);
let unlistenResize: (() => void) | null = null;

const themeIcon = computed(() => {
    if (themeMode.value === "dark") return mdiWeatherNight;
    if (themeMode.value === "light") return mdiWeatherSunny;
    return mdiThemeLightDark;
});

const themeTooltip = computed(() => {
    if (themeMode.value === "dark") return t("theme.dark");
    if (themeMode.value === "light") return t("theme.light");
    return t("theme.system");
});

function toggleTheme() {
    if (themeMode.value === "light") {
        setTheme("dark");
    } else if (themeMode.value === "dark") {
        setTheme("system");
    } else {
        setTheme("light");
    }
}

async function minimizeWindow() {
    await appWindow.minimize();
}

async function toggleMaximize() {
    if (isMaximized.value) {
        await appWindow.unmaximize();
    } else {
        await appWindow.maximize();
    }
}

async function closeWindow() {
    await appWindow.close();
}

function handleDoubleClick() {
    toggleMaximize();
}

function handleClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (target.closest('[data-tauri-no-drag]')) {
        return;
    }
    toggleMaximize();
}

async function checkMaximized() {
    isMaximized.value = await appWindow.isMaximized();
}

function checkPlatform() {
    const ua = navigator.userAgent.toLowerCase();
    isMacOS.value = ua.includes("mac") || ua.includes("darwin");
}

let resizeTimeout: number | null = null;

function debouncedCheckMaximized() {
    if (resizeTimeout) {
        clearTimeout(resizeTimeout);
    }
    resizeTimeout = window.setTimeout(() => {
        checkMaximized();
    }, 100);
}

onMounted(async () => {
    checkPlatform();
    await checkMaximized();
    unlistenResize = await appWindow.onResized(() => {
        debouncedCheckMaximized();
    });
});

onUnmounted(() => {
    if (unlistenResize) {
        unlistenResize();
    }
});
</script>

<style scoped>
.app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    padding: 0 12px;
    background: linear-gradient(180deg, #0f0f1a 0%, #0a0a12 100%);
    border-bottom: 1px solid rgba(0, 255, 255, 0.15);
    position: relative;
    z-index: 100;
    user-select: none;
    font-family: "JetBrains Mono", "Fira Code", "SF Mono", monospace;
}

/* Status Indicators */
.status-indicators {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 60px;
}

.status-light {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    position: relative;
}

.status-light.online {
    background: #00ff88;
    box-shadow:
        0 0 10px #00ff88,
        0 0 20px rgba(0, 255, 136, 0.33);
    animation: pulse-online 2s ease-in-out infinite;
}

.status-light.standby {
    background: #ffaa00;
    box-shadow:
        0 0 10px #ffaa00,
        0 0 20px rgba(255, 170, 0, 0.33);
    animation: pulse-standby 1.5s ease-in-out infinite;
}

.status-light.scanning {
    background: #00aaff;
    box-shadow:
        0 0 10px #00aaff,
        0 0 20px rgba(0, 170, 255, 0.33);
    animation: pulse-scanning 1s ease-in-out infinite;
}

@keyframes pulse-online {
    0%,
    100% {
        opacity: 1;
    }
    50% {
        opacity: 0.5;
    }
}

@keyframes pulse-standby {
    0%,
    100% {
        opacity: 0.8;
        transform: scale(1);
    }
    50% {
        opacity: 1;
        transform: scale(1.2);
    }
}

@keyframes pulse-scanning {
    0%,
    100% {
        box-shadow:
            0 0 10px #00aaff,
            0 0 20px rgba(0, 170, 255, 0.33);
    }
    50% {
        box-shadow:
            0 0 15px #00aaff,
            0 0 30px rgba(0, 170, 255, 0.53);
    }
}

.header-left {
    display: flex;
    align-items: center;
    min-width: 68px;
}

.header-right {
    display: flex;
    align-items: center;
    min-width: 80px;
    justify-content: flex-end;
    gap: 8px;
}

.mini-player {
    background: rgba(0, 255, 255, 0.05);
    border-radius: 4px;
    padding: 2px 8px 2px 2px;
    border: 1px solid rgba(0, 255, 255, 0.1);
    height: 24px;
}

.player-btn {
    width: 20px !important;
    height: 20px !important;
    min-width: 20px !important;
}

.app-controls {
    gap: 2px;
}

.control-btn {
    width: 24px !important;
    height: 24px !important;
    min-width: 24px !important;
    border-radius: 4px;
    opacity: 0.8;
    transition: all 0.15s ease;
}

.control-btn:hover {
    background-color: rgba(0, 255, 255, 0.1);
    opacity: 1;
    color: #00ffff !important;
    box-shadow: 0 0 10px rgba(0, 255, 255, 0.2);
}

.divider-vertical {
    width: 1px;
    height: 16px;
    background-color: rgba(0, 255, 255, 0.2);
}

.window-controls {
    display: flex;
    align-items: center;
    gap: 8px;
}

/* macOS Traffic Lights */
.macos-controls {
    margin-left: 2px;
    position: relative;
    z-index: 1000;
}

.traffic-light {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: default;
    margin-right: 8px;
}

.traffic-light:last-child {
    margin-right: 0;
}

.traffic-light.close {
    background-color: #ff5f57;
    border: 1px solid #e0443e;
}

.traffic-light.minimize {
    background-color: #febc2e;
    border: 1px solid #dba524;
}

.traffic-light.maximize {
    background-color: #28c840;
    border: 1px solid #1aa428;
}

.traffic-icon {
    width: 8px;
    height: 8px;
    opacity: 0;
    transition: opacity 0.1s;
    fill: rgba(0, 0, 0, 0.5);
}

.window-controls:hover .traffic-icon {
    opacity: 1;
}

/* Windows/Linux Controls */
.windows-controls {
    margin-left: 8px;
}

.window-btn {
    height: 38px;
    width: 46px;
    border-radius: 0;
    transition: all 0.1s ease;
}

.window-btn .v-icon {
    opacity: 0.7;
    transition: opacity 0.1s ease;
}

.window-btn:hover {
    background: rgba(0, 255, 255, 0.1);
}

.window-btn:hover .v-icon {
    opacity: 1;
    color: #00ffff;
}

.close-btn:hover {
    background: #ff4444 !important;
}

.close-btn:hover .v-icon {
    color: #0a0a0f !important;
    opacity: 1;
}

/* =========================================
   Light Theme Styles (Tatooine Outpost)
   ========================================= */
.light-theme .app-header {
    background: linear-gradient(180deg, #f5e6d3 0%, #efe0cc 100%);
    border-bottom-color: rgba(184, 134, 11, 0.3);
}

.light-theme .status-light.online {
    background: #daa520;
    box-shadow: 0 0 8px rgba(218, 165, 32, 0.4);
}

.light-theme .status-light.standby {
    background: #cd853f;
    box-shadow: 0 0 8px rgba(205, 133, 63, 0.4);
}

.light-theme .status-light.scanning {
    background: #cd7f32;
    box-shadow: 0 0 8px rgba(205, 127, 50, 0.4);
}

.light-theme .traffic-light {
    color: #6b4423;
}

.light-theme .window-btn .v-icon {
    color: #8b7355;
}

.light-theme .window-btn:hover {
    background: rgba(205, 127, 50, 0.15);
}

.light-theme .window-btn:hover .v-icon {
    color: #cd7f32;
}

.light-theme .mini-player {
    background: rgba(184, 134, 11, 0.15);
    border-color: rgba(184, 134, 11, 0.3);
}

.light-theme .control-btn {
    color: #8b7355;
}

.light-theme .control-btn:hover {
    color: #cd7f32;
}

.light-theme .divider-vertical {
    background: #d4a574;
}
</style>
