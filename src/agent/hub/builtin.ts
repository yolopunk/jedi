// BuiltinDriver: the always-available "engine #0" (docs §4.2 D4).
//
// Thin adapter over the existing runAgent executor. Keeping the builtin
// engine behind the same AgentDriver interface as the external engines means
// the chat UI only ever talks to the unified event model.

import { type RunAgentHooks, runAgent } from '@/agent/runAgent'
import type { AgentDriver, DriverHandle, DriverStartOpts, EngineDescriptor } from './types'

export class BuiltinDriver implements AgentDriver {
  id = 'builtin'

  capabilities = {
    steering: false,
    compaction: false,
    images: true,
    resume: false,
    models: true,
    approval: 'builtin',
  } as const

  supports(engine: EngineDescriptor): boolean {
    return engine.id === 'builtin'
  }

  async start(opts: DriverStartOpts, hooks: RunAgentHooks): Promise<DriverHandle> {
    const done = runAgent(opts, hooks)
    return {
      engineId: this.id,
      capabilities: { ...this.capabilities },
      done,
    }
  }
}
