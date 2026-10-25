// Driver registry: maps an engine descriptor to the driver that can run it.
//
// M1 ships only the builtin driver; pi / Claude Code / Codex drivers arrive
// in M2-M4 and register here. The chat store asks for a driver and falls back
// to builtin when none exists yet (engine stays visible but flagged).

import { BuiltinDriver } from './builtin'
import { AntigravityDriver } from './drivers/antigravity'
import { ClaudeCodeDriver } from './drivers/claudeCode'
import { CodexDriver } from './drivers/codex'
import { GeminiCliDriver } from './drivers/geminiCli'
import { OpencodeDriver } from './drivers/opencode'
import { PiDriver } from './drivers/pi'
import type { AgentDriver, EngineDescriptor } from './types'

const drivers: AgentDriver[] = [
  new BuiltinDriver(),
  new PiDriver(),
  new ClaudeCodeDriver(),
  new CodexDriver(),
  new GeminiCliDriver(),
  new OpencodeDriver(),
  new AntigravityDriver(),
]

export function registerDriver(driver: AgentDriver): void {
  const idx = drivers.findIndex(d => d.id === driver.id)
  if (idx >= 0) drivers[idx] = driver
  else drivers.push(driver)
}

export function driverFor(engine: EngineDescriptor): AgentDriver | null {
  return drivers.find(d => d.supports(engine)) ?? null
}

export function listDrivers(): AgentDriver[] {
  return [...drivers]
}
