import { ref, onMounted, onUnmounted } from 'vue'
import { getPoolStatus, type WorkerStatus } from './pool'

const POOL_POLL_INTERVAL = 1000

export function useAgentPool() {
  const workers = ref<WorkerStatus[]>([])
  const panelOpen = ref(false)
  let pollTimer: ReturnType<typeof setInterval> | null = null

  async function refresh() {
    try {
      workers.value = await getPoolStatus()
    } catch (e) {
      console.error('Failed to fetch pool status:', e)
    }
  }

  function startPolling() {
    refresh()
    pollTimer = setInterval(refresh, POOL_POLL_INTERVAL)
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  onMounted(() => startPolling())
  onUnmounted(() => stopPolling())

  function togglePanel() {
    panelOpen.value = !panelOpen.value
  }

  return {
    workers,
    panelOpen,
    togglePanel,
    refresh,
  }
}
