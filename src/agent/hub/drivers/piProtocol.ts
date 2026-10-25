// pi RPC event translator — the pure half of PiDriver.
//
// Maps pi `--mode rpc` records (verified against pi 1.4.0, docs §10) onto the
// shared RunAgentHooks shape. Kept free of Tauri/Vue imports so it can be
// replayed against captured event streams in scripts/pi-protocol-replay.ts.

import type { RunAgentHooks } from '../../runAgent'

export function textOfContent(content: unknown): string {
  if (typeof content === 'string') return content
  if (!Array.isArray(content)) return ''
  return (content as Array<Record<string, unknown>>)
    .map(part => (typeof part?.text === 'string' ? part.text : ''))
    .join('')
}

export interface PiSnapshot {
  fullText: string
  reasoning: string
  usage: unknown
  finishReason: string | undefined
  toolCount: number
  stepCount: number
  piSessionId: string | undefined
  lastError: string | null
}

/**
 * Consumes one parsed pi RPC record (response / event). The caller owns the
 * process lifecycle and decides settlement; this only accumulates state and
 * fires hooks.
 */
export function createPiTranslator(hooks: RunAgentHooks) {
  let fullText = ''
  let reasoning = ''
  let lastError: string | null = null
  let usage: unknown
  let finishReason: string | undefined
  let stepCount = 0
  let toolCount = 0
  let piSessionId: string | undefined

  function handleRecord(record: any): void {
    const type = record?.type as string | undefined
    if (!type) return

    if (type === 'response') {
      if (record.command === 'get_state' && record.success) {
        const sid = record.data?.sessionId
        if (typeof sid === 'string' && sid) piSessionId = sid
      }
      return
    }

    if (type === 'message_update') {
      const ev = record.assistantMessageEvent ?? {}
      const delta = String(ev.delta ?? ev.text ?? '')
      if (ev.type === 'text_delta') {
        if (delta) {
          fullText += delta
          hooks.onTextDelta?.({ fullContent: fullText, delta })
        }
      } else if (String(ev.type).includes('reasoning') || String(ev.type).includes('thinking')) {
        if (delta) {
          reasoning += delta
          hooks.onReasoningDelta?.({ fullReasoning: reasoning, delta })
        }
      }
      return
    }

    if (type === 'tool_execution_start') {
      toolCount += 1
      const toolName = String(record.toolName ?? 'unknown')
      hooks.onToolCall?.({ toolName, toolCallId: record.toolCallId, input: record.args })
      hooks.onToolStart?.({
        toolId: String(record.toolCallId ?? toolName),
        toolName,
        toolDescription: '',
        args: record.args,
        startedAt: Date.now(),
      })
      return
    }

    if (type === 'tool_execution_end') {
      const toolName = String(record.toolName ?? 'unknown')
      hooks.onToolResult?.({ toolName, toolCallId: record.toolCallId, output: record.result })
      hooks.onToolEnd?.({
        toolId: String(record.toolCallId ?? toolName),
        toolName,
        args: undefined,
        startedAt: Date.now(),
        output: record.result,
      })
      return
    }

    if (type === 'message_end' && record.message?.role === 'assistant') {
      const m = record.message
      if (m.usage && (m.usage.input || m.usage.output)) usage = m.usage
      if (m.stopReason) finishReason = String(m.stopReason)
      if (m.errorMessage) lastError = String(m.errorMessage)
      // Some providers only surface the full text on message_end.
      const endText = textOfContent(m.content)
      if (!fullText.trim() && endText.trim()) {
        fullText = endText
        hooks.onTextDelta?.({ fullContent: fullText, delta: endText })
      }
      return
    }

    if (type === 'turn_end') {
      stepCount += 1
      hooks.onStepFinish?.({
        stepNumber: stepCount,
        finishReason: String(record.message?.stopReason ?? finishReason ?? ''),
        usage: record.message?.usage ?? usage,
        text: fullText,
        toolCalls: [],
        toolResults: [],
        stepKind: toolCount > 0 ? 'tool-result' : stepCount === 1 ? 'initial' : 'continue',
        stepText: fullText,
      })
      return
    }

    if (type === 'agent_settled') {
      if (!finishReason) finishReason = record.aborted ? 'aborted' : 'stop'
      return
    }
    // agent_start/end, auto_retry_start/end, entry_appended, message_start:
    // no hook slot yet (engine_status lands with the steering UI).
  }

  function snapshot(): PiSnapshot {
    return {
      fullText,
      reasoning,
      usage,
      finishReason,
      toolCount,
      stepCount,
      piSessionId,
      lastError,
    }
  }

  return { handleRecord, snapshot }
}
