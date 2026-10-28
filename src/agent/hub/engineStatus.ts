// Presentation helpers for detected engines: readiness badge and the
// configured default model. The *actual* model of a turn is never derived
// from here — it only comes from what the agent reports at runtime.

import type { EngineDescriptor } from './types'

type T = (key: string, values?: Record<string, unknown>) => string

export type EngineBadgeKind = 'ready' | 'warn' | 'danger' | 'muted'

export interface EngineBadge {
  kind: EngineBadgeKind
  label: string
}

/** Readiness badge for an engine; null for the builtin engine. */
export function engineBadge(engine: EngineDescriptor, t: T): EngineBadge | null {
  if (engine.tier === 'builtin') return null
  if (engine.tier === 'B') return { kind: 'muted', label: t('chatWorkspace.engineComingSoon') }
  switch (engine.installation?.status) {
    case 'ready':
      return { kind: 'ready', label: t('chatWorkspace.engineReady') }
    case 'needs_auth':
      return { kind: 'warn', label: t('chatWorkspace.engineNeedsAuth') }
    case 'invalid_path':
      return { kind: 'danger', label: t('chatWorkspace.engineInvalidPath') }
    default:
      return { kind: 'muted', label: t('chatWorkspace.engineUnconfirmed') }
  }
}

/** An engine can be picked unless its adapter is missing or its binary vanished. */
export function engineSelectable(engine: EngineDescriptor): boolean {
  if (engine.tier === 'builtin') return true
  if (engine.tier === 'B') return false
  return engine.installation?.status !== 'invalid_path'
}

/** Configured default model, or a statement that the agent decides it. */
export function defaultModelLabel(engine: EngineDescriptor, t: T): string {
  const inst = engine.installation
  if (inst?.defaultModel) return inst.defaultModel
  if (inst?.modelSource === 'agent-default') return t('chatWorkspace.defaultModelByAgent')
  return t('chatWorkspace.defaultModelUnknown')
}
