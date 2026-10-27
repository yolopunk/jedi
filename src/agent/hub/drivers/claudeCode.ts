// ClaudeCodeDriver: drives Claude Code as a per-turn child process via its
// stream-json protocol (docs §3.2 Tier A).
//
// Invocation verified on 2.1.299 (docs §10, M3 session):
//   claude -p --input-format stream-json --output-format stream-json \
//     --verbose --include-partial-messages --permission-mode default \
//     [--resume <session_id>] [--model <id>]
//  - each turn: one user-message JSONL line on stdin; the turn ends with a
//    `result` record (terminal for -p), carrying session_id for --resume.
//  - auth: Claude Code's own login state (~/.claude); no key injection.
//  - permissions: 2.1.299 auto-approves tools in -p mode and does NOT emit
//    control_request; the translator + confirmTool wiring below activates
//    automatically when a version starts emitting them (shapes implemented
//    per the Agent SDK control protocol).

import { Channel, invoke } from '@tauri-apps/api/core'
import type { RunAgentHooks, RunAgentResult } from '@/agent/runAgent'
import type { AgentDriver, DriverHandle, DriverStartOpts, EngineDescriptor } from '../types'
import {
  type ClaudeApprovalRequest,
  controlResponseFor,
  createClaudeTranslator,
} from './claudeCodeProtocol'

// claude session id per Jedi chat session for --resume continuity. In-memory
// for M3 (persisting in message metadata is a documented follow-up).
const claudeSessionByJediSession = new Map<string, string>()

function lastUserText(messages: DriverStartOpts['messages']): string {
  const lastUser = [...messages].reverse().find(m => m.role === 'user')
  if (!lastUser) return ''
  if (typeof lastUser.content === 'string') return lastUser.content
  return (lastUser.content as Array<{ type: string; text?: string }>)
    .map(part => (part.type === 'text' ? (part.text ?? '') : ''))
    .join('')
}

/** Anthropic base64 image block from a `data:<mime>;base64,<data>` URL. */
function dataUrlToImageBlock(dataUrl: string): Record<string, unknown> | null {
  const match = /^data:([^;,]+);base64,(.+)$/s.exec(dataUrl)
  if (!match) return null
  return {
    type: 'image',
    source: { type: 'base64', media_type: match[1], data: match[2] },
  }
}

function hasImageParts(content: unknown): boolean {
  return (
    Array.isArray(content) && content.some(part => (part as { type?: string })?.type === 'image')
  )
}

/** ChatTurn content -> claude stream-json user content blocks (text + image). */
function buildUserContentBlocks(content: unknown): Array<Record<string, unknown>> {
  if (typeof content === 'string') return [{ type: 'text', text: content }]
  if (!Array.isArray(content)) return [{ type: 'text', text: '' }]
  const blocks: Array<Record<string, unknown>> = []
  for (const part of content as Array<{ type: string; text?: string; image?: string }>) {
    if (part?.type === 'text' && part.text) {
      blocks.push({ type: 'text', text: part.text })
    } else if (part?.type === 'image' && part.image) {
      blocks.push(dataUrlToImageBlock(part.image) ?? { type: 'text', text: '[无法解析的图片]' })
    }
  }
  return blocks.length ? blocks : [{ type: 'text', text: '' }]
}

export class ClaudeCodeDriver implements AgentDriver {
  id = 'claude-code'

  capabilities = {
    steering: true,
    compaction: true,
    images: true,
    resume: true,
    models: true,
    approval: 'native',
    usesJediKey: false, // own login state (~/.claude)
  } as const

  supports(engine: EngineDescriptor): boolean {
    return engine.agentId === 'claude-code' && !!engine.installation
  }

  async start(opts: DriverStartOpts, hooks: RunAgentHooks): Promise<DriverHandle> {
    const engine = opts.engine
    if (!engine?.installation) {
      throw new Error('ClaudeCodeDriver: engine installation missing')
    }

    const args: string[] = [
      '-p',
      '--input-format',
      'stream-json',
      '--output-format',
      'stream-json',
      '--verbose',
      '--include-partial-messages',
      '--permission-mode',
      'default',
    ]
    // Only forward the Jedi-selected model when it is a Claude family id —
    // Claude Code rejects foreign model ids.
    if (opts.model && /claude/i.test(opts.model)) {
      args.push('--model', opts.model)
    }
    const resumedSession = opts.nativeSessionId ?? claudeSessionByJediSession.get(opts.sessionId)
    if (resumedSession) args.push('--resume', resumedSession)

    const procId = `claude-${opts.sessionId}-${Date.now()}`
    let settled = false

    let resolveDone: (r: RunAgentResult) => void = () => {}
    let rejectDone: (e: Error) => void = () => {}
    const done = new Promise<RunAgentResult>((resolve, reject) => {
      resolveDone = resolve
      rejectDone = reject
    })

    const send = (payload: Record<string, unknown>): Promise<void> =>
      invoke('agent_proc_stdin', { id: procId, line: JSON.stringify(payload) })

    const handleApproval = (req: ClaudeApprovalRequest) => {
      const decide = async () => {
        let allow = false
        if (opts.confirmTool) {
          allow = await opts.confirmTool({
            toolId: `claude-code:${req.toolName}`,
            toolName: req.toolName,
            risk: 'write',
            args: req.toolInput,
          })
        }
        await send(controlResponseFor(req.requestId, allow))
      }
      decide().catch(e => console.warn('[claude-code] approval response failed:', e))
    }

    const translator = createClaudeTranslator({ hooks, onApprovalRequest: handleApproval })

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
      if (snap.nativeSessionId) claudeSessionByJediSession.set(opts.sessionId, snap.nativeSessionId)
      hooks.onFinish?.({ finishReason: snap.finishReason, usage: snap.usage })
      resolveDone({
        text: snap.fullText,
        reasoning: snap.reasoning,
        finishReason: snap.finishReason,
        usage: snap.usage,
        toolCount: snap.toolCount,
        enabledToolNames: [],
        nativeSessionId: snap.nativeSessionId,
      })
    }

    function onAbort() {
      // No abort command on this protocol; terminating the process keeps the
      // partial output that already streamed through the hooks.
      finish()
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
          /* non-JSON line — ignore */
        }
        if (translator.snapshot().turnComplete) finish()
      } else if (msg.type === 'exit') {
        const snap = translator.snapshot()
        if (!settled && !snap.turnComplete) {
          finish(new Error(`claude process exited before result (code ${msg.code ?? '?'})`))
        }
      } else if (msg.type === 'stderr') {
        console.warn('[claude-code]', msg.line)
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
      throw new Error(`failed to spawn claude: ${e}`)
    }

    if (opts.signal?.aborted) {
      finish(new Error('aborted'))
      return {
        engineId: `claude-code@${engine.host}`,
        capabilities: { ...this.capabilities },
        done,
      }
    }
    opts.signal?.addEventListener('abort', onAbort)

    const lastUser = [...opts.messages].reverse().find(m => m.role === 'user')
    const promptText = lastUserText(opts.messages)
    if (!promptText.trim() && !hasImageParts(lastUser?.content)) {
      finish(new Error('ClaudeCodeDriver: no user message to send'))
    } else {
      // The last user message is the turn prompt; earlier history lives in
      // the resumed claude session (--resume) or starts a fresh conversation.
      // Image parts (data URLs built by the chat store) become Anthropic
      // base64 content blocks — no filesystem path needed, WSL included.
      await send({
        type: 'user',
        message: { role: 'user', content: buildUserContentBlocks(lastUser?.content) },
      })
    }

    return {
      engineId: `claude-code@${engine.host}`,
      capabilities: { ...this.capabilities },
      done,
      // Bidirectional stream-json: an extra user message mid-turn steers the
      // agent (it is delivered after the current tool batch finishes).
      steer: (text: string) =>
        send({
          type: 'user',
          message: { role: 'user', content: [{ type: 'text', text }] },
        }),
      abort: onAbort,
      dispose: () => {
        opts.signal?.removeEventListener('abort', onAbort)
        void invoke('agent_proc_kill', { id: procId }).catch(() => {})
      },
    }
  }
}
