import { storeToRefs } from 'pinia'
import { ref } from 'vue'
import { useAgentPoolStore } from '@/stores/agentPool'
import type { WorkerStatus } from './pool'

// The worker pool is now a reactive Pinia store (see `stores/agentPool.ts`).
// These wrappers preserve the previous composable API so existing components
// keep working without the old 1s Rust polling loop.

export function setOnWorkerCompleteCallback(cb: (worker: WorkerStatus) => void) {
  useAgentPoolStore().setOnWorkerComplete(cb)
}

export function useAgentPool() {
  const store = useAgentPoolStore()
  const { workers } = storeToRefs(store)
  const panelOpen = ref(false)

  function togglePanel() {
    panelOpen.value = !panelOpen.value
  }

  // State is reactive via the store; refresh is kept as a no-op for compatibility.
  async function refresh() {}

  return {
    workers,
    panelOpen,
    togglePanel,
    refresh,
    stop: store.stop,
    remove: store.remove,
  }
}
