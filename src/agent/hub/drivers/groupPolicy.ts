// Opt-in group policy. These restrictions are native CLI controls, not a
// claim that Jedi supplies an OS sandbox for Claude Code.
//
// Verified locally without model requests:
// - Claude Code 2.1.299 --help: --tools, --allowedTools, dontAsk, --safe-mode,
//   --restricted, --permission-prompts none, --strict-mcp-config.
// - Codex 0.166.0 (WSL) / 0.171.0 (Windows) generated app-server schemas:
//   read-only/readOnly, never/untrusted, requestApproval and accept/decline.
//   Both native config loaders REJECT untrusted (--print-startup-config), so
//   approval is deliberately NOT advertised for these installations. Never
//   substitute on-request (model-controlled) or an unrestricted sandbox.

import type { ConfirmToolRequest, RunAgentHooks } from '@/agent/runAgent'
import type { DriverStartOpts, EngineDescriptor, GroupChatMode } from '../types'

export const claudeGroupChatModes = ['discussion'] as const
export const codexGroupChatModes = ['discussion'] as const

export function validateDriverStart(
  opts: DriverStartOpts,
  supported: readonly GroupChatMode[]
): string | undefined {
  if (opts.executionMode !== undefined && !supported.includes(opts.executionMode)) {
    throw new Error(`Unsupported group execution mode: ${opts.executionMode}`)
  }
  const cwd = opts.cwd
  if (cwd !== undefined) {
    if (
      typeof cwd !== 'string' ||
      !cwd.trim() ||
      /[\0\r\n]/.test(cwd) ||
      !(/^[A-Za-z]:[\\/]/.test(cwd) || cwd.startsWith('/') || /^\\\\[^\\]+\\[^\\]+/.test(cwd))
    ) {
      throw new Error('Agent working directory must be a nonempty absolute path')
    }
  }
  if (opts.executionMode !== undefined && cwd === undefined) {
    throw new Error('Group execution requires an explicit working directory')
  }
  return cwd
}

/** Legacy no-cwd chats retain their existing cache key. Group scopes do not. */
export function driverSessionKey(engine: EngineDescriptor, opts: DriverStartOpts): string {
  return JSON.stringify(
    opts.executionMode !== undefined || opts.cwd !== undefined
      ? [engine.id, opts.sessionId, opts.executionMode ?? null, opts.cwd ?? null]
      : [engine.id, opts.sessionId]
  )
}

/** An unproven persisted ID must never import a legacy session into a group. */
export function driverResumeId(opts: DriverStartOpts, scopedId?: string): string | undefined {
  if (opts.executionMode === undefined) return opts.nativeSessionId ?? scopedId
  return opts.nativeSessionId && opts.nativeSessionId === scopedId ? scopedId : undefined
}

/** Suppress reentrant/queued hook calls once the owning run has settled. */
export function activeRunHooks(hooks: RunAgentHooks, active: () => boolean): RunAgentHooks {
  return new Proxy(hooks, {
    get(target, property, receiver) {
      const value = Reflect.get(target, property, receiver)
      if (typeof value !== 'function') return value
      return (...args: unknown[]) => {
        if (active()) return value(...args)
      }
    },
  })
}

export function claudeStartArgs(mode?: GroupChatMode): string[] {
  if (mode !== undefined && mode !== 'discussion') {
    throw new Error('Claude Code group approval mode is unsupported')
  }
  const args = [
    '-p',
    '--input-format',
    'stream-json',
    '--output-format',
    'stream-json',
    '--verbose',
    '--include-partial-messages',
    '--permission-mode',
    mode === 'discussion' ? 'dontAsk' : 'default',
  ]
  if (mode === 'discussion') {
    args.push(
      '--tools',
      'Read,Glob,Grep',
      '--allowedTools',
      'Read,Glob,Grep',
      '--permission-prompts',
      'none',
      '--strict-mcp-config',
      '--mcp-config',
      JSON.stringify({ mcpServers: {} }),
      '--safe-mode',
      '--restricted',
      '--disable-slash-commands',
      '--no-chrome'
    )
  }
  return args
}

export function codexApprovalPolicy(mode: GroupChatMode): 'never' | 'untrusted' {
  if (mode === 'approval') {
    throw new Error('Codex group approval mode is unavailable in the installed CLI')
  }
  return 'never'
}

const codexCustomizationConfig = {
  'features.hooks': false,
  'features.plugins': false,
  'features.apps': false,
  'features.skill_mcp_dependency_install': false,
  'features.mcp_connection_sharing': false,
  web_search: 'disabled',
} as const

/** Codex -c splits dotted keys literally; reject names that cannot round-trip. */
export function codexMcpConfig(value: unknown): Record<string, false> {
  if (!Array.isArray(value)) throw new Error('Codex did not return an MCP configuration list')
  const config: Record<string, false> = {}
  for (const server of value) {
    if (
      !server ||
      typeof server !== 'object' ||
      typeof server.name !== 'string' ||
      !/^[A-Za-z0-9_-]+$/.test(server.name)
    ) {
      throw new Error('Codex returned an invalid MCP server name')
    }
    // mcp_servers={} is NOT sufficient: native config merges tables. Disable
    // every configured server by name before app-server can connect to it.
    config[`mcp_servers.${server.name}.enabled`] = false
  }
  return config
}

export function codexStartArgs(
  mode?: GroupChatMode,
  mcpConfig: Record<string, false> = {}
): string[] {
  if (mode === undefined) return ['app-server']
  const config: Record<string, unknown> = {
    sandbox_mode: 'read-only',
    approval_policy: codexApprovalPolicy(mode),
    approvals_reviewer: 'user',
    ...codexCustomizationConfig,
    ...mcpConfig,
  }
  const args = ['app-server']
  for (const [key, value] of Object.entries(config)) {
    args.push('-c', `${key}=${JSON.stringify(value)}`)
  }
  return args
}

export function codexMcpProbeArgs(): string[] {
  const args: string[] = []
  for (const [key, value] of Object.entries(codexCustomizationConfig)) {
    args.push('-c', `${key}=${JSON.stringify(value)}`)
  }
  return [...args, 'mcp', 'list', '--json']
}

export function codexThreadPolicy(
  mode: GroupChatMode,
  cwd: string,
  mcpConfig: Record<string, false> = {}
): Record<string, unknown> {
  return {
    cwd,
    sandbox: 'read-only',
    approvalPolicy: codexApprovalPolicy(mode),
    approvalsReviewer: 'user',
    config: { ...codexCustomizationConfig, ...mcpConfig },
  }
}

export function codexTurnPolicy(mode: GroupChatMode, cwd: string): Record<string, unknown> {
  return {
    cwd,
    approvalPolicy: codexApprovalPolicy(mode),
    approvalsReviewer: 'user',
    sandboxPolicy: { type: 'readOnly', networkAccess: false },
  }
}

function sameCwd(left: string, right: string): boolean {
  if (/^[A-Za-z]:[\\/]/.test(left) && /^[A-Za-z]:[\\/]/.test(right)) {
    return left.replace(/\\/g, '/').toLowerCase() === right.replace(/\\/g, '/').toLowerCase()
  }
  return left === right
}

/** Fail before turn/start if the server ignored/overrode native restrictions. */
export function assertCodexThreadPolicy(value: unknown, mode: GroupChatMode, cwd: string): void {
  const result = value as {
    approvalPolicy?: unknown
    approvalsReviewer?: unknown
    cwd?: unknown
    sandbox?: { type?: unknown; networkAccess?: unknown; writableRoots?: unknown }
  } | null
  if (
    !result ||
    result.approvalPolicy !== codexApprovalPolicy(mode) ||
    result.approvalsReviewer !== 'user' ||
    result.sandbox?.type !== 'readOnly' ||
    (result.sandbox.networkAccess !== undefined && result.sandbox.networkAccess !== false) ||
    result.sandbox.writableRoots !== undefined ||
    typeof result.cwd !== 'string' ||
    !sameCwd(result.cwd, cwd)
  ) {
    throw new Error(
      'Codex did not confirm the requested group working directory and read-only policy'
    )
  }
}

export interface CodexServerRequest {
  id: string | number
  method: string
  params?: unknown
}

export function isCodexServerRequest(value: unknown): value is CodexServerRequest {
  if (!value || typeof value !== 'object') return false
  const record = value as Record<string, unknown>
  return (
    typeof record.method === 'string' &&
    ((typeof record.id === 'string' && record.id.length > 0) ||
      (typeof record.id === 'number' && Number.isSafeInteger(record.id)))
  )
}

export function isCodexApprovalMethod(method: string): boolean {
  return (
    method === 'item/commandExecution/requestApproval' ||
    method === 'item/fileChange/requestApproval'
  )
}

export interface CodexApprovalContext {
  threadId?: string
  turnId?: string
  /** File-change requests omit the diff; include the preceding item if known. */
  item?: unknown
  active: () => boolean
}

/** Only these two known requests can ever receive a one-shot acceptance. */
export async function codexApprovalDecision(
  request: CodexServerRequest,
  opts: Pick<DriverStartOpts, 'executionMode' | 'confirmTool' | 'signal'>,
  context: CodexApprovalContext
): Promise<'accept' | 'decline'> {
  if (
    !isCodexApprovalMethod(request.method) ||
    opts.executionMode === 'discussion' ||
    !opts.confirmTool ||
    opts.signal?.aborted ||
    !context.active()
  ) {
    return 'decline'
  }
  const params = request.params as Record<string, unknown> | null
  const command = request.method === 'item/commandExecution/requestApproval'
  if (
    !params ||
    typeof params !== 'object' ||
    typeof params.itemId !== 'string' ||
    !params.itemId ||
    !context.threadId ||
    !context.turnId ||
    params.threadId !== context.threadId ||
    params.turnId !== context.turnId ||
    // A root grant is session-scoped. Never convert it into a one-shot yes.
    params.grantRoot != null ||
    (command && (typeof params.command !== 'string' || !params.command.trim())) ||
    (!command && !context.item)
  ) {
    return 'decline'
  }
  const confirmation: ConfirmToolRequest = {
    toolId: `codex:${request.method}:${typeof request.id}:${request.id}`,
    toolName: command ? 'bash' : 'edit',
    risk: command ? 'system' : 'write',
    args: { ...params, ...(context.item ? { item: context.item } : {}) },
  }
  try {
    const approved = await opts.confirmTool(confirmation)
    return approved === true && !opts.signal?.aborted && context.active() ? 'accept' : 'decline'
  } catch {
    return 'decline'
  }
}
