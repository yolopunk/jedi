import { ref, onMounted, onUnmounted } from 'vue'
import { getPoolStatus, type WorkerStatus } from './pool'

const POOL_POLL_INTERVAL = 1000

let onWorkerCompleteCallback: ((worker: WorkerStatus) => void) | null = null

export function setOnWorkerCompleteCallback(cb: (worker: WorkerStatus) => void) {
  onWorkerCompleteCallback = cb
}

export function useAgentPool() {
  const workers = ref<WorkerStatus[]>([])
  const panelOpen = ref(false)
  let pollTimer: ReturnType<typeof setInterval> | null = null
  let previousWorkers: WorkerStatus[] = []

  async function refresh() {
    try {
      const newWorkers = await getPoolStatus()

      // Check for newly completed workers
      if (onWorkerCompleteCallback) {
        for (const worker of newWorkers) {
          const prev = previousWorkers.find(p => p.id === worker.id)
          if (prev && prev.status === 'running' &&
              (worker.status === 'completed' || worker.status === 'failed')) {
            onWorkerCompleteCallback(worker)
          }
        }
      }

      previousWorkers = [...newWorkers]
      workers.value = newWorkers
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
