<template>
  <v-footer app class="system-info-bar py-0 px-0">
    <div class="d-flex align-center w-100 h-100">
      <!-- Left Section: App Info -->
      <div class="app-info d-flex align-center px-4 h-100 border-right">
        <span class="text-caption font-weight-black text-disabled mr-2 ls-1">JEDI</span>
        <span class="text-caption text-medium-emphasis">v{{ appVersion }}</span>
      </div>

      <v-spacer></v-spacer>

      <!-- Right Section: System Info Modules -->
      <div class="d-flex align-center h-100">
        
        <!-- Module: Network -->
        <div class="info-module d-flex align-center px-3 h-100 border-left">
          <v-icon :icon="mdiEthernet" size="16" class="mr-3 text-medium-emphasis"></v-icon>
          <div class="d-flex flex-column text-caption" style="line-height: 1.1;">
            <div class="d-flex align-center">
              <v-icon :icon="mdiArrowDownThin" size="10" class="mr-1 text-success"></v-icon>
              {{ formatNetworkSpeed(osInfo?.metrics?.network_received) }}
            </div>
            <div class="d-flex align-center">
              <v-icon :icon="mdiArrowUpThin" size="10" class="mr-1 text-info"></v-icon>
              {{ formatNetworkSpeed(osInfo?.metrics?.network_transmitted) }}
            </div>
          </div>
        </div>

        <!-- Module: Performance (CPU/MEM) -->
        <div class="info-module d-flex align-center px-3 h-100 border-left">
          <!-- CPU -->
          <div class="d-flex align-center mr-4 icon-module" style="width: 80px;">
            <v-icon :icon="mdiCpu64Bit" size="18" class="mr-2 text-medium-emphasis icon-hover"></v-icon>
            <v-progress-linear
              :model-value="osInfo?.metrics?.cpu_usage || 0"
              color="primary"
              height="4"
              rounded
              class="flex-grow-1 opacity-80"
            ></v-progress-linear>
            <v-tooltip activator="parent" location="top" content-class="jedi-tooltip">
              <div class="d-flex flex-column align-center">
                <span class="font-weight-bold mb-1">{{ $t('system.cpuUsage') }}</span>
                <span>{{ formatPercentage(osInfo?.metrics?.cpu_usage) }}</span>
              </div>
            </v-tooltip>
          </div>

          <!-- Memory -->
          <div class="d-flex align-center icon-module" style="width: 80px;">
            <v-icon :icon="mdiMemory" size="18" class="mr-2 text-medium-emphasis icon-hover"></v-icon>
            <v-progress-linear
              :model-value="getMemoryUsagePercentage()"
              color="warning"
              height="4"
              rounded
              class="flex-grow-1 opacity-80"
            ></v-progress-linear>
            <v-tooltip activator="parent" location="top" content-class="jedi-tooltip">
              <div class="d-flex flex-column align-center">
                <span class="font-weight-bold mb-1">{{ $t('system.memory') }}</span>
                <span class="mb-1">{{ formatPercentage(getMemoryUsagePercentage()) }}</span>
                <span class="text-caption text-high-emphasis" style="font-size: 10px; opacity: 0.9;">
                  {{ formatDataSize(osInfo?.metrics?.memory_used) }} / {{ formatDataSize(osInfo?.metrics?.memory_total) }}
                </span>
              </div>
            </v-tooltip>
          </div>
        </div>

        <!-- Module: Host Info -->
        <div class="info-module d-flex align-center px-3 h-100 border-left hide-on-mobile">
          <v-icon :icon="mdiServer" size="16" class="text-medium-emphasis icon-hover"></v-icon>
          <v-tooltip activator="parent" location="top">
            <div class="d-flex flex-column align-center">
              <span class="font-weight-bold mb-1">{{ osInfo?.host_name || 'Localhost' }}</span>
              <span class="text-caption">{{ osInfo?.name }} {{ osInfo?.os_version }}</span>
            </div>
          </v-tooltip>
        </div>

        <!-- Module: Time -->
        <div class="info-module d-flex align-center px-4 h-100 border-left bg-surface-variant-opt">
          <span class="text-caption font-weight-bold">{{ currentTime }}</span>
        </div>
      </div>
    </div>
  </v-footer>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  mdiEthernet,
  mdiServer,
  mdiCpu64Bit,
  mdiMemory,
  mdiArrowDownThin,
  mdiArrowUpThin
} from '@mdi/js'
import { getOsInfo } from '@/api/hosts'
import { OsInfo } from '@/types/os'
import pkg from '../../../package.json'

const appVersion = pkg.version

// Connection status simulation
const isConnected = ref(true)

// 系统信息
const osInfo = ref<OsInfo | null>(null)

// 刷新间隔 (毫秒)
const REFRESH_INTERVAL = 3000

let refreshTimer: number | null = null
let timeTimer: number | null = null

// 当前时间
const now = ref(new Date())
const currentTime = computed(() => {
  return now.value.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
})

// 格式化百分比
function formatPercentage(value?: number): string {
  if (value === undefined) return '0%'
  return `${Math.round(value)}%`
}

// 格式化网络速率
function formatNetworkSpeed(bytes?: number): string {
  if (bytes === undefined) return '0 B/s'
  const speed = bytes / (REFRESH_INTERVAL / 1000)
  return formatDataSize(speed) + '/s'
}

// 格式化数据大小
function formatDataSize(bytes?: number): string {
  if (bytes === undefined) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIndex = 0
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }
  return `${size.toFixed(unitIndex > 0 ? 1 : 0)}${units[unitIndex]}`
}

// 计算内存使用百分比
function getMemoryUsagePercentage(): number {
  if (!osInfo.value?.metrics) return 0
  const { memory_total, memory_used } = osInfo.value.metrics
  if (!memory_total) return 0
  return (memory_used / memory_total) * 100
}

// 加载系统信息
async function loadSystemInfo() {
  try {
    osInfo.value = await getOsInfo() as OsInfo
  } catch (error) {
    console.error('加载系统信息失败:', error)
    isConnected.value = false
  }
}

// 组件挂载时
onMounted(async () => {
  await loadSystemInfo()
  refreshTimer = window.setInterval(loadSystemInfo, REFRESH_INTERVAL)
  timeTimer = window.setInterval(() => {
    now.value = new Date()
  }, 1000)
})

// 组件卸载时
onUnmounted(() => {
  if (refreshTimer !== null) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
  if (timeTimer !== null) {
    clearInterval(timeTimer)
    timeTimer = null
  }
})
</script>

<style scoped>
.system-info-bar {
  height: 36px !important;
  border-top: 1px solid var(--jedi-border) !important;
  background-color: rgba(var(--v-theme-surface), 0.95) !important;
  backdrop-filter: blur(10px);
  z-index: 100;
  font-family: var(--jedi-font-ui);
}

.border-right {
  border-right: 1px solid var(--jedi-border);
}

.border-left {
  border-left: 1px solid var(--jedi-border);
}

.info-module {
  transition: background-color 0.2s;
}

.info-module:hover {
  background-color: rgba(var(--v-theme-on-surface), 0.03);
}

.bg-surface-variant-opt {
  background-color: rgba(var(--v-theme-surface-variant), 0.05);
}

.icon-hover {
  transition: all 0.2s;
  cursor: help;
}

.icon-hover:hover {
  color: rgb(var(--v-theme-primary)) !important;
  transform: scale(1.1);
  opacity: 1 !important;
}

.ls-1 {
  letter-spacing: 1px;
}

/* Pulsing Dot Animation */
.pulsing-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: #ff5252;
  box-shadow: 0 0 0 0 rgba(255, 82, 82, 0.7);
  transition: all 0.3s;
}

.pulsing-dot.online {
  background-color: #4caf50;
  box-shadow: 0 0 0 0 rgba(76, 175, 80, 0.7);
  animation: pulse-green 2s infinite;
}

@keyframes pulse-green {
  0% {
    transform: scale(0.95);
    box-shadow: 0 0 0 0 rgba(76, 175, 80, 0.7);
  }
  70% {
    transform: scale(1);
    box-shadow: 0 0 0 4px rgba(76, 175, 80, 0);
  }
  100% {
    transform: scale(0.95);
    box-shadow: 0 0 0 0 rgba(76, 175, 80, 0);
  }
}

@media (max-width: 700px) {
  .hide-on-mobile {
    display: none !important;
  }
}
</style>

<style>
/* Global tooltip style override */
.jedi-tooltip {
  background-color: rgba(15, 23, 42, 0.95) !important;
  color: #ffffff !important;
  border: 1px solid rgba(255, 255, 255, 0.15) !important;
  backdrop-filter: blur(8px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3) !important;
}

.jedi-tooltip .text-high-emphasis {
  color: #ffffff !important;
}
</style>
