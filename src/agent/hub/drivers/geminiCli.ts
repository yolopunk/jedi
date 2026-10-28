// GeminiCliDriver: one adapter for the gemini-cli lineage — Gemini CLI,
// Qwen Code and Cursor CLI share the same CLI surface (docs §3.2: one
// adapter covers the three Tier B engines).
//
// Mode: per-turn one-shot `-p <conversation> -o json`. The whole stdout is
// accumulated (pretty-printed document or record array) and parsed at process
// exit — token streaming is not available on this path pre-auth (verified:
// stream-json stdout stays empty), and `-r/--resume` is index-based rather
// than id-based, so continuity is conversation replay in the prompt.
// Auth: env keys (GEMINI_API_KEY / DASHSCOPE_API_KEY / QWEN_API_KEY …)
// inherited from the installed agent configuration.

import { Channel, invoke } from '@tauri-apps/api/core'
import type { RunAgentResult } from '@/agent/runAgent'
import { isWslHost, toWslPath } from '../paths'
import type { AgentDriver, DriverHandle, DriverStartOpts, EngineDescriptor } from '../types'
import {
  buildConversationPrompt,
  emitGeminiFamilyResult,
  parseGeminiFamilyOutput,
} from './geminiCliProtocol'

const SUPPORTED_AGENTS = new Set(['gemini-cli', 'qwen-code', 'cursor-cli'])

export class GeminiCliDriver implements AgentDriver {
  id = 'gemini-cli-family'

  capabilities = {
    steering: false,
    compaction: false,
    images: true, // via -f <file> (WSL paths translated)
    resume: false, // conversation replay instead of native resume
    models: true,
    approval: 'none', // headless default mode fails unapproved tools
    usesJediKey: false, // own configuration / login
  } as const

  supports(engine: EngineDescriptor): boolean {
    return SUPPORTED_AGENTS.has(engine.agentId) && !!engine.installation
  }

  async start(opts: DriverStartOpts, hooks: any): Promise<DriverHandle> {
    const engine = opts.engine
    if (!engine?.installation) {
      throw new Error('GeminiCliDriver: engine installation missing')
    }

    // Model and authentication come from the installed agent.
    const args: string[] = ['-p', buildConversationPrompt(opts.messages), '-o', 'json']
    // Image attachments via `-f <file>`; WSL engines need drvfs-translated
    // paths (docs §5.3 W5) — computed locally, no wslpath subprocess.
    if (opts.attachments?.length) {
      const translate = isWslHost(engine.host) ? (p: string) => toWslPath(p) ?? p : (p: string) => p
      for (const attachment of opts.attachments) {
        args.push('-f', translate(attachment))
      }
    }

    const procId = `${engine.agentId}-${opts.sessionId}-${Date.now()}`
    let settled = false
    const stdoutChunks: string[] = []

    let resolveDone: (r: RunAgentResult) => void = () => {}
    let rejectDone: (e: Error) => void = () => {}
    const done = new Promise<RunAgentResult>((resolve, reject) => {
      resolveDone = resolve
      rejectDone = reject
    })

    const finish = () => {
      if (settled) return
      settled = true
      opts.signal?.removeEventListener('abort', onAbort)
      const result = parseGeminiFamilyOutput(stdoutChunks.join('\n'))
      if (result.error && !result.text.trim()) {
        rejectDone(new Error(result.error))
        return
      }
      emitGeminiFamilyResult(result, hooks)
      resolveDone({
        text: result.text,
        reasoning: '',
        finishReason: result.finishReason,
        usage: result.usage,
        toolCount: 0,
        enabledToolNames: [],
        nativeSessionId: result.sessionId,
      })
    }

    function onAbort() {
      void invoke('agent_proc_kill', { id: procId }).catch(() => {})
      setTimeout(finish, 200)
    }

    const channel = new Channel<any>()
    channel.onmessage = (msg: any) => {
      if (!msg || typeof msg !== 'object') return
      if (msg.type === 'line') {
        stdoutChunks.push(String(msg.line ?? ''))
      } else if (msg.type === 'exit') {
        finish()
      } else if (msg.type === 'stderr') {
        console.warn(`[${engine.agentId}]`, msg.line)
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
      throw new Error(`failed to spawn ${engine.agentId}: ${e}`)
    }

    if (opts.signal?.aborted) {
      finish()
      return {
        engineId: `${engine.agentId}@${engine.host}`,
        capabilities: { ...this.capabilities },
        done,
      }
    }
    opts.signal?.addEventListener('abort', onAbort)

    return {
      engineId: `${engine.agentId}@${engine.host}`,
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
