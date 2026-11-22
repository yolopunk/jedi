// CodexDriver: per-turn app-server JSON-RPC over stdio, using Codex's own
// login. Explicit group discussion uses native read-only/never policies on
// process, thread (including resume), and turn. The installed config loaders
// reject untrusted, so group approval is intentionally unavailable (see
// groupPolicy.ts); legacy approvals are still individually fail-closed.

import { Channel, invoke } from '@tauri-apps/api/core'
import type { RunAgentHooks, RunAgentResult } from '@/agent/runAgent'
import { createAgentDiagnostics } from '../agentError'
import { nativeModel } from '../model'
import type { AgentDriver, DriverHandle, DriverStartOpts, EngineDescriptor } from '../types'
import { createCodexTranslator } from './codexProtocol'
import {
  activeRunHooks,
  assertCodexThreadPolicy,
  codexApprovalDecision,
  codexGroupChatModes,
  codexMcpConfig,
  codexMcpProbeArgs,
  codexStartArgs,
  codexThreadPolicy,
  codexTurnPolicy,
  driverResumeId,
  driverSessionKey,
  isCodexApprovalMethod,
  isCodexServerRequest,
  validateDriverStart,
} from './groupPolicy'

const codexThreadByJediSession = new Map<string, string>()

function lastUserText(messages: DriverStartOpts['messages']): string {
  const lastUser = [...messages].reverse().find(m => m.role === 'user')
  if (!lastUser) return ''
  if (typeof lastUser.content === 'string') return lastUser.content
  return (lastUser.content as Array<{ type: string; text?: string }>)
    .map(part => (part.type === 'text' ? (part.text ?? '') : ''))
    .join('')
}

/**
 * List native config without connecting to MCP or submitting a model prompt.
 * Codex merges mcp_servers tables, so an empty table override cannot disable
 * preconfigured servers; names must each receive enabled=false at launch.
 */
async function disabledMcpServers(
  opts: DriverStartOpts,
  installation: NonNullable<EngineDescriptor['installation']>,
  cwd: string
): Promise<Record<string, false>> {
  if (opts.signal?.aborted) throw new Error('aborted')
  const id = `codex-config-${crypto.randomUUID()}`
  const lines: string[] = []
  const errors: string[] = []
  let bytes = 0
  let spawned = false
  let settled = false
  let stopped = false
  let killStarted = false
  let resolveResult: (value: Record<string, false>) => void = () => {}
  let rejectResult: (error: Error) => void = () => {}
  const result = new Promise<Record<string, false>>((resolve, reject) => {
    resolveResult = resolve
    rejectResult = reject
  })
  void result.catch(() => {})
  const stop = () => {
    stopped = true
    if (!spawned || killStarted) return
    killStarted = true
    void invoke('agent_proc_kill', { id }).catch(() => {})
  }
  const finish = (failure?: Error) => {
    if (settled) return
    settled = true
    clearTimeout(timer)
    opts.signal?.removeEventListener('abort', onAbort)
    stop()
    if (failure) {
      rejectResult(failure)
      return
    }
    try {
      resolveResult(codexMcpConfig(JSON.parse(lines.join('\n'))))
    } catch (error) {
      rejectResult(error instanceof Error ? error : new Error(String(error)))
    }
  }
  function onAbort() {
    finish(new Error('aborted'))
  }
  const timer = setTimeout(
    () => finish(new Error('Codex MCP configuration probe timed out')),
    10_000
  )
  const channel = new Channel<any>()
  channel.onmessage = (msg: any) => {
    if (settled || !msg || typeof msg !== 'object') return
    if (msg.type === 'line') {
      const line = String(msg.line ?? '')
      bytes += line.length
      if (bytes > 1_000_000) {
        finish(new Error('Codex MCP configuration response exceeded the limit'))
      } else {
        lines.push(line)
      }
    } else if (msg.type === 'stderr') {
      if (errors.length < 10) errors.push(String(msg.line ?? ''))
    } else if (msg.type === 'exit') {
      finish(
        msg.code === 0
          ? undefined
          : new Error(`Codex MCP configuration probe failed: ${errors.join('\n') || msg.code}`)
      )
    }
  }
  opts.signal?.addEventListener('abort', onAbort, { once: true })
  if (opts.signal?.aborted) onAbort()
  if (!settled) {
    try {
      await invoke<string | null | void>('agent_proc_spawn', {
        id,
        spec: installation.spawn,
        args: codexMcpProbeArgs(),
        cwd,
        env: null,
        onEvent: channel,
      })
      spawned = true
      if (stopped || opts.signal?.aborted) stop()
    } catch (error) {
      finish(error instanceof Error ? error : new Error(String(error)))
    }
  }
  return result
}

export class CodexDriver implements AgentDriver {
  id = 'codex'

  capabilities = {
    steering: false,
    compaction: true,
    images: false,
    resume: true,
    models: true,
    approval: 'native',
    usesJediKey: false,
    groupChatModes: codexGroupChatModes,
  } as const

  supports(engine: EngineDescriptor): boolean {
    return engine.agentId === 'codex' && !!engine.installation
  }

  async start(opts: DriverStartOpts, hooks: RunAgentHooks): Promise<DriverHandle> {
    if (opts.signal?.aborted) throw new Error('aborted')
    const engine = opts.engine
    if (!engine?.installation) {
      throw new Error('CodexDriver: engine installation missing')
    }
    const cwd = validateDriverStart(opts, this.capabilities.groupChatModes)
    const promptText = lastUserText(opts.messages)
    if (!promptText.trim()) throw new Error('CodexDriver: no user message to send')
    const mode = opts.executionMode
    const mcpConfig =
      mode === undefined ? {} : await disabledMcpServers(opts, engine.installation, cwd!)
    if (opts.signal?.aborted) throw new Error('aborted')
    const sessionKey = driverSessionKey(engine, opts)
    const resumed = driverResumeId(opts, codexThreadByJediSession.get(sessionKey))
    const procId = opts.processId ?? `codex-${crypto.randomUUID()}`
    const diag = createAgentDiagnostics(engine.name)
    let settled = false
    let spawned = false
    let stopRequested = false
    let killStarted = false
    let nativeSessionId: string | undefined
    let rpcSeq = 0
    const liveHooks = activeRunHooks(hooks, () => !settled)
    const translator = createCodexTranslator(liveHooks)
    // Retain the exact pending change/command for the human confirmation.
    const items = new Map<string, unknown>()
    const serverRequests = new Set<string>()
    const pendingRpc = new Map<
      number,
      { resolve: (value: any) => void; reject: (error: Error) => void }
    >()

    let resolveDone: (r: RunAgentResult) => void = () => {}
    let rejectDone: (e: Error) => void = () => {}
    const done = new Promise<RunAgentResult>((resolve, reject) => {
      resolveDone = resolve
      rejectDone = reject
    })
    void done.catch(() => {})

    const stopProcess = () => {
      stopRequested = true
      if (!spawned || killStarted) return
      killStarted = true
      void invoke('agent_proc_kill', { id: procId }).catch(() => {})
    }
    const send = (payload: Record<string, unknown>): Promise<void> => {
      if (settled || opts.signal?.aborted) return Promise.reject(new Error('codex run ended'))
      return invoke('agent_proc_stdin', { id: procId, line: JSON.stringify(payload) })
    }
    const saveNativeSessionId = (id: unknown) => {
      if (settled || typeof id !== 'string' || !id || id === nativeSessionId) return
      nativeSessionId = id
      codexThreadByJediSession.set(sessionKey, id)
      liveHooks.onNativeSessionId?.(id)
    }
    const rpc = (method: string, params: Record<string, unknown>): Promise<any> => {
      if (settled || opts.signal?.aborted) return Promise.reject(new Error('codex run ended'))
      const requestId = ++rpcSeq
      return new Promise<any>((resolve, reject) => {
        const timer = setTimeout(() => {
          pendingRpc.delete(requestId)
          reject(new Error(`codex ${method} timed out`))
        }, 30_000)
        pendingRpc.set(requestId, {
          resolve: value => {
            clearTimeout(timer)
            resolve(value)
          },
          reject: error => {
            clearTimeout(timer)
            reject(error)
          },
        })
        void send({ id: requestId, method, params }).catch(error => {
          pendingRpc.get(requestId)?.reject(new Error(`codex stdin write failed: ${error}`))
          pendingRpc.delete(requestId)
        })
      })
    }

    const finish = (failure?: Error, aborted = false) => {
      if (settled) return
      const snap = translator.snapshot()
      settled = true
      opts.signal?.removeEventListener('abort', onAbort)
      for (const pending of pendingRpc.values()) {
        pending.reject(failure ?? new Error('codex run ended'))
      }
      pendingRpc.clear()
      stopProcess()
      if (failure || snap.lastError) {
        rejectDone(failure ?? diag.protocol(snap.lastError!))
        return
      }
      const finishReason = aborted ? 'aborted' : snap.finishReason
      try {
        hooks.onFinish?.({ finishReason, usage: snap.usage })
        resolveDone({
          text: snap.fullText,
          reasoning: snap.reasoning,
          finishReason,
          usage: snap.usage,
          toolCount: snap.toolCount,
          enabledToolNames: [],
          nativeSessionId: nativeSessionId ?? resumed,
        })
      } catch (error) {
        rejectDone(error instanceof Error ? error : new Error(String(error)))
      }
    }
    function onAbort() {
      finish(undefined, true)
    }
    const handle: DriverHandle = {
      engineId: `codex@${engine.host}`,
      capabilities: { ...this.capabilities },
      done,
      abort: onAbort,
      dispose: onAbort,
    }

    const handleServerRequest = (record: unknown) => {
      if (!isCodexServerRequest(record) || settled) return
      const key = `${typeof record.id}:${record.id}`
      // A retransmission never opens a second confirmation or grants twice.
      if (serverRequests.has(key)) return
      serverRequests.add(key)
      const decide = async () => {
        if (!isCodexApprovalMethod(record.method)) {
          await send({
            id: record.id,
            error: { code: -32601, message: 'Unsupported server request; denied by Jedi' },
          })
          return
        }
        const snap = translator.snapshot()
        const params = record.params as { itemId?: unknown } | null
        const decision = await codexApprovalDecision(record, opts, {
          threadId: snap.threadId,
          turnId: snap.turnId,
          item: typeof params?.itemId === 'string' ? items.get(params.itemId) : undefined,
          active: () => !settled,
        })
        if (!settled && !opts.signal?.aborted) {
          await send({ id: record.id, result: { decision } })
        }
      }
      void decide().catch(error => {
        if (!settled) finish(error instanceof Error ? error : new Error(String(error)))
      })
    }

    const channel = new Channel<any>()
    channel.onmessage = (msg: any) => {
      if (settled || !msg || typeof msg !== 'object') return
      if (msg.type === 'line') {
        const line = String(msg.line ?? '').trim()
        if (!line) return
        let record: any
        try {
          record = JSON.parse(line)
        } catch {
          const early = diag.line(line)
          if (early) finish(early)
          return
        }
        try {
          // Server requests share IDs with client RPCs. Route by method
          // before looking up pendingRpc; never mistake a tool for a reply.
          if (isCodexServerRequest(record)) {
            handleServerRequest(record)
            return
          }
          if (record?.method && record.id !== undefined && record.id !== null) {
            finish(new Error('Codex sent an invalid server request id'))
            return
          }
          if (record?.method === 'item/started' || record?.method === 'item/updated') {
            const item = record.params?.item
            if (typeof item?.id === 'string') items.set(item.id, item)
          }
          if (record && record.id !== undefined && record.id !== null) {
            const pending = pendingRpc.get(record.id)
            if (pending) {
              pendingRpc.delete(record.id)
              if (record.error) {
                pending.reject(new Error(record.error.message ?? JSON.stringify(record.error)))
              } else {
                pending.resolve(record)
              }
            }
          }
          translator.handleRecord(record)
          if (settled) return
          // Save IDs as soon as the reply arrives, including on runs that
          // later fail/abort, before a start handle has reached the caller.
          saveNativeSessionId(translator.snapshot().threadId)
          if (settled) return
          const model = nativeModel(record)
          if (model) liveHooks.onModel?.(model)
          if (!settled && translator.snapshot().turnComplete) finish()
        } catch (error) {
          finish(error instanceof Error ? error : new Error(String(error)))
        }
      } else if (msg.type === 'exit') {
        const snap = translator.snapshot()
        finish(snap.turnComplete ? undefined : diag.exit(msg.code, snap.lastError))
      } else if (msg.type === 'stderr') {
        const early = diag.line(String(msg.line ?? ''))
        if (early) finish(early)
      }
    }

    opts.signal?.addEventListener('abort', onAbort, { once: true })
    if (opts.signal?.aborted) onAbort()
    if (settled) return handle
    try {
      const resolvedCwd = await invoke<string | null | void>('agent_proc_spawn', {
        id: procId,
        spec: engine.installation.spawn,
        args: codexStartArgs(mode, mcpConfig),
        cwd: cwd ?? null,
        env: null,
        onEvent: channel,
      })
      spawned = true
      if (stopRequested || settled || opts.signal?.aborted) {
        if (!settled) onAbort()
        stopProcess()
        return handle
      }
      // Updated procman returns physical cwd after WSL conversion. Keep the
      // response backward-compatible for legacy callers, but fail a group
      // launch rather than guessing a custom WSL mount on an older runtime.
      if (mode !== undefined && (typeof resolvedCwd !== 'string' || !resolvedCwd)) {
        throw new Error('Agent runtime did not return the canonical group working directory')
      }
      const rpcCwd = typeof resolvedCwd === 'string' ? resolvedCwd : cwd
      await rpc('initialize', {
        clientInfo: { name: 'jedi', title: 'Jedi', version: '0.3.0' },
      })
      if (settled) return handle
      await send({ method: 'initialized' })
      const threadPolicy =
        mode !== undefined
          ? codexThreadPolicy(mode, rpcCwd!, mcpConfig)
          : rpcCwd
            ? { cwd: rpcCwd }
            : {}
      const threadResp = await rpc(
        resumed ? 'thread/resume' : 'thread/start',
        resumed ? { threadId: resumed, ...threadPolicy } : threadPolicy
      )
      if (settled) return handle
      if (mode !== undefined) assertCodexThreadPolicy(threadResp?.result, mode, rpcCwd!)
      const threadId = threadResp?.result?.thread?.id ?? resumed
      if (typeof threadId !== 'string' || !threadId) {
        throw new Error('codex did not return a thread id')
      }
      saveNativeSessionId(threadId)
      if (settled) return handle
      if (threadResp?.result?.model) liveHooks.onModel?.(threadResp.result.model)
      if (settled) return handle
      await rpc('turn/start', {
        threadId,
        input: [{ type: 'text', text: promptText }],
        ...(mode !== undefined ? codexTurnPolicy(mode, rpcCwd!) : rpcCwd ? { cwd: rpcCwd } : {}),
      })
    } catch (error) {
      if (!settled) {
        const failure = spawned
          ? error instanceof Error
            ? error
            : new Error(String(error))
          : diag.spawn(error)
        finish(failure)
        throw failure
      }
    }
    return handle
  }
}
