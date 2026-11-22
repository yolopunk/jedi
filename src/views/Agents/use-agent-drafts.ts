import { reactive } from 'vue'

interface AgentDraft {
  content: string
  recipientId: string
  sending: boolean
}

// Keep unfinished messages when changing groups or visiting the legacy chat.
// Drafts are UI state only; persisted conversation data belongs to the Agents store.
const drafts = reactive<Record<string, AgentDraft>>({})

export function agentDraftFor(groupId: string): AgentDraft {
  if (!drafts[groupId]) {
    drafts[groupId] = { content: '', recipientId: 'all', sending: false }
  }
  return drafts[groupId]
}

export function forgetAgentDraft(groupId: string): void {
  delete drafts[groupId]
}
