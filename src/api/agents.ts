import { invoke } from '@tauri-apps/api/core'
import type { AgentGroup } from '@/types/agents'

export async function listAgentGroups(): Promise<AgentGroup[]> {
  return invoke<AgentGroup[]>('list_agent_groups')
}

export async function saveAgentGroup(group: AgentGroup): Promise<AgentGroup> {
  return invoke<AgentGroup>('save_agent_group', { group })
}

export async function deleteAgentGroup(groupId: string): Promise<void> {
  return invoke<void>('delete_agent_group', { groupId })
}

/** Reclaim only the deterministic process owned by a saved group run. */
export async function stopAgentGroupProcess(runId: string): Promise<void> {
  return invoke<void>('agent_proc_kill', { id: `jedi-agents-${runId}` })
}
