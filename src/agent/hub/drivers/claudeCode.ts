// ClaudeCodeDriver: per-turn stream-json process using Claude's own auth.
// Legacy calls retain their defaults; explicit discussion is restricted by
// the CLI tool allowlist and permission mode, not a Jedi/OS sandbox.

import { Channel, invoke } from '@tauri-apps/api/core'
import type { RunAgentHooks, RunAgentResult } from '@/agent/runAgent'
import { createAgentDiagnostics } from '../agentError'
import { nativeModel } from '../model'
import type { AgentDriver, DriverHandle, DriverStartOpts, EngineDescriptor } from '../types'
import {
  type ClaudeApprovalRequest,
  controlResponseFor,
  createClaudeTranslator,
} from './claudeCodeProtocol'
import {
  activeRunHooks,
  claudeGroupChatModes,
  claudeStartArgs,
  driverResumeId,
  driverSessionKey,
  validateDriverStart,
} from './groupPolicy'

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
    usesJediKey: false,
    groupChatModes: claudeGroupChatModes,
  } as const

  supports(engine: EngineDescriptor): boolean {
    return engine.agentId === 'claude-code' && !!engine.installation
  }

  async start(opts: DriverStartOpts, hooks: RunAgentHooks): Promise<DriverHandle> {
    // Register cancellation before awaiting spawn: the caller has no handle
    // yet and can only stop a starting run through its signal.
    if (opts.signal?.aborted) throw new Error('aborted')
    const engine = opts.engine
    if (!engine?.installation) {
      throw new Error('ClaudeCodeDriver: engine installation missing')
    }
    const cwd = validateDriverStart(opts, this.capabilities.groupChatModes)
    const lastUser = [...opts.messages].reverse().find(m => m.role === 'user')
    if (!lastUserText(opts.messages).trim() && !hasImageParts(lastUser?.content)) {
      throw new Error('ClaudeCodeDriver: no user message to send')
    }
    const sessionKey = driverSessionKey(engine, opts)
    const resumed = driverResumeId(opts, claudeSessionByJediSession.get(sessionKey))
    const args = claudeStartArgs(opts.executionMode)
    if (resumed) args.push('--resume', resumed)

    const procId = opts.processId ?? `claude-${crypto.randomUUID()}`
    const diag = createAgentDiagnostics(engine.name)
    let settled = false
    let spawned = false
    let stopRequested = false
    let killStarted = false
    let nativeSessionId: string | undefined
    const liveHooks = activeRunHooks(hooks, () => !settled)
    const approvalRequests = new Set<string>()

    let resolveDone: (r: RunAgentResult) => void = () => {}
    let rejectDone: (e: Error) => void = () => {}
    const done = new Promise<RunAgentResult>((resolve, reject) => {
      resolveDone = resolve
      rejectDone = reject
    })
    // A process can fail before start returns. The caller still receives the
    // original rejecting promise, without an unhandled-rejection window.
    void done.catch(() => {})

    const stopProcess = () => {
      stopRequested = true
      if (!spawned || killStarted) return
      killStarted = true
      void invoke('agent_proc_kill', { id: procId }).catch(() => {})
    }
    const send = (payload: Record<string, unknown>): Promise<void> => {
      if (settled || opts.signal?.aborted) return Promise.reject(new Error('claude run ended'))
      return invoke('agent_proc_stdin', { id: procId, line: JSON.stringify(payload) })
    }
    const saveNativeSessionId = (id: unknown) => {
      if (settled || typeof id !== 'string' || !id || id === nativeSessionId) return
      nativeSessionId = id
      claudeSessionByJediSession.set(sessionKey, id)
      liveHooks.onNativeSessionId?.(id)
    }

    const handleApproval = (req: ClaudeApprovalRequest) => {
      if (settled || !req.requestId || approvalRequests.has(req.requestId)) return
      approvalRequests.add(req.requestId)
      const decide = async () => {
        let allow = false
        // Discussion never asks or grants even if a newer CLI emits requests.
        if (opts.executionMode === undefined && opts.confirmTool && !opts.signal?.aborted) {
          try {
            allow =
              (await opts.confirmTool({
                toolId: `claude-code:${req.requestId}`,
                toolName: req.toolName,
                risk: 'write',
                args: req.toolInput,
              })) === true
          } catch {
            allow = false
          }
        }
        if (!settled && !opts.signal?.aborted) {
          await send(controlResponseFor(req.requestId, allow))
        }
      }
      void decide().catch(error => {
        if (!settled) finish(error instanceof Error ? error : new Error(String(error)))
      })
    }
    const translator = createClaudeTranslator({
      hooks: liveHooks,
      onApprovalRequest: handleApproval,
    })

    const finish = (failure?: Error, aborted = false) => {
      if (settled) return
      const snap = translator.snapshot()
      settled = true
      opts.signal?.removeEventListener('abort', onAbort)
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
      engineId: `claude-code@${engine.host}`,
      capabilities: { ...this.capabilities },
      done,
      steer: text =>
        send({
          type: 'user',
          message: { role: 'user', content: [{ type: 'text', text }] },
        }),
      abort: onAbort,
      dispose: onAbort,
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
          if (record?.type === 'system' && record.subtype === 'init') {
            // The tool list is an additional runtime tripwire. CLI flags are
            // still the enforcement; no write is ever executed by this driver.
            if (
              opts.executionMode === 'discussion' &&
              (record.permissionMode !== 'dontAsk' ||
                !Array.isArray(record.tools) ||
                record.tools.some(
                  (tool: unknown) =>
                    typeof tool !== 'string' || !['Read', 'Glob', 'Grep'].includes(tool)
                ))
            ) {
              finish(new Error('Claude Code did not confirm discussion tool restrictions'))
              return
            }
          }
          saveNativeSessionId(record?.session_id)
          if (settled) return
          const model = nativeModel(record)
          if (model) liveHooks.onModel?.(model)
          if (settled) return
          translator.handleRecord(record)
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
      await invoke<string | null | void>('agent_proc_spawn', {
        id: procId,
        spec: engine.installation.spawn,
        args,
        cwd: cwd ?? null,
        env: null,
        onEvent: channel,
      })
      spawned = true
      // Cancellation or an early result may arrive while invoke is pending.
      if (stopRequested || settled || opts.signal?.aborted) {
        if (!settled) onAbort()
        stopProcess()
        return handle
      }
      await send({
        type: 'user',
        message: { role: 'user', content: buildUserContentBlocks(lastUser?.content) },
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
