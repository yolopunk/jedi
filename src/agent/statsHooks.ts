// src/agent/statsHooks.ts
//
// Composes a caller's RunAgentHooks with execution-stats recording: the
// original onToolEnd runs first, then the skillStats store records the call.
// This keeps runAgent itself free of any stats/store dependency while both
// the foreground chat and the background pool workers get instrumented.

import { useSkillStatsStore } from '@/stores/skillStats'
import type { RunAgentHooks } from './runAgent'

export function withStatsHooks(hooks: RunAgentHooks): RunAgentHooks {
  const baseToolEnd = hooks.onToolEnd
  return {
    ...hooks,
    onToolEnd: event => {
      baseToolEnd?.(event)
      useSkillStatsStore().recordToolEnd(event)
    },
  }
}
