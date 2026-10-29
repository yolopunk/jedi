// CodexDriver: drives Codex CLI's `app-server` mode (JSON-RPC 2.0 over stdio,
// "jsonrpc" header omitted — verified on codex-cli 0.162.1, docs §10).
//
// Per-turn flow: spawn → initialize → thread/resume (or thread/start) →
// turn/start {threadId, input} → stream item/* + turn/* notifications →
// settle at turn/completed. Auth is Codex's own login (~/.codex/auth.json);
// no key injection. Approvals: not surfaced by this version in probes —
// translation is a documented follow-up once shapes are captured.

import { Channel, invoke } from '@tauri-apps/api/core'
import type { RunAgentHooks, RunAgentResult } from '@/agent/runAgent'
import { nativeModel } from '../model'
import type { AgentDriver, DriverHandle, DriverStartOpts, EngineDescriptor } from '../types'
import { createAgentDiagnostics } from '../agentError'
import { createCodexTranslator } from './codexProtocol'

// codex threadId per Jedi chat session for thread/resume continuity.
// In-memory for M4 (persisting in message metadata is a documented follow-up).
const codexThreadByJediSession = new Map<string, string>()

function lastUserText(messages: DriverStartOpts['messages']): string {
  const lastUser = [...messages].reverse().find(m => m.role === 'user')
  if (!lastUser) return ''
  if (typeof lastUser.content === 'string') return lastUser.content
  return (lastUser.content as Array<{ type: string; text?: string }>)
    .map(part => (part.type === 'text' ? (part.text ?? '') : ''))
    .join('')
}

export class CodexDriver implements AgentDriver {
  id = 'codex'

  capabilities = {
    steering: false, // interrupt/steer methods not yet captured on 0.162.1
    compaction: true,
    images: false,
    resume: true,
    models: true,
    approval: 'native',
    usesJediKey: false, // own ChatGPT login (~/.codex)
  } as const

  supports(engine: EngineDescriptor): boolean {
    return engine.agentId === 'codex' && !!engine.installation
  }

  async start(opts: DriverStartOpts, hooks: RunAgentHooks): Promise<DriverHandle> {
    const engine = opts.engine
    if (!engine?.installation) {
      throw new Error('CodexDriver: engine installation missing')
    }
    const sessionKey = JSON.stringify([engine.id, opts.sessionId])

    const procId = `codex-${opts.sessionId}-${Date.now()}`
    const translator = createCodexTranslator(hooks)
    const diag = createAgentDiagnostics(engine.name)
    let settled = false
    let rpcSeq = 0
    // id-correlated pending requests: turn/start needs the threadId from the
    // thread/start response, so rpc() must resolve on the matching reply.
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

    const rpc = (method: string, params: Record<string, unknown>): Promise<any> => {
      rpcSeq += 1
      const requestId = rpcSeq
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
        invoke('agent_proc_stdin', {
          id: procId,
          line: JSON.stringify({ id: requestId, method, params }),
        }).catch(e => {
          pendingRpc.get(requestId)?.reject(new Error(`codex stdin write failed: ${e}`))
          pendingRpc.delete(requestId)
        })
      })
    }

    const finish = (failure?: Error) => {
      if (settled) return
      settled = true
      for (const pending of pendingRpc.values())
        pending.reject(failure ?? new Error('codex run ended'))
      pendingRpc.clear()
      opts.signal?.removeEventListener('abort', onAbort)
      void invoke('agent_proc_kill', { id: procId }).catch(() => {})
      const snap = translator.snapshot()
      if (failure || snap.lastError) {
        rejectDone(failure ?? diag.protocol(snap.lastError!))
        return
      }
      if (snap.threadId) codexThreadByJediSession.set(sessionKey, snap.threadId)
      hooks.onFinish?.({ finishReason: snap.finishReason, usage: snap.usage })
      resolveDone({
        text: snap.fullText,
        reasoning: snap.reasoning,
        finishReason: snap.finishReason,
        usage: snap.usage,
        toolCount: snap.toolCount,
        enabledToolNames: [],
        nativeSessionId: snap.threadId,
      })
    }

    function onAbort() {
      // Interrupt methods not yet captured; terminating keeps partial output.
      finish()
    }

    const channel = new Channel<any>()
    channel.onmessage = (msg: any) => {
      if (!msg || typeof msg !== 'object') return
      if (msg.type === 'line') {
        const line = String(msg.line ?? '').trim()
        if (!line) return
        try {
          const record = JSON.parse(line)
          const model = nativeModel(record)
          if (model) hooks.onModel?.(model)
          if (record && record.id !== undefined && record.id !== null) {
            const pending = pendingRpc.get(record.id)
            if (pending) {
              pendingRpc.delete(record.id)
              if (record.error)
                pending.reject(new Error(record.error.message ?? JSON.stringify(record.error)))
              else pending.resolve(record)
            }
          }
          translator.handleRecord(record)
        } catch {
          const early = diag.line(line)
          if (early) finish(early)
        }
        if (translator.snapshot().turnComplete) finish()
      } else if (msg.type === 'exit') {
        if (!settled && !translator.snapshot().turnComplete) {
          finish(diag.exit(msg.code, translator.snapshot().lastError))
        }
      } else if (msg.type === 'stderr') {
        const early = diag.line(String(msg.line ?? ''))
        if (early) finish(early)
      }
    }

    try {
      await invoke('agent_proc_spawn', {
        id: procId,
        spec: engine.installation.spawn,
        args: ['app-server'],
        cwd: null,
        env: null,
        onEvent: channel,
      })
    } catch (e) {
      throw diag.spawn(e)
    }

    if (opts.signal?.aborted) {
      finish(new Error('aborted'))
      return { engineId: `codex@${engine.host}`, capabilities: { ...this.capabilities }, done }
    }
    opts.signal?.addEventListener('abort', onAbort)

    const promptText = lastUserText(opts.messages)
    if (!promptText.trim()) {
      finish(new Error('CodexDriver: no user message to send'))
      return { engineId: `codex@${engine.host}`, capabilities: { ...this.capabilities }, done }
    }

    // Observe early process failures even while initialization is pending.
    void done.catch(() => {})
    try {
      // Handshake, then (re)open the conversation (awaiting its threadId), then
      // start the turn.
      await rpc('initialize', {
        clientInfo: { name: 'jedi', title: 'Jedi', version: '0.3.0' },
      })
      await invoke('agent_proc_stdin', {
        id: procId,
        line: JSON.stringify({ method: 'initialized' }),
      })
      const resumed = opts.nativeSessionId ?? codexThreadByJediSession.get(sessionKey)
      const threadResp = await rpc(
        resumed ? 'thread/resume' : 'thread/start',
        resumed ? { threadId: resumed } : {}
      )
      if (threadResp?.result?.model) hooks.onModel?.(threadResp.result.model)
      const threadId: string | undefined = threadResp?.result?.thread?.id ?? resumed
      if (!threadId) {
        finish(new Error('codex did not return a thread id'))
        return { engineId: `codex@${engine.host}`, capabilities: { ...this.capabilities }, done }
      }
      await rpc('turn/start', {
        threadId,
        input: [{ type: 'text', text: promptText }],
      })
    } catch (error) {
      finish(error instanceof Error ? error : new Error(String(error)))
      throw error
    }

    return {
      engineId: `codex@${engine.host}`,
      capabilities: { ...this.capabilities },
      done,
      abort: onAbort,
      dispose: () => {
        opts.signal?.removeEventListener('abort', onAbort)
        void invoke('agent_proc_kill', { id: procId }).catch(() => {})
      },
    }
  }
}
