// Frontend-driven agent worker pool.
//
// The agent execution loop (LLM calls + tool execution) lives in the renderer
// via the AI SDK (see `runAgent`), so the worker pool that schedules concurrent
// background agents must live here too — a Rust-side `schedule()` cannot drive a
// JS agent loop. This store runs up to `maxWorkers` `runAgent` invocations in
// parallel, each cancellable via its own AbortController, and exposes a reactive
// `WorkerStatus[]` that the pool panel renders.

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { type ChatTurn, runAgent } from '@/agent/runAgent'
import type { TaskSpec, WorkerStatus } from '@/agent/poolTypes'
import { useModelsDevStore } from './modelsDev'
import { useProviderConfigStore } from './providerConfig'

export const useAgentPoolStore = defineStore('agentPool', () => {
  const maxWorkers = ref(4)
  const workers = ref<WorkerStatus[]>([])

  // Non-reactive runtime state keyed by worker id.
  const specs = new Map<string, TaskSpec>()
  const controllers = new Map<string, AbortController>()

  let onWorkerComplete: ((worker: WorkerStatus) => void) | null = null
  let counter = 0

  const runningCount = computed(() => workers.value.filter(w => w.status === 'running').length)

  function findWorker(id: string): WorkerStatus | undefined {
    return workers.value.find(w => w.id === id)
  }

  function setOnWorkerComplete(cb: (worker: WorkerStatus) => void) {
    onWorkerComplete = cb
  }

  /** Queue a task; it starts immediately if a slot is free, otherwise waits. */
  function schedule(spec: TaskSpec): string {
    const id = spec.id || `worker-${Date.now()}-${++counter}`
    specs.set(id, spec)
    workers.value.push({
      id,
      task_id: spec.id,
      description: spec.description,
      status: 'idle',
      progress: 'Queued',
    })
    pump()
    return id
  }

  /** Start as many idle workers as capacity allows. */
  function pump() {
    while (runningCount.value < maxWorkers.value) {
      const next = workers.value.find(w => w.status === 'idle')
      if (!next) break
      void runWorker(next)
    }
  }

  async function runWorker(worker: WorkerStatus) {
    const spec = specs.get(worker.id)
    if (!spec) {
      worker.status = 'failed'
      worker.error = 'Missing task spec'
      return
    }

    worker.status = 'running'
    worker.started_at = Date.now()
    worker.progress = 'Starting…'

    const controller = new AbortController()
    controllers.set(worker.id, controller)

    try {
      const modelsDevStore = useModelsDevStore()
      const providerConfigStore = useProviderConfigStore()
      const provider = modelsDevStore.selectedProviderId || 'openai'
      const model = spec.model || modelsDevStore.selectedModelId || 'gpt-4o-mini'

      const apiKeyInfo = await providerConfigStore.getApiKey(provider)
      if (!apiKeyInfo) {
        throw new Error(`API key not configured for provider: ${provider}`)
      }

      const messages: ChatTurn[] = [{ role: 'user', content: spec.prompt }]
      const result = await runAgent(
        {
          provider,
          model,
          apiKey: apiKeyInfo.key,
          endpoint: apiKeyInfo.endpoint,
          messages,
          sessionId: worker.id,
          signal: controller.signal,
        },
        {
          onToolStart: ({ skillName }) => {
            worker.progress = `Using ${skillName}…`
          },
          onTextDelta: ({ fullContent }) => {
            worker.progress = fullContent.slice(-160)
          },
        }
      )

      if (controller.signal.aborted) return
      worker.status = 'completed'
      worker.result = result.text
      worker.progress = undefined
      onWorkerComplete?.(worker)
    } catch (e) {
      if (controller.signal.aborted) {
        worker.status = 'stopped'
      } else {
        worker.status = 'failed'
        worker.error = e instanceof Error ? e.message : String(e)
      }
      worker.progress = undefined
      onWorkerComplete?.(worker)
    } finally {
      controllers.delete(worker.id)
      pump()
    }
  }

  /** Abort a running worker. */
  function stop(id: string) {
    const worker = findWorker(id)
    if (!worker) return
    const controller = controllers.get(id)
    if (controller) controller.abort()
    if (worker.status === 'running' || worker.status === 'idle') {
      worker.status = 'stopped'
    }
    pump()
  }

  /** Remove a worker from the list (aborting it first if needed). */
  function remove(id: string) {
    stop(id)
    workers.value = workers.value.filter(w => w.id !== id)
    specs.delete(id)
    controllers.delete(id)
  }

  /** Drop all finished (completed/failed/stopped) workers. */
  function clearFinished() {
    for (const w of workers.value) {
      if (w.status !== 'running' && w.status !== 'idle') {
        specs.delete(w.id)
        controllers.delete(w.id)
      }
    }
    workers.value = workers.value.filter(w => w.status === 'running' || w.status === 'idle')
  }

  return {
    maxWorkers,
    workers,
    runningCount,
    schedule,
    stop,
    remove,
    clearFinished,
    setOnWorkerComplete,
  }
})
