import { invoke } from '@tauri-apps/api/core'
import type { TaskSpec, WorkerResult, WorkerStatus } from './poolTypes'

export type { TaskSpec, WorkerResult, WorkerStatus }

export async function initPool(maxWorkers: number = 4): Promise<void> {
  await invoke('init_pool', { maxWorkers })
}

export async function getPoolStatus(): Promise<WorkerStatus[]> {
  return await invoke<WorkerStatus[]>('get_pool_status')
}

export async function scheduleTask(spec: TaskSpec): Promise<string> {
  return await invoke<string>('schedule_task', { task: spec })
}

export async function stopWorker(workerId: string): Promise<void> {
  await invoke('stop_worker', { workerId })
}

export async function removeWorker(workerId: string): Promise<WorkerResult> {
  return await invoke<WorkerResult>('remove_worker', { workerId })
}
