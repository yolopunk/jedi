export type WorkerStatusKind = 'idle' | 'running' | 'completed' | 'failed' | 'stopped'

export interface TaskSpec {
  id: string
  description: string
  prompt: string
  tools: string[]
  model?: string
  abort_on_error: boolean
}

export interface WorkerStatus {
  id: string
  task_id?: string
  description: string
  status: WorkerStatusKind
  progress?: string
  started_at?: number
  result?: string
  error?: string
}

export interface WorkerResult {
  worker_id: string
  task_id: string
  status: WorkerStatusKind
  result?: string
  error?: string
  duration_ms?: number
}
