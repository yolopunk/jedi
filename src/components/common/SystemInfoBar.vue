<template>
  <v-footer app class="system-info-bar px-3" color="grey-lighten-5">
    <div class="d-flex flex-wrap align-center w-100">
      <!-- 左侧：版本号 -->
      <div class="d-flex align-center mb-1 mb-sm-0">
        <span class="version-tag mr-1">
          <v-icon :icon="mdiInformation" size="x-small" class="mr-1" />
          v{{ appVersion }}
        </span>
      </div>

      <!-- 中间：系统信息 -->
      <div class="d-flex align-center ml-0 ml-sm-3 mb-1 mb-sm-0 flex-grow-1 flex-wrap">
        <div class="info-item mb-1 mb-sm-0">
          <v-icon :icon="mdiDesktopTower" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1">{{ osInfo?.name }} {{ osInfo?.os_version }}</span>
        </div>
        <v-divider vertical class="mx-2 d-none d-sm-flex" style="height: 12px"></v-divider>
        <div class="info-item mb-1 mb-sm-0">
          <v-icon :icon="mdiServer" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1">{{ osInfo?.host_name }}</span>
        </div>
      </div>

      <v-spacer class="d-none d-md-block"></v-spacer>

      <!-- 右侧：系统资源信息 -->
      <div class="d-flex align-center flex-wrap justify-end">
        <!-- CPU -->
        <div class="info-item mx-1 mb-1 mb-sm-0">
          <v-icon :icon="mdiCpu64Bit" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1">CPU: {{ formatPercentage(osInfo?.metrics?.cpu_usage) }}</span>
        </div>

        <!-- 内存 -->
        <div class="info-item mx-1 mb-1 mb-sm-0">
          <v-icon :icon="mdiMemory" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1">内存: {{ formatMemory(osInfo?.metrics?.memory_used) }}</span>
        </div>

        <!-- 磁盘 -->
        <div class="info-item mx-1 mb-1 mb-sm-0">
          <v-icon :icon="mdiHarddisk" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1">磁盘: {{ formatPercentage(getDiskUsagePercentage()) }}</span>
        </div>

        <!-- 网络 -->
        <div class="info-item mx-1 mb-1 mb-sm-0 d-none d-md-flex">
          <v-icon :icon="mdiEthernet" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1">网络: ↓{{ formatDataSize(osInfo?.metrics?.network_received) }} ↑{{ formatDataSize(osInfo?.metrics?.network_transmitted) }}</span>
        </div>

        <!-- 时间 -->
        <div class="info-item ml-1 mb-1 mb-sm-0">
          <v-icon :icon="mdiClockOutline" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1">{{ currentTime }}</span>
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
  mdiInformation,
  mdiDesktopTower,
  mdiServer
} from '@mdi/js'
import { getOsInfo } from '@/api/hosts'
import { OsInfo } from '@/types/os'

// 系统信息
const osInfo = ref<OsInfo | null>(null)

// 刷新间隔 (毫秒)
const REFRESH_INTERVAL = 3000
let refreshTimer: number | null = null

// 应用版本号
const appVersion = '0.1.0'

// 当前时间
const currentTime = computed(() => {
  const now = new Date()
  return now.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
})

// 计算CPU使用率
const cpuUsage = computed(() => {
  return osInfo.value?.metrics?.cpu_usage || 0
})

// 计算内存使用率
const memoryUsage = computed(() => {
  if (!osInfo.value?.metrics) return 0
  const { memory_total, memory_used } = osInfo.value.metrics
  if (memory_total === 0) return 0
  return (memory_used / memory_total) * 100
})

// 计算磁盘使用率
const diskUsage = computed(() => {
  return getDiskUsagePercentage()
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
  min-height: 32px !important;
  height: auto !important;
  font-size: 12px;
  border-top: 1px solid rgba(0, 0, 0, 0.08);
  border-left: none !important;
  border-right: none !important;
  background-color: #FAFAFA !important;
  transition: all 0.25s ease;
  padding: 4px 12px !important;
}

.info-item {
  display: flex;
  align-items: center;
  padding: 0 4px;
  border-radius: 4px;
  transition: all 0.15s ease;
  height: 24px;
}

.info-item:hover {
  background-color: rgba(0, 0, 0, 0.03);
}

.version-tag {
  color: var(--jedi-primary);
  font-weight: 500;
  background-color: var(--jedi-primary-light);
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  letter-spacing: 0.3px;
  border: 1px solid rgba(212, 175, 55, 0.2);
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  height: 20px;
}

.version-tag:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}
</style>
