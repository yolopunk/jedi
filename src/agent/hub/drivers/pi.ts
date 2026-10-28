// PiDriver: drives pi (`--mode rpc`) as a per-turn child process.
//
// Protocol notes verified against pi 1.4.0 on a real machine (docs §10):
//  - spawn: pi --mode rpc [--session <id>] --model <provider/model>
//    * omitting --session auto-creates one; --session <id> resumes it
//      (verified: same sessionId + messageCount across processes).
//  - stdin commands (JSONL, id-correlated): prompt / get_state / steer /
//    follow_up / abort. Responses: {"type":"response","command","success",…}.
//  - stdout events: message_update (assistantMessageEvent carries deltas),
//    message_start/end, turn_end, agent_settled (the true end signal —
//    retries/steering may follow agent_end), auto_retry_start/end.
//  - auth: provider API keys from env (ANTHROPIC_API_KEY, OPENAI_API_KEY, …);
//    inherited from the installed agent configuration.
//
// M2 scope: prompt streaming + tool events + session resume.
// Approval gating via pi's permission extension is a documented follow-up;
// until then pi runs with its own default permission behavior.

import { Channel, invoke } from '@tauri-apps/api/core'
import type { RunAgentHooks, RunAgentResult } from '@/agent/runAgent'
import { nativeModel } from '../model'
import type { AgentDriver, DriverHandle, DriverStartOpts, EngineDescriptor } from '../types'
import { createAgentDiagnostics } from '../agentError'
import { createPiTranslator, textOfContent } from './piProtocol'

// pi sessionId per Jedi chat session, so consecutive turns resume the same
// pi conversation. In-memory for M2: after an app restart a fresh pi session
// starts (persisting this in message metadata is a documented follow-up).
const piSessionByJediSession = new Map<string, string>()

export class PiDriver implements AgentDriver {
  id = 'pi'

  capabilities = {
    steering: true,
    compaction: true,
    images: false,
    resume: true,
    models: true,
    approval: 'extension',
    usesJediKey: false,
  } as const

  supports(engine: EngineDescriptor): boolean {
    return engine.agentId === 'pi' && !!engine.installation
  }

  async start(opts: DriverStartOpts, hooks: RunAgentHooks): Promise<DriverHandle> {
    const engine = opts.engine
    if (!engine?.installation) {
      throw new Error('PiDriver: engine installation missing')
    }

    // Keep the installed agent's model and authentication configuration.
    const env: Record<string, string> = { PI_OFFLINE: '1', PI_TELEMETRY: '0' }
    const args: string[] = ['--mode', 'rpc']
    const resumedSession = opts.nativeSessionId ?? piSessionByJediSession.get(opts.sessionId)
    if (resumedSession) args.push('--session', resumedSession)

    // ---- Spawn through the Rust process manager (strict \n framing). ----
    const procId = `pi-${opts.sessionId}-${Date.now()}`
    const translator = createPiTranslator(hooks)
    const diag = createAgentDiagnostics(engine.name)
    let settled = false
    let cmdSeq = 0

    let resolveDone: (r: RunAgentResult) => void = () => {}
    let rejectDone: (e: Error) => void = () => {}
    const done = new Promise<RunAgentResult>((resolve, reject) => {
      resolveDone = resolve
      rejectDone = reject
    })

    const send = (payload: Record<string, unknown>): Promise<void> => {
      const id = `c${++cmdSeq}`
      return invoke('agent_proc_stdin', { id: procId, line: JSON.stringify({ ...payload, id }) })
    }

    const finish = (failure?: Error) => {
      if (settled) return
      settled = true
      opts.signal?.removeEventListener('abort', onAbort)
      void invoke('agent_proc_kill', { id: procId }).catch(() => {})
      const snap = translator.snapshot()
      if (failure && !snap.fullText.trim()) {
        rejectDone(failure)
        return
      }
      if (snap.piSessionId) piSessionByJediSession.set(opts.sessionId, snap.piSessionId)
      hooks.onFinish?.({ finishReason: snap.finishReason, usage: snap.usage })
      resolveDone({
        text: snap.fullText,
        reasoning: snap.reasoning,
        finishReason: snap.finishReason,
        usage: snap.usage,
        toolCount: snap.toolCount,
        enabledToolNames: [],
        nativeSessionId: snap.piSessionId,
      })
    }

    function onAbort() {
      void send({ type: 'abort' }).catch(() => {})
      setTimeout(() => finish(), 300)
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
          if (record.type === 'agent_settled') {
            const error = translator.snapshot().lastError
            finish(error ? diag.protocol(error) : undefined)
          } else if (record.type === 'response' && record.success === false) {
            finish(diag.protocol(String(record.error ?? 'pi command failed')))
          }
        } catch {
          const early = diag.line(line)
          if (early) finish(early)
        }
      } else if (msg.type === 'exit') {
        if (!settled) {
          finish(diag.exit(msg.code, translator.snapshot().lastError))
        }
      } else if (msg.type === 'stderr') {
        // pi keeps stdout protocol-only; stderr carries auth / startup errors.
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
        env,
        onEvent: channel,
      })
    } catch (e) {
      throw diag.spawn(e)
    }

    if (opts.signal?.aborted) {
      finish(new Error('aborted'))
      return {
        engineId: `pi@${engine.host}`,
        capabilities: { ...this.capabilities },
        done,
        dispose: () => {},
      }
    }
    opts.signal?.addEventListener('abort', onAbort)

    // The last user message is the turn prompt; earlier history lives in the
    // resumed pi session (or is a fresh conversation on the first turn).
    const lastUser = [...opts.messages].reverse().find(m => m.role === 'user')
    const promptText = lastUser ? textOfContent(lastUser.content) : ''
    if (!promptText.trim()) {
      finish(new Error('PiDriver: no user message to send'))
    } else {
      await send({ type: 'prompt', message: promptText })
      // Learn the session id for later resume (also covers fresh sessions).
      void send({ type: 'get_state' }).catch(() => {})
    }

    return {
      engineId: `pi@${engine.host}`,
      capabilities: { ...this.capabilities },
      done,
      steer: (text: string) => send({ type: 'steer', message: text }),
      followUp: (text: string) => send({ type: 'follow_up', message: text }),
      abort: onAbort,
      dispose: () => {
        opts.signal?.removeEventListener('abort', onAbort)
        void invoke('agent_proc_kill', { id: procId }).catch(() => {})
      },
    }
  }
}
