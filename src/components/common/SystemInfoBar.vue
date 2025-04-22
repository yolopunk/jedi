<template>
  <v-footer app class="system-info-bar px-3 lightsaber-border-container" color="grey-lighten-5">

    <div class="d-flex flex-wrap align-center w-100" style="position: relative; z-index: 2;">
      <!-- 左侧：版本号 -->
      <div class="d-flex align-center mb-1 mb-sm-0">
        <span class="version-tag mr-1 responsive-version-tag">
          <v-tooltip activator="parent" location="top" :text="`版本: v${appVersion}`">
          </v-tooltip>
          <v-icon :icon="mdiPackageVariant" size="x-small" class="mr-1" />
          <span class="version-text">v{{ appVersion }}</span>
        </span>
      </div>

      <!-- 中间：系统信息 -->
      <div class="d-flex align-center ml-0 ml-sm-3 mb-1 mb-sm-0 flex-grow-1 flex-wrap">
        <div class="info-item mb-1 mb-sm-0 responsive-info-item">
          <v-tooltip activator="parent" location="top" :text="`${osInfo?.name} ${osInfo?.os_version}`">
          </v-tooltip>
          <v-icon :icon="mdiDesktopTower" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1 info-text">{{ osInfo?.name }} {{ osInfo?.os_version }}</span>
        </div>
        <v-divider vertical class="mx-2 d-none d-sm-flex" style="height: 12px"></v-divider>
        <div class="info-item mb-1 mb-sm-0 responsive-info-item">
          <v-tooltip activator="parent" location="top" :text="osInfo?.host_name">
          </v-tooltip>
          <v-icon :icon="mdiServer" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1 info-text">{{ osInfo?.host_name }}</span>
        </div>
      </div>

      <v-spacer class="d-none d-md-block"></v-spacer>

      <!-- 右侧：系统资源信息 -->
      <div class="d-flex align-center flex-wrap justify-end">
        <!-- CPU -->
        <div class="info-item mx-1 mb-1 mb-sm-0 responsive-info-item">
          <v-tooltip activator="parent" location="top" :text="`CPU: ${formatPercentage(osInfo?.metrics?.cpu_usage)}`">
          </v-tooltip>
          <v-icon :icon="mdiCpu64Bit" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1 info-text">CPU: {{ formatPercentage(osInfo?.metrics?.cpu_usage) }}</span>
        </div>

        <!-- 内存 -->
        <div class="info-item mx-1 mb-1 mb-sm-0 responsive-info-item">
          <v-tooltip activator="parent" location="top" :text="`内存: ${formatMemory(osInfo?.metrics?.memory_used)}`">
          </v-tooltip>
          <v-icon :icon="mdiMemory" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1 info-text">内存: {{ formatMemory(osInfo?.metrics?.memory_used) }}</span>
        </div>

        <!-- 磁盘 -->
        <div class="info-item mx-1 mb-1 mb-sm-0 responsive-info-item">
          <v-tooltip activator="parent" location="top" :text="`磁盘: ${formatPercentage(getDiskUsagePercentage())}`">
          </v-tooltip>
          <v-icon :icon="mdiHarddisk" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1 info-text">磁盘: {{ formatPercentage(getDiskUsagePercentage()) }}</span>
        </div>

        <!-- 网络 -->
        <div class="info-item mx-1 mb-1 mb-sm-0 responsive-info-item">
          <v-tooltip activator="parent" location="top" :text="`网络: ↓${formatDataSize(osInfo?.metrics?.network_received)} ↑${formatDataSize(osInfo?.metrics?.network_transmitted)}`">
          </v-tooltip>
          <v-icon :icon="mdiEthernet" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1 info-text">网络: ↓{{ formatDataSize(osInfo?.metrics?.network_received) }} ↑{{ formatDataSize(osInfo?.metrics?.network_transmitted) }}</span>
        </div>

        <!-- 时间 -->
        <div class="info-item ml-1 mb-1 mb-sm-0 responsive-info-item">
          <v-tooltip activator="parent" location="top" :text="currentTime">
          </v-tooltip>
          <v-icon :icon="mdiClockOutline" size="x-small" class="mr-1" color="var(--jedi-primary)" />
          <span class="text-caption text-grey-darken-1 info-text">{{ currentTime }}</span>
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

// 应用版本号
const appVersion = ref('') 

// 当前时间
const currentTime = computed(() => {
  const now = new Date()
  return now.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
})

// 注意：移除了未使用的计算属性

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
  appVersion.value = (await getAppInfo()).version

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
  backdrop-filter: blur(10px);
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
  background-color: rgba(0, 0, 0, 0.05);
  transform: translateY(-1px);
}

.version-tag {
  color: var(--jedi-primary);
  font-weight: 500;
  background-color: var(--jedi-primary-light);
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  letter-spacing: 0.3px;
  border: 1px solid rgba(44, 62, 80, 0.15);
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  height: 20px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  background: linear-gradient(135deg, var(--jedi-primary-light) 0%, rgba(52, 152, 219, 0.1) 100%);
}

.version-tag:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(44, 62, 80, 0.25);
}

/* 响应式信息项样式 */
.responsive-info-item {
  position: relative;
  transition: all 0.3s ease;
}

.info-text {
  transition: all 0.3s ease;
  white-space: nowrap;
}

/* 在小屏幕上只显示图标 */
@media (max-width: 1200px) {
  .responsive-info-item .info-text {
    display: none;
  }

  .responsive-info-item {
    width: 24px;
    justify-content: center;
  }

  .responsive-info-item .v-icon {
    margin-right: 0 !important;
  }
}

/* 版本标签响应式样式 */
.responsive-version-tag {
  transition: all 0.3s ease;
}

.version-text {
  transition: all 0.3s ease;
  white-space: nowrap;
}

/* 在小屏幕上只显示图标 */
@media (max-width: 1200px) {
  .responsive-version-tag .version-text {
    display: none;
  }

  .responsive-version-tag {
    width: 24px;
    justify-content: center;
    padding: 2px;
  }

  .responsive-version-tag .v-icon {
    margin-right: 0 !important;
  }
}

/* 在更小的屏幕上调整间距 */
@media (max-width: 600px) {
  .responsive-info-item {
    margin: 0 1px;
    padding: 0 3px;
  }

  .responsive-version-tag {
    margin: 0 1px;
    padding: 2px;
  }
}

/* 光剑边框效果 - 只在底部 */
.lightsaber-border-container {
  position: relative;
  overflow: hidden;
}

.lightsaber-border-container::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 2px;
  background: linear-gradient(
    to right,
    transparent,
    transparent,
    rgba(52, 152, 219, 0.1),
    rgba(52, 152, 219, 0.4),
    rgba(52, 152, 219, 0.8),
    rgba(52, 152, 219, 1),
    rgba(52, 152, 219, 0.8),
    rgba(52, 152, 219, 0.4),
    rgba(52, 152, 219, 0.1),
    transparent,
    transparent
  );
  background-size: 200% 100%;
  opacity: 0;
  transition: opacity 0.3s ease;
  pointer-events: none;
  z-index: 1;
  box-shadow: 0 0 12px rgba(52, 152, 219, 0.8), 0 0 5px rgba(255, 255, 255, 0.5);
  animation: lightsaber-flow 3s linear infinite;
  animation-play-state: paused;
}

.lightsaber-border-container:hover::after {
  opacity: 1;
  animation-play-state: running;
}

@keyframes lightsaber-flow {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}
</style>
