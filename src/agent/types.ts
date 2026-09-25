// src/agent/types.ts

export type AgentStepType = 'think' | 'tool' | 'skill' | 'finish'
export type AgentStepStatus = 'pending' | 'running' | 'done' | 'error'
export type AgentStatus = 'idle' | 'planning' | 'executing' | 'paused' | 'done' | 'error'

export interface AgentStep {
  id: string
  type: AgentStepType
  status: AgentStepStatus
  content: string
  detail?: string
  // biome-ignore lint/suspicious/noExplicitAny: result type depends on step execution
  result?: any
  error?: string
  timestamp: number
}

export interface AgentState {
  status: AgentStatus
  currentStep: AgentStep | null
  history: AgentStep[]
}

export interface AgentEvent {
  type: 'step_start' | 'step_done' | 'step_error' | 'status_change'
  step?: AgentStep
  status?: AgentStatus
  timestamp: number
}
