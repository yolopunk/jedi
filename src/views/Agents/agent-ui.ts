import type { AgentEngineOption, AgentExecutionMode, AgentGroupRun } from '@/types/agents'
import { ACTIVE_AGENT_RUN_STATUSES } from '@/types/agents'

export function engineSupportsMode(
  engine: AgentEngineOption | undefined,
  mode: AgentExecutionMode
): boolean {
  return !!engine?.available && engine.modes.includes(mode)
}

export function engineStatusKey(
  engine: AgentEngineOption | undefined,
  mode?: AgentExecutionMode
): string {
  if (!engine) return 'agents.engine.notDetected'
  if (!engine.modes.length) return 'agents.engine.unsupported'
  if (mode && !engine.modes.includes(mode)) return 'agents.engine.unsupportedMode'
  switch (engine.status) {
    case 'needs_auth':
    case 'auth_required':
      return 'agents.engine.needsAuth'
    case 'invalid_path':
      return 'agents.engine.invalidPath'
    case 'unconfirmed':
    case 'unverified':
    case 'unknown':
      return 'agents.engine.unconfirmed'
    default:
      return engine.available ? 'agents.engine.ready' : 'agents.engine.unavailable'
  }
}

export function isActiveRun(run: AgentGroupRun | undefined): boolean {
  return !!run && ACTIVE_AGENT_RUN_STATUSES.has(run.status)
}

export function runStatusClass(run: AgentGroupRun | undefined): string {
  if (!run) return ''
  if (run.status === 'failed' || run.status === 'interrupted') return 'is-danger'
  if (run.status === 'awaiting_approval') return 'is-warning'
  if (run.status === 'completed') return 'is-success'
  if (isActiveRun(run)) return 'is-active'
  return ''
}
