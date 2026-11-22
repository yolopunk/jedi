export type AgentExecutionMode = import('@/agent/hub/types').GroupChatMode

export type AgentRunStatus =
  | 'queued'
  | 'running'
  | 'awaiting_approval'
  | 'completed'
  | 'failed'
  | 'stopped'
  | 'interrupted'

export interface AgentGroupMember {
  id: string
  engineId: string
  name: string
  role: string
  enabled: boolean
  nativeSessionId?: string
  lastDeliveredSeq: number
}

export interface AgentGroupMessage {
  id: string
  seq: number
  role: 'user' | 'assistant'
  authorId?: string
  authorName: string
  recipients: string[]
  content: string
  createdAt: string
  runId?: string
}

export interface AgentRunTool {
  id: string
  name: string
  status: 'running' | 'completed' | 'failed'
  detail?: string
}

export interface AgentGroupRun {
  id: string
  memberId: string
  triggerMessageId: string
  messageId: string
  status: AgentRunStatus
  contextSeq: number
  model?: string
  error?: string
  startedAt?: string
  finishedAt?: string
  tools: AgentRunTool[]
}

export interface AgentGroup {
  id: string
  name: string
  projectPath: string
  executionMode: AgentExecutionMode
  members: AgentGroupMember[]
  messages: AgentGroupMessage[]
  runs: AgentGroupRun[]
  createdAt: string
  updatedAt: string
  revision: number
}

export interface CreateAgentGroupInput {
  name: string
  projectPath: string
  executionMode: AgentExecutionMode
  members: Array<{ engineId: string; role: string }>
}

export interface PendingAgentApproval {
  request: AgentApproval
  resolve: (allow: boolean) => void
}

export interface AgentApproval {
  id: string
  groupId: string
  runId: string
  memberId: string
  memberName: string
  toolName: string
  args: unknown
}

export interface AgentEngineOption {
  id: string
  name: string
  location: string
  status: string
  modes: readonly AgentExecutionMode[]
  available: boolean
}

export const ACTIVE_AGENT_RUN_STATUSES: ReadonlySet<AgentRunStatus> = new Set([
  'queued',
  'running',
  'awaiting_approval',
])
