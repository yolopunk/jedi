// src/agent/types.ts

export type AgentStepType = 'think' | 'tool' | 'skill' | 'finish'
export type AgentStepStatus = 'pending' | 'running' | 'done' | 'error'
export type AgentStatus = 'idle' | 'planning' | 'executing' | 'paused' | 'done' | 'error'
export type ConfirmationMode = 'auto' | 'always' | 'dangerous'

export interface AgentStep {
  id: string
  type: AgentStepType
  status: AgentStepStatus
  content: string
  result?: any
  error?: string
  timestamp: number
}

export interface AgentState {
  status: AgentStatus
  currentStep: AgentStep | null
  history: AgentStep[]
  confirmationRequired: boolean
}

export interface AgentConfig {
  model: string
  provider: string
  confirmationMode: ConfirmationMode
  maxIterations: number
  temperature: number
}

export interface AgentEvent {
  type: 'step_start' | 'step_done' | 'step_error' | 'status_change' | 'confirmation_needed'
  step?: AgentStep
  status?: AgentStatus
  timestamp: number
}
