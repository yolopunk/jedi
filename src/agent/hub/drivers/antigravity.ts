// AntigravityDriver: drives Google's Antigravity CLI (`agy`) in print mode
// (docs §3.2 Tier B3, verified on agy 1.4.7).
//
// Spawn: agy -p <conversation> --output-format stream-json [--conversation
// <id>]. Stdin stays untouched: pre-auth agy waits on stdin for a pasted
// OAuth code, so writing anything there hangs an unauthenticated run — the
// ERROR result record still arrives via agy's internal 60s auth timeout.
// Auth is Google OAuth (interactive `agy` login by the user); no key
// injection. `-p` runs exactly one turn and the `result` record is terminal.

import { Channel, invoke } from '@tauri-apps/api/core'
import type { RunAgentHooks, RunAgentResult } from '@/agent/runAgent'
import { nativeModel } from '../model'
import type { AgentDriver, DriverHandle, DriverStartOpts, EngineDescriptor } from '../types'
import { createAgentDiagnostics } from '../agentError'
import { createAntigravityTranslator } from './antigravityProtocol'
import { buildConversationPrompt } from './geminiCliProtocol'

// conversation_id per Jedi chat session for --conversation resume.
const agyConversationByJediSession = new Map<string, string>()

export class AntigravityDriver implements AgentDriver {
  id = 'antigravity'

  capabilities = {
    steering: false,
    compaction: false,
    images: false,
    resume: true,
    models: true,
    approval: 'none', // headless default fails/waits on unapproved tools
    usesJediKey: false, // Google OAuth interactive login
  } as const

  supports(engine: EngineDescriptor): boolean {
    return engine.agentId === 'antigravity' && !!engine.installation
  }

  async start(opts: DriverStartOpts, hooks: RunAgentHooks): Promise<DriverHandle> {
    const engine = opts.engine
    if (!engine?.installation) {
      throw new Error('AntigravityDriver: engine installation missing')
    }
    const sessionKey = JSON.stringify([engine.id, opts.sessionId])

    const args: string[] = [
      '-p',
      buildConversationPrompt(opts.messages),
      '--output-format',
      'stream-json',
    ]
    const resumed = opts.nativeSessionId ?? agyConversationByJediSession.get(sessionKey)
    if (resumed) args.push('--conversation', resumed)

    const procId = `agy-${opts.sessionId}-${Date.now()}`
    const translator = createAntigravityTranslator(hooks)
    const diag = createAgentDiagnostics(engine.name)
    let settled = false

    let resolveDone: (r: RunAgentResult) => void = () => {}
    let rejectDone: (e: Error) => void = () => {}
    const done = new Promise<RunAgentResult>((resolve, reject) => {
      resolveDone = resolve
      rejectDone = reject
    })

    const finish = (failure?: Error) => {
      if (settled) return
      settled = true
      opts.signal?.removeEventListener('abort', onAbort)
      void invoke('agent_proc_kill', { id: procId }).catch(() => {})
      const snap = translator.snapshot()
      if (failure || snap.lastError) {
        rejectDone(failure ?? diag.protocol(snap.lastError ?? 'agy run failed'))
        return
      }
      if (snap.nativeSessionId) {
        agyConversationByJediSession.set(sessionKey, snap.nativeSessionId)
      }
      hooks.onFinish?.({ finishReason: snap.finishReason, usage: snap.usage })
      resolveDone({
        text: snap.fullText,
        reasoning: '',
        finishReason: snap.finishReason,
        usage: snap.usage,
        toolCount: snap.toolCount,
        enabledToolNames: [],
        nativeSessionId: snap.nativeSessionId,
      })
    }

    function onAbort() {
      void invoke('agent_proc_kill', { id: procId }).catch(() => {})
      setTimeout(() => finish(), 200)
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
          translator.handleRecord(record)
        } catch {
          const early = diag.line(line)
          if (early) finish(early)
        }
        if (translator.snapshot().turnComplete) finish()
      } else if (msg.type === 'exit') {
        if (!settled) finish(diag.exit(msg.code, translator.snapshot().lastError))
      } else if (msg.type === 'stderr') {
        // Pre-auth stderr carries the OAuth URL: agy would block on it forever.
        const early = diag.line(String(msg.line ?? ''))
        if (early) finish(early)
      }
    }

    try {
      await invoke('agent_proc_spawn', {
        id: procId,
        spec: engine.installation.spawn,
        args,
        cwd: null,
        env: null,
        onEvent: channel,
      })
    } catch (e) {
      throw diag.spawn(e)
    }

    if (opts.signal?.aborted) {
      finish(new Error('aborted'))
      return {
        engineId: `antigravity@${engine.host}`,
        capabilities: { ...this.capabilities },
        done,
      }
    }
    opts.signal?.addEventListener('abort', onAbort)

    return {
      engineId: `antigravity@${engine.host}`,
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
