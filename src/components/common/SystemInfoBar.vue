<template>
    <div class="system-info-bar py-0 px-0">
        <div class="d-flex align-center w-100 h-100">
            <!-- Left Section: App Info -->
            <div class="app-info d-flex align-center px-2 h-100 border-right">
                <span
                    class="text-caption font-weight-black text-disabled mr-1 ls-1"
                    >JEDI</span
                >
                <span class="text-caption text-medium-emphasis"
                    >v{{ appVersion }}</span
                >

                <!-- Update Indicator -->
                <v-btn
                    v-if="hasUpdate"
                    variant="text"
                    size="x-small"
                    density="compact"
                    class="ml-1 px-1 update-btn"
                    @click="showUpdateDialog = true"
                >
                    <v-icon
                        :icon="mdiDownload"
                        size="14"
                        class="text-error"
                    ></v-icon>
                    <v-tooltip activator="parent" location="top">
                        {{ $t("update.title") }}: {{ updateInfo?.version }}
                    </v-tooltip>
                </v-btn>

                <!-- Checking Indicator -->
                <v-btn
                    v-if="isChecking"
                    variant="text"
                    size="x-small"
                    density="compact"
                    class="ml-1 px-1"
                >
                    <v-progress-circular
                        :size="14"
                        :width="2"
                        indeterminate
                        class="text-primary"
                    ></v-progress-circular>
                    <v-tooltip activator="parent" location="top">
                        {{ $t("settings.updateChecking") }}
                    </v-tooltip>
                </v-btn>
            </div>

            <!-- Center Section: Podcast Player -->
            <div
                v-if="currentPlaying"
                class="player-section d-flex align-center px-2 h-100 border-left border-right"
            >
                <PlayerBar />
            </div>

            <v-spacer></v-spacer>

            <!-- Right Section: System Info Modules -->
            <div class="d-flex align-center h-100">
                <!-- Module: Network (Compact) -->
                <div
                    class="info-module d-flex align-center px-2 h-100 border-left"
                >
                    <v-icon
                        :icon="mdiEthernet"
                        size="14"
                        class="mr-2 text-medium-emphasis"
                    ></v-icon>
                    <!-- Download -->
                    <div class="d-flex align-center mr-3">
                        <span class="network-speed mono-font text-caption">{{
                            formatNetworkSpeed(
                                osInfo?.metrics?.network_received,
                            )
                        }}</span>
                        <v-icon
                            :icon="mdiDownload"
                            size="14"
                            class="ml-1 text-success"
                        ></v-icon>
                    </div>
                    <!-- Upload -->
                    <div class="d-flex align-center">
                        <span class="network-speed mono-font text-caption">{{
                            formatNetworkSpeed(
                                osInfo?.metrics?.network_transmitted,
                            )
                        }}</span>
                        <v-icon
                            :icon="mdiUpload"
                            size="14"
                            class="ml-1 text-info"
                        ></v-icon>
                    </div>
                </div>

                <!-- Module: Performance (CPU/MEM) -->
                <div
                    class="info-module d-flex align-center px-2 h-100 border-left"
                >
                    <!-- CPU -->
                    <div
                        class="d-flex align-center mr-2 icon-module"
                        style="width: 65px"
                    >
                        <v-icon
                            :icon="mdiCpu64Bit"
                            size="14"
                            class="mr-1 text-medium-emphasis icon-hover"
                        ></v-icon>
                        <v-progress-linear
                            :model-value="osInfo?.metrics?.cpu_usage || 0"
                            color="primary"
                            height="3"
                            rounded
                            class="flex-grow-1 opacity-80"
                        ></v-progress-linear>
                        <v-tooltip
                            activator="parent"
                            location="top"
                            content-class="jedi-tooltip"
                        >
                            <div class="d-flex flex-column align-center">
                                <span class="font-weight-bold mb-1">{{
                                    $t("system.cpuUsage")
                                }}</span>
                                <span class="mono-font">{{
                                    formatPercentage(osInfo?.metrics?.cpu_usage)
                                }}</span>
                            </div>
                        </v-tooltip>
                    </div>

                    <!-- Memory -->
                    <div
                        class="d-flex align-center icon-module"
                        style="width: 65px"
                    >
                        <v-icon
                            :icon="mdiMemory"
                            size="14"
                            class="mr-1 text-medium-emphasis icon-hover"
                        ></v-icon>
                        <v-progress-linear
                            :model-value="getMemoryUsagePercentage()"
                            color="warning"
                            height="3"
                            rounded
                            class="flex-grow-1 opacity-80"
                        ></v-progress-linear>
                        <v-tooltip
                            activator="parent"
                            location="top"
                            content-class="jedi-tooltip"
                        >
                            <div class="d-flex flex-column align-center">
                                <span class="font-weight-bold mb-1">{{
                                    $t("system.memory")
                                }}</span>
                                <span class="mb-1 mono-font">{{
                                    formatPercentage(getMemoryUsagePercentage())
                                }}</span>
                                <span
                                    class="text-caption text-high-emphasis mono-font"
                                    style="font-size: 10px; opacity: 0.9"
                                >
                                    {{
                                        formatDataSize(
                                            osInfo?.metrics?.memory_used,
                                        )
                                    }}
                                    /
                                    {{
                                        formatDataSize(
                                            osInfo?.metrics?.memory_total,
                                        )
                                    }}
                                </span>
                            </div>
                        </v-tooltip>
                    </div>
                </div>

                <!-- Module: Host Info -->
                <div
                    class="info-module d-flex align-center px-2 h-100 border-left hide-on-mobile"
                >
                    <v-icon
                        :icon="mdiServer"
                        size="14"
                        class="text-medium-emphasis icon-hover"
                    ></v-icon>
                    <v-tooltip activator="parent" location="top">
                        <div class="d-flex flex-column align-center">
                            <span class="font-weight-bold mb-1">{{
                                osInfo?.host_name || "Localhost"
                            }}</span>
                            <span class="text-caption"
                                >{{ osInfo?.name }}
                                {{ osInfo?.os_version }}</span
                            >
                        </div>
                    </v-tooltip>
                </div>
            </div>
        </div>

        <!-- Update Dialog -->
        <UpdateDialog
            v-if="updateInfo"
            v-model="showUpdateDialog"
            :update-info="updateInfo"
            :is-installing="isInstalling"
            @install="_handleInstallUpdate"
        ></UpdateDialog>
    </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import {
  mdiCpu64Bit,
  mdiDownload,
  mdiEthernet,
  mdiMemory,
  mdiPause,
  mdiPlay,
  mdiServer,
  mdiUpload,
} from '@mdi/js'
import { getOsInfo } from '@/api/hosts'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { useUpdate } from '@/composables/useUpdate'
import UpdateDialog from '@/components/dialogs/UpdateDialog.vue'
import PlayerBar from '@/components/podcast/PlayerBar.vue'
import type { OsInfo } from '@/types/os'
import pkg from '../../../package.json'

const appVersion = pkg.version

// Audio player
const { currentPlaying, isPaused, togglePlay } = useAudioPlayer()

// Update composable
const { hasUpdate, updateInfo, isChecking, isInstalling, installUpdate } = useUpdate()

const showUpdateDialog = ref(false)

const _handleInstallUpdate = async () => {
  try {
    await installUpdate()
    showUpdateDialog.value = false
  } catch (error) {
    console.error('Failed to install update:', error)
  }
}

// 系统信息
const osInfo = ref<OsInfo | null>(null)

// 刷新间隔 (毫秒)
const REFRESH_INTERVAL = 3000

let refreshTimer: number | null = null

// 格式化百分比
function _formatPercentage(value?: number): string {
  if (value === undefined) return '0%'
  return `${Math.round(value)}%`
}

// 格式化网络速率
function _formatNetworkSpeed(bytes?: number): string {
  if (bytes === undefined) return '0 B/s'

  if (bytes < 1024) {
    return `${bytes}B/s`
  } else if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(0)}K/s`
  } else {
    return `${(bytes / (1024 * 1024)).toFixed(1)}M/s`
  }
}

// 格式化数据大小
function _formatDataSize(bytes?: number): string {
  if (bytes === undefined) return '0 B'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}

function _getMemoryUsagePercentage(): number {
  if (!osInfo.value?.metrics) return 0
  const { memory_used, memory_total } = osInfo.value.metrics
  if (!memory_total) return 0
  return (memory_used / memory_total) * 100
}

async function refreshOsInfo() {
  try {
    const info = await getOsInfo()
    osInfo.value = info
  } catch (error) {
    console.error('Failed to get OS info:', error)
  }
}

onMounted(async () => {
  await refreshOsInfo()

  refreshTimer = window.setInterval(refreshOsInfo, REFRESH_INTERVAL)
})

onUnmounted(() => {
  if (refreshTimer) clearInterval(refreshTimer)
})

// 暴露给模板使用的函数
const formatPercentage = _formatPercentage
const formatNetworkSpeed = _formatNetworkSpeed
const formatDataSize = _formatDataSize
const getMemoryUsagePercentage = _getMemoryUsagePercentage
const handleInstallUpdate = _handleInstallUpdate
</script>

<style scoped>
.system-info-bar {
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-size: 11px; /* Ensure base font size is small enough */
}

.border-right {
    border-right: 1px solid rgba(var(--v-theme-on-surface), 0.08);
}

.border-left {
    border-left: 1px solid rgba(var(--v-theme-on-surface), 0.08);
}

.ls-1 {
    letter-spacing: 1px;
}

.info-module {
    white-space: nowrap;
    transition: background-color 0.2s;
}

.info-module:hover {
    background-color: rgba(var(--v-theme-on-surface), 0.04);
}

.bg-surface-variant-opt {
    background-color: rgba(var(--v-theme-on-surface), 0.03);
}

.icon-module {
    position: relative;
}

.icon-hover {
    transition: color 0.2s;
}

.icon-module:hover .icon-hover {
    color: rgb(var(--v-theme-primary)) !important;
}

/* Monospace font for numbers to prevent jitter */
.mono-font {
    font-family: "SF Mono", "Roboto Mono", "Menlo", monospace;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.5px;
}

.network-speed {
    display: inline-block;
    min-width: 54px; /* Fixed width for stability */
    text-align: right;
}

/* Podcast Player Section */
.player-section {
    flex: 1;
    min-width: 0;
    overflow: hidden;
}

/* Hide non-critical info on mobile/small screens */
@media (max-width: 600px) {
    .hide-on-mobile {
        display: none !important;
    }
}
</style>
