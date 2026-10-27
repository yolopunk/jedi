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
//    we inject whatever the user configured in Jedi's keyring.
//
// M2 scope: prompt streaming + tool events + session resume + key injection.
// Approval gating via pi's permission extension is a documented follow-up;
// until then pi runs with its own default permission behavior.

import { Channel, invoke } from '@tauri-apps/api/core'
import type { RunAgentHooks, RunAgentResult } from '@/agent/runAgent'
import { useProviderConfigStore } from '@/stores/providerConfig'
import type { AgentDriver, DriverHandle, DriverStartOpts, EngineDescriptor } from '../types'
import { createPiTranslator, textOfContent } from './piProtocol'

/** Jedi provider id -> pi env var (pi providers.md "Use an API key from the environment"). */
const PROVIDER_ENV: Record<string, string> = {
  anthropic: 'ANTHROPIC_API_KEY',
  openai: 'OPENAI_API_KEY',
  deepseek: 'DEEPSEEK_API_KEY',
  openrouter: 'OPENROUTER_API_KEY',
  google: 'GEMINI_API_KEY',
  gemini: 'GEMINI_API_KEY',
  xai: 'XAI_API_KEY',
  groq: 'GROQ_API_KEY',
  mistral: 'MISTRAL_API_KEY',
  zai: 'ZAI_API_KEY',
}

/** Jedi provider id -> pi provider prefix for --model. */
const PROVIDER_MODEL_PREFIX: Record<string, string> = {
  anthropic: 'anthropic',
  openai: 'openai',
  deepseek: 'deepseek',
  openrouter: 'openrouter',
  google: 'google',
  xai: 'xai',
  groq: 'groq',
  mistral: 'mistral',
}

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
    usesJediKey: true,
  } as const

  supports(engine: EngineDescriptor): boolean {
    return engine.agentId === 'pi' && !!engine.installation
  }

  async start(opts: DriverStartOpts, hooks: RunAgentHooks): Promise<DriverHandle> {
    const engine = opts.engine
    if (!engine?.installation) {
      throw new Error('PiDriver: engine installation missing')
    }

    // ---- Environment: inject every configured Jedi provider key pi knows. ----
    const env: Record<string, string> = { PI_OFFLINE: '1', PI_TELEMETRY: '0' }
    const providerConfig = useProviderConfigStore()
    if (!providerConfig.configuredProviders.length) {
      await providerConfig.loadConfiguredProviders()
    }
    for (const configured of providerConfig.configuredProviders) {
      const envName = PROVIDER_ENV[configured.provider]
      if (!configured.has_key || !envName) continue
      try {
        const info = await providerConfig.getApiKey(configured.provider)
        if (info?.key) env[envName] = info.key
      } catch {
        /* best-effort: skip providers we cannot read */
      }
    }

    // ---- Model: pass the Jedi-selected provider/model when pi knows it. ----
    const args: string[] = ['--mode', 'rpc']
    const prefix = PROVIDER_MODEL_PREFIX[opts.provider]
    if (prefix && opts.model && !opts.model.includes('/')) {
      args.push('--model', `${prefix}/${opts.model}`)
    }
    const resumedSession = opts.nativeSessionId ?? piSessionByJediSession.get(opts.sessionId)
    if (resumedSession) args.push('--session', resumedSession)

    // ---- Spawn through the Rust process manager (strict \n framing). ----
    const procId = `pi-${opts.sessionId}-${Date.now()}`
    const translator = createPiTranslator(hooks)
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
          translator.handleRecord(JSON.parse(line))
        } catch {
          /* non-JSON line (banner etc.) — ignore */
        }
      } else if (msg.type === 'exit') {
        if (!settled) {
          finish(new Error(`pi process exited unexpectedly (code ${msg.code ?? '?'})`))
        }
      } else if (msg.type === 'stderr') {
        // pi keeps stdout protocol-only; stderr goes to the console.
        console.warn('[pi]', msg.line)
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
      throw new Error(`failed to spawn pi: ${e}`)
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
