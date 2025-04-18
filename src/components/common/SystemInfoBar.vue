<template>
  <v-footer app class="system-info-bar px-2" color="grey-lighten-3" border>
    <div class="d-flex justify-space-between align-center w-100">
      <!-- 操作系统信息 -->
      <div class="d-flex align-center">
        <v-icon :icon="mdiDesktopTower" size="small" class="mr-1" color="grey-darken-1" />
        <span class="text-caption text-grey-darken-2">{{ osInfo?.name }} {{ osInfo?.os_version }}</span>
      </div>

      <!-- 系统资源信息 -->
      <div class="d-flex align-center">
        <!-- CPU 使用率 -->
        <div class="d-flex align-center mx-2">
          <v-icon :icon="mdiCpu64Bit" size="small" class="mr-1" color="blue-darken-1" />
          <span class="text-caption">CPU: {{ formatPercentage(osInfo?.metrics?.cpu_usage) }}</span>
        </div>

        <!-- 内存使用率 -->
        <div class="d-flex align-center mx-2">
          <v-icon :icon="mdiMemory" size="small" class="mr-1" color="green-darken-1" />
          <span class="text-caption">内存: {{ formatMemory(osInfo?.metrics?.memory_used) }}/{{ formatMemory(osInfo?.metrics?.memory_total) }}</span>
        </div>

        <!-- 磁盘使用率 -->
        <div class="d-flex align-center mx-2">
          <v-icon :icon="mdiHarddisk" size="small" class="mr-1" color="amber-darken-1" />
          <span class="text-caption">磁盘: {{ formatPercentage(getDiskUsagePercentage()) }}</span>
        </div>

        <!-- 网络流量 -->
        <div class="d-flex align-center mx-2">
          <v-icon :icon="mdiEthernet" size="small" class="mr-1" color="purple-darken-1" />
          <span class="text-caption">网络: ↓{{ formatDataSize(osInfo?.metrics?.network_received) }} ↑{{ formatDataSize(osInfo?.metrics?.network_transmitted) }}</span>
        </div>

        <!-- 主机名 -->
        <div class="d-flex align-center mx-2">
          <v-icon :icon="mdiServer" size="small" class="mr-1" color="grey-darken-1" />
          <span class="text-caption text-grey-darken-2">{{ osInfo?.host_name }}</span>
        </div>
      </div>
    </div>
  </v-footer>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { mdiDesktopTower, mdiCpu64Bit, mdiMemory, mdiHarddisk, mdiEthernet, mdiServer } from '@mdi/js'
import { getOsInfo } from '@/services/hostsService'
import { OsInfo } from '@/types/os'

// 系统信息
const osInfo = ref<OsInfo | null>(null)

// 刷新间隔 (毫秒)
const REFRESH_INTERVAL = 5000
let refreshTimer: number | null = null

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
  // 立即加载一次
  await loadSystemInfo()
  
  // 设置定时刷新
  refreshTimer = window.setInterval(loadSystemInfo, REFRESH_INTERVAL)
})

// 组件卸载时
onUnmounted(() => {
  // 清除定时器
  if (refreshTimer !== null) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
})
</script>

<style scoped>
.system-info-bar {
  height: 28px !important;
  font-size: 12px;
  border-top: 1px solid rgba(0, 0, 0, 0.12);
}
</style>
