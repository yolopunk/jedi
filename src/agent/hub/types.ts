// Agent hub unified types (docs/ai-chat/05-multi-agent-rd-plan.md §4).
//
// One event model for every chat engine (builtin / pi / Claude Code / Codex).
// Deliberately aligned with the existing RunAgentHooks shape (decision D6) so
// the current chat trace UI keeps working unchanged; external drivers
// translate their native protocol events into these hooks.

import type { RunAgentHooks, RunAgentResult, RunAgentSpec } from '@/agent/runAgent'

export interface SpawnSpec {
  kind: 'windows-exe' | 'windows-cmd' | 'windows-app' | 'wsl' | 'unix'
  distro?: string | null
  exe: string
}

export interface AgentInstallation {
  agentId: string
  /** "windows" | "wsl:<distro>" | "macos" | "linux" */
  host: string
  exe: string
  version?: string | null
  spawn: SpawnSpec
}

export interface AgentReport {
  id: string
  name: string
  installations: AgentInstallation[]
}

export interface DetectionResult {
  reports: AgentReport[]
  durationMs: number
}

/** A selectable engine = one agent installation at one location ("agent@host"). */
export interface EngineDescriptor {
  /** Stable engine id: "builtin" | "<agentId>@<host>" */
  id: string
  agentId: string
  name: string
  /** Short location badge for the selector: 本机 / WSL:ArchLinux / ... */
  location: string
  host: string
  installation?: AgentInstallation
  /** Driver tier per docs §3.2: A = protocol verified, B = pending M0. */
  tier: 'builtin' | 'A' | 'B'
}

export interface DriverCapabilities {
  steering: boolean
  compaction: boolean
  images: boolean
  resume: boolean
  models: boolean
  approval: 'native' | 'extension' | 'builtin' | 'none'
}

/** Options for starting a run on an engine. */
export interface DriverStartOpts extends RunAgentSpec {
  /** The engine this run targets; external drivers need its installation. */
  engine?: EngineDescriptor
  /**
   * Last known native session id for this Jedi chat session (recovered from
   * persisted message metadata) so external engines resume across app
   * restarts instead of starting a fresh conversation.
   */
  nativeSessionId?: string
  /**
   * Windows file paths of this turn's image attachments. Engines with
   * capability `images` consume them (WSL engines get translated paths via
   * toWslPath; data-URL parts in `messages` serve the same purpose for
   * engines that accept inline base64).
   */
  attachments?: string[]
}

export interface DriverHandle {
  /** Human-readable engine id, for logs. */
  engineId: string
  capabilities: DriverCapabilities
  /** Resolves when the run settles (stream end / result / agent_settled). */
  done: Promise<RunAgentResult>
  /** Mid-run steering; only present when capabilities.steering. */
  steer?: (text: string) => Promise<void>
  /** Queue work for after the run settles; only for capable engines. */
  followUp?: (text: string) => Promise<void>
  /** Abort the run, keeping partial output. */
  abort?: () => void
  /** Tear down process resources without waiting for settlement. */
  dispose?: () => void
}

export interface AgentDriver {
  id: string
  capabilities: DriverCapabilities
  /** True when this driver can run the given engine descriptor. */
  supports(engine: EngineDescriptor): boolean
  /**
   * Start a run. The unified event stream is the RunAgentHooks shape; builtin
   * forwards them directly, external drivers translate native protocol
   * events into the same callbacks.
   */
  start(opts: DriverStartOpts, hooks: RunAgentHooks): Promise<DriverHandle>
}
