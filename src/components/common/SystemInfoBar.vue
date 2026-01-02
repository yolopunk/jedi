<template>
  <v-footer app class="system-info-bar">
    <div class="d-flex flex-wrap align-center w-100 px-2">
      <!-- Version -->
      <div class="d-flex align-center mr-4 info-group">
        <v-icon :icon="mdiPackageVariant" size="x-small" class="system-info-icon mr-1" />
        <span class="system-info-text hide-on-narrow">v{{ appVersion }}</span>
        <v-tooltip activator="parent" location="top">Jedi v{{ appVersion }}</v-tooltip>
      </div>

      <!-- OS Info -->
      <div class="d-flex align-center mr-4 info-group">
        <v-icon :icon="mdiDesktopTower" size="x-small" class="system-info-icon mr-1" />
        <span class="system-info-text hide-on-narrow">{{ osInfo?.name }} {{ osInfo?.os_version }}</span>
        <v-tooltip activator="parent" location="top">{{ osInfo?.name }} {{ osInfo?.os_version }}</v-tooltip>
      </div>

      <div class="d-flex align-center mr-4 info-group">
        <v-icon :icon="mdiServer" size="x-small" class="system-info-icon mr-1" />
        <span class="system-info-text hide-on-narrow">{{ osInfo?.host_name }}</span>
        <v-tooltip activator="parent" location="top">Host: {{ osInfo?.host_name }}</v-tooltip>
      </div>

      <v-spacer></v-spacer>

      <!-- Resources -->
      <div class="d-flex align-center gap-3">
        <!-- CPU -->
        <div class="d-flex align-center mx-2 info-group">
          <v-icon :icon="mdiCpu64Bit" size="x-small" class="system-info-icon mr-1" />
          <span class="system-info-text hide-on-narrow">CPU: {{ formatPercentage(osInfo?.metrics?.cpu_usage) }}</span>
          <v-tooltip activator="parent" location="top">CPU Usage: {{ formatPercentage(osInfo?.metrics?.cpu_usage) }}</v-tooltip>
        </div>

        <!-- Memory -->
        <div class="d-flex align-center mx-2 info-group">
          <v-icon :icon="mdiMemory" size="x-small" class="system-info-icon mr-1" />
          <span class="system-info-text hide-on-narrow">MEM: {{ formatMemory(osInfo?.metrics?.memory_used) }}</span>
          <v-tooltip activator="parent" location="top">Memory Used: {{ formatMemory(osInfo?.metrics?.memory_used) }}</v-tooltip>
        </div>

        <!-- Network -->
        <div class="d-flex align-center mx-2 info-group">
          <v-icon :icon="mdiEthernet" size="x-small" class="system-info-icon mr-1" />
          <span class="system-info-text hide-on-narrow" style="font-feature-settings: 'tnum'">
            ↓ {{ formatNetworkSpeed(osInfo?.metrics?.network_received) }}
            ↑ {{ formatNetworkSpeed(osInfo?.metrics?.network_transmitted) }}
          </span>
          <v-tooltip activator="parent" location="top">
            Download: {{ formatNetworkSpeed(osInfo?.metrics?.network_received) }} <br/>
            Upload: {{ formatNetworkSpeed(osInfo?.metrics?.network_transmitted) }}
          </v-tooltip>
        </div>

        <!-- Time -->
        <div class="d-flex align-center mx-2 info-group">
          <v-icon :icon="mdiClockOutline" size="x-small" class="system-info-icon mr-1" />
          <span class="system-info-text hide-on-narrow">{{ currentTime }}</span>
          <v-tooltip activator="parent" location="top">{{ currentTime }}</v-tooltip>
        </div>
      </div>
    </div>
  </v-footer>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  mdiCpu64Bit,
  mdiMemory,
  mdiHarddisk,
  mdiEthernet,
  mdiClockOutline,
  mdiPackageVariant,
  mdiDesktopTower,
  mdiServer
} from '@mdi/js'
import { getOsInfo } from '@/api/hosts'
import { getAppInfo } from '@/api/app'
import { OsInfo } from '@/types/os'

// 系统信息
const osInfo = ref<OsInfo | null>(null)

// 刷新间隔 (毫秒)
const REFRESH_INTERVAL = 3000

let refreshTimer: number | null = null
let timeTimer: number | null = null

// 应用版本号
const appVersion = ref('') 

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

// 格式化内存大小
function formatMemory(bytes?: number): string {
  if (bytes === undefined) return '0 GB'
  return formatDataSize(bytes)
}

// 格式化网络速率
function formatNetworkSpeed(bytes?: number): string {
  if (bytes === undefined) return '0 B/s'
  // 计算每秒速率 (bytes / 3s)
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

  return `${size.toFixed(unitIndex > 0 ? 1 : 0)} ${units[unitIndex]}`
}

// 计算磁盘使用百分比
function getDiskUsagePercentage(): number {
  if (!osInfo.value?.metrics) return 0
  const { disk_total_space, disk_used_space } = osInfo.value.metrics
  if (disk_total_space === 0) return 0
  return (disk_used_space / disk_total_space) * 100
}

// 加载系统信息
async function loadSystemInfo() {
  try {
    osInfo.value = await getOsInfo() as OsInfo
  } catch (error) {
    console.error('加载系统信息失败:', error)
  }
}

// 组件挂载时
onMounted(async () => {
  appVersion.value = (await getAppInfo()).version

  // 立即加载一次
  await loadSystemInfo()

  // 设置定时刷新
  refreshTimer = window.setInterval(loadSystemInfo, REFRESH_INTERVAL)
  
  // 每秒更新时间
  timeTimer = window.setInterval(() => {
    now.value = new Date()
  }, 1000)
})

// 组件卸载时
onUnmounted(() => {
  // 清除定时器
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
/* Responsive text hiding */
@media (max-width: 900px) {
  .hide-on-narrow {
    display: none;
  }
  
  /* Adjust margins when text is hidden */
  .system-info-icon.mr-1 {
    margin-right: 0 !important;
  }
  
  .info-group {
    margin-right: 8px !important;
    margin-left: 8px !important;
  }
}

.system-info-bar {
  min-height: 32px !important;
  height: auto !important;
  font-size: 12px;
  border-top: 1px solid var(--jedi-border) !important;
  border-left: none !important;
  border-right: none !important;
  background-color: var(--jedi-bg-surface) !important;
  color: var(--jedi-text-secondary);
  transition: all 0.25s ease;
  padding: 4px 12px !important;
  backdrop-filter: blur(10px);
}

.system-info-text {
  color: var(--jedi-text-secondary);
  font-weight: 500;
}

.system-info-icon {
  color: var(--jedi-text-tertiary);
}

.info-item {
  display: flex;
  align-items: center;
  padding: 0 6px;
  border-radius: 6px;
  transition: all 0.15s ease;
  height: 24px;
  margin: 0 2px;
}

.info-item:hover {
  background-color: var(--jedi-bg-surface-hover);
  transform: translateY(-1px);
}

.version-tag {
  color: var(--jedi-primary);
  font-weight: 600;
  background-color: var(--jedi-primary-light);
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  letter-spacing: 0.3px;
  border: 1px solid var(--jedi-border);
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  height: 20px;
}

.version-tag:hover {
  transform: translateY(-1px);
  border-color: var(--jedi-primary);
}
</style>