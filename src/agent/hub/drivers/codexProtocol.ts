// Codex app-server translator — the pure half of CodexDriver.
//
// Record shapes verified against codex-cli 0.162.1 on a real machine
// (docs §10, M4 session):
//  - wire: JSON-RPC 2.0 with the "jsonrpc" header OMITTED; requests carry
//    id/method/params, notifications arrive as {method, params, emittedAtMs}.
//  - handshake: initialize {clientInfo} → {result:{userAgent, codexHome…}}.
//  - conversation: thread/start {} (cwd optional, defaults to process cwd) →
//    result.thread.id; thread/resume {threadId} restores it (verified).
//  - turn: turn/start {threadId, input:[{type:"text",text}]} → result.turn.id
//    + turn/started; items stream as item/started|updated|completed with
//    item.type userMessage | agentMessage | reasoning | commandExecution…;
//    the turn is TERMINAL at turn/completed {turn:{status, error}}.
//  - failures surface as an `error` notification
//    ({error:{message, codexErrorInfo}}) + thread/status/changed
//    {status:{type:"systemError"}} before turn/completed (observed live:
//    usageLimitExceeded).
//  - approvals: this protocol version did not surface approval requests in
//    the probe session; exec-approval translation is a documented follow-up
//    once shapes are captured.

import type { RunAgentHooks } from '../../runAgent'

export interface CodexSnapshot {
  fullText: string
  reasoning: string
  usage: unknown
  finishReason: string | undefined
  toolCount: number
  stepCount: number
  nativeSessionId: string | undefined
  threadId: string | undefined
  turnId: string | undefined
  lastError: string | null
  turnComplete: boolean
}

function itemText(item: any): string {
  if (typeof item?.text === 'string') return item.text
  const content = item?.content
  if (typeof content === 'string') return content
  if (!Array.isArray(content)) return ''
  return content.map((part: any) => (typeof part?.text === 'string' ? part.text : '')).join('')
}

export function createCodexTranslator(hooks: RunAgentHooks) {
  let fullText = ''
  let reasoning = ''
  let lastError: string | null = null
  let usage: unknown
  let finishReason: string | undefined
  let stepCount = 0
  let toolCount = 0
  let nativeSessionId: string | undefined
  let threadId: string | undefined
  let turnId: string | undefined
  let turnComplete = false
  // Per-item text already emitted, so item/updated re-broadcasts become
  // incremental deltas instead of duplicating the whole message.
  const emittedTextById = new Map<string, string>()

  function handleRecord(record: any): void {
    if (!record || typeof record !== 'object') return

    // --- JSON-RPC responses (id-correlated) ---
    if ('id' in record && record.id !== undefined && record.id !== null) {
      if (record.error) {
        lastError = String(record.error.message ?? JSON.stringify(record.error))
        if (!turnComplete) {
          turnComplete = true
          finishReason = 'error'
        }
        return
      }
      const result = record.result
      if (result?.thread?.id) {
        threadId = String(result.thread.id)
        if (!nativeSessionId) nativeSessionId = threadId
      }
      if (result?.turn?.id) turnId = String(result.turn.id)
      return
    }

    const method = record.method as string | undefined
    if (!method) return

    if (method === 'thread/started' || method === 'thread/resumed') {
      const id = record.params?.thread?.id
      if (id) {
        threadId = String(id)
        nativeSessionId = threadId
      }
      return
    }

    if (method === 'item/agentMessage/delta') {
      const delta = String(record.params?.delta ?? '')
      const id = String(record.params?.itemId ?? '')
      emittedTextById.set(id, (emittedTextById.get(id) ?? '') + delta)
      fullText += delta
      if (delta) hooks.onTextDelta?.({ fullContent: fullText, delta })
      return
    }

    if (method === 'turn/started') {
      const id = record.params?.turn?.id
      if (id) turnId = String(id)
      return
    }

    if (method === 'item/started' || method === 'item/updated' || method === 'item/completed') {
      const item = record.params?.item ?? {}
      const itemId = String(item.id ?? `${item.type}-${toolCount}`)
      const text = itemText(item)

      if (item.type === 'agentMessage') {
        if (method === 'item/started') {
          emittedTextById.set(itemId, '')
          return
        }
        const already = emittedTextById.get(itemId) ?? ''
        if (text.length > already.length && text.startsWith(already)) {
          const delta = text.slice(already.length)
          emittedTextById.set(itemId, text)
          fullText += delta
          hooks.onTextDelta?.({ fullContent: fullText, delta })
        } else if (method === 'item/completed' && text && text !== already) {
          // Content replaced (rare); emit the difference defensively.
          emittedTextById.set(itemId, text)
          fullText += text
          hooks.onTextDelta?.({ fullContent: fullText, delta: text })
        }
        return
      }

      if (item.type === 'reasoning' || item.type === 'thinking') {
        if (text) {
          reasoning += text
          hooks.onReasoningDelta?.({ fullReasoning: reasoning, delta: text })
        }
        return
      }

      if (
        item.type === 'commandExecution' ||
        item.type === 'fileChange' ||
        item.type === 'mcpToolCall'
      ) {
        const toolName =
          item.type === 'commandExecution'
            ? 'bash'
            : item.type === 'fileChange'
              ? 'edit'
              : item.type
        if (method === 'item/started') {
          toolCount += 1
          const args =
            item.type === 'commandExecution' ? { command: item.command } : (item.extra ?? item)
          hooks.onToolCall?.({ toolName, toolCallId: itemId, input: args })
          hooks.onToolStart?.({
            toolId: itemId,
            toolName,
            toolDescription: item.type,
            args,
            startedAt: Date.now(),
          })
        } else if (method === 'item/completed') {
          const output =
            item.type === 'commandExecution'
              ? (item.aggregatedOutput ?? item.output)
              : (item.extra ?? item)
          hooks.onToolResult?.({ toolName, toolCallId: itemId, output })
          hooks.onToolEnd?.({
            toolId: itemId,
            toolName,
            args: undefined,
            startedAt: Date.now(),
            output,
            ...(item.exitCode !== undefined && item.exitCode !== null && Number(item.exitCode) !== 0
              ? { error: `exit code ${item.exitCode}` }
              : {}),
          })
        }
        return
      }

      if (item.type === 'error') {
        lastError = text || item.message || 'unknown error'
        return
      }
      return // userMessage and informational items carry no hook
    }

    if (method === 'error') {
      const err = record.params?.error ?? {}
      lastError = String(err.message ?? err.codexErrorInfo ?? JSON.stringify(err))
      return
    }

    if (method === 'thread/status/changed') {
      const type = record.params?.status?.type
      if (type === 'systemError' && !lastError) {
        lastError = 'thread system error'
      }
      return
    }

    if (method === 'turn/completed') {
      turnComplete = true
      stepCount += 1
      const turn = record.params?.turn ?? {}
      const status = String(turn.status ?? 'completed')
      if (turn.error?.message) lastError = String(turn.error.message)
      finishReason =
        status === 'failed' ? 'error' : status === 'aborted' ? 'aborted' : (finishReason ?? 'stop')
      if (turn.tokenCount) usage = turn.tokenCount
      hooks.onStepFinish?.({
        stepNumber: stepCount,
        finishReason,
        usage,
        text: fullText,
        toolCalls: [],
        toolResults: [],
        stepKind: toolCount > 0 ? 'tool-result' : 'initial',
        stepText: fullText,
      })
      return
    }
    // account/*, mcpServer/*, hook/*, remoteControl/*, deprecationNotice…:
    // no hook slot yet.
  }

  function snapshot(): CodexSnapshot {
    return {
      fullText,
      reasoning,
      usage,
      finishReason,
      toolCount,
      stepCount,
      nativeSessionId,
      threadId,
      turnId,
      lastError,
      turnComplete,
    }
  }

  return { handleRecord, snapshot }
}
