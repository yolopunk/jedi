// OpencodeDriver: drives opencode (`opencode run --format json`) as a
// per-turn child process (docs §3.2 Tier B2).
//
// Verified on opencode 1.19.4 and 2.0.29 (docs §10): NDJSON events (step/text/tool_use/
// step_finish), every event carries sessionID, and `opencode run -s <id>`
// resumes across processes. The engine's free default model worked without
// any key configuration, making this an end-to-end usable engine. Auth:
// opencode's own config; no key injection needed.

import { Channel, invoke } from '@tauri-apps/api/core'
import type { RunAgentHooks, RunAgentResult } from '@/agent/runAgent'
import { nativeModel } from '../model'
import type { AgentDriver, DriverHandle, DriverStartOpts, EngineDescriptor } from '../types'
import { createAgentDiagnostics } from '../agentError'
import { createOpencodeTranslator } from './opencodeProtocol'

// opencode sessionID per Jedi chat session for `-s` resume continuity.
const opencodeSessionByJediSession = new Map<string, string>()

function lastUserText(messages: DriverStartOpts['messages']): string {
  const lastUser = [...messages].reverse().find(m => m.role === 'user')
  if (!lastUser) return ''
  if (typeof lastUser.content === 'string') return lastUser.content
  return (lastUser.content as Array<{ type: string; text?: string }>)
    .map(part => (part.type === 'text' ? (part.text ?? '') : ''))
    .join('')
}

/**
 * Model of the last message in an opencode session, via `session export`.
 * Best-effort: resolves '' on any failure or after a short timeout.
 */
function exportedModel(spec: unknown, procId: string, sessionId: string): Promise<string> {
  return new Promise(resolve => {
    const lines: string[] = []
    let finished = false
    const end = (model: string) => {
      if (finished) return
      finished = true
      clearTimeout(timer)
      void invoke('agent_proc_kill', { id: procId }).catch(() => {})
      resolve(model)
    }
    const timer = setTimeout(() => end(''), 15000)
    const channel = new Channel<any>()
    channel.onmessage = (msg: any) => {
      if (msg?.type === 'line') lines.push(String(msg.line ?? ''))
      else if (msg?.type === 'exit') {
        try {
          const raw = lines.join('\n')
          const data = JSON.parse(raw.slice(raw.indexOf('{')))
          const messages: any[] = Array.isArray(data?.messages) ? data.messages : []
          const model = [...messages].reverse().find(m => m?.model?.id)?.model
          end(model ? (model.providerID ? `${model.providerID}/${model.id}` : String(model.id)) : '')
        } catch {
          end('')
        }
      }
    }
    invoke('agent_proc_spawn', {
      id: procId,
      spec,
      args: ['session', 'export', sessionId],
      cwd: null,
      env: null,
      onEvent: channel,
    }).catch(() => end(''))
  })
}

export class OpencodeDriver implements AgentDriver {
  id = 'opencode'

  capabilities = {
    steering: false, // no headless steer command captured on 1.19.4
    compaction: false,
    images: false,
    resume: true,
    models: true,
    approval: 'none', // headless run auto-executed bash in probes
    usesJediKey: false, // free default model / own auth config
  } as const

  supports(engine: EngineDescriptor): boolean {
    return engine.agentId === 'opencode' && !!engine.installation
  }

  async start(opts: DriverStartOpts, hooks: RunAgentHooks): Promise<DriverHandle> {
    const engine = opts.engine
    if (!engine?.installation) {
      throw new Error('OpencodeDriver: engine installation missing')
    }

    const promptText = lastUserText(opts.messages)
    if (!promptText.trim()) {
      throw new Error('OpencodeDriver: no user message to send')
    }

    const args: string[] = ['run', promptText, '--format', 'json']
    const resumed = opts.nativeSessionId ?? opencodeSessionByJediSession.get(opts.sessionId)
    if (resumed) args.push('-s', resumed)

    const procId = `opencode-${opts.sessionId}-${Date.now()}`
    const translator = createOpencodeTranslator(hooks)
    const diag = createAgentDiagnostics(engine.name)
    let sawModel = false
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
      // Prefer the final message's text over the whole narration.
      const text = snap.finalText.trim() ? snap.finalText : snap.allText
      if (failure && !text.trim()) {
        rejectDone(failure)
        return
      }
      if (snap.lastError && !text.trim()) {
        rejectDone(diag.protocol(snap.lastError))
        return
      }
      if (snap.nativeSessionId) {
        opencodeSessionByJediSession.set(opts.sessionId, snap.nativeSessionId)
      }
      hooks.onFinish?.({ finishReason: snap.finishReason, usage: snap.usage })
      const result: RunAgentResult = {
        text,
        reasoning: snap.reasoning,
        finishReason: snap.finishReason,
        usage: snap.usage,
        toolCount: snap.toolCount,
        enabledToolNames: [],
        nativeSessionId: snap.nativeSessionId,
      }
      // The JSON stream never names the model (2.x); the session export does.
      if (!sawModel && snap.nativeSessionId) {
        void exportedModel(engine.installation!.spawn, `${procId}-export`, snap.nativeSessionId)
          .then(m => {
            if (m) hooks.onModel?.(m)
          })
          .finally(() => resolveDone(result))
      } else {
        resolveDone(result)
      }
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
          if (model) {
            sawModel = true
            hooks.onModel?.(model)
          }
          translator.handleRecord(record)
        } catch {
          // Non-JSON line — opencode prints failures like "Error: …".
          translator.noteErrorLine(line)
          const early = diag.line(line)
          if (early) finish(early)
        }
      } else if (msg.type === 'exit') {
        // One-shot mode: the process ending IS the turn boundary.
        finish(msg.code ? diag.exit(msg.code, translator.snapshot().lastError) : undefined)
      } else if (msg.type === 'stderr') {
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
      return { engineId: `opencode@${engine.host}`, capabilities: { ...this.capabilities }, done }
    }
    opts.signal?.addEventListener('abort', onAbort)

    return {
      engineId: `opencode@${engine.host}`,
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
