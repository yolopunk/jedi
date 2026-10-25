// Antigravity CLI (agy) translator — the pure half of AntigravityDriver.
//
// Record shape captured live on agy 1.4.7 (docs §10): NDJSON lines with an
// `event` envelope. The terminal record is
//   {"event":"result","result":{conversation_id, status:"OK"|"ERROR",
//    response, error, duration_seconds, num_turns,
//    usage:{input_tokens,output_tokens,thinking_tokens,cache_read_tokens,
//    total_tokens}}}
// (captured live: unauthenticated run returns status ERROR with
//  "authentication failed or timed out" after agy's internal 60s wait).
// Pre-auth stderr carries the Google OAuth URL + code-paste prompt, which is
// why the driver never writes to stdin. Mid-run streaming event types are
// unknown until first login; the translator treats `result` as authoritative
// and passes through text-ish events defensively.

import type { RunAgentHooks } from '../../runAgent'

export interface AntigravitySnapshot {
  fullText: string
  usage: unknown
  finishReason: string | undefined
  toolCount: number
  nativeSessionId: string | undefined
  lastError: string | null
  turnComplete: boolean
}

export function createAntigravityTranslator(hooks: RunAgentHooks) {
  let fullText = ''
  let usage: unknown
  let finishReason: string | undefined
  let toolCount = 0
  let nativeSessionId: string | undefined
  let lastError: string | null = null
  let turnComplete = false

  function handleRecord(record: any): void {
    const event = record?.event as string | undefined
    if (!event) return

    if (event === 'text' || event === 'message' || event === 'delta') {
      // Defensive: pre-login streaming shapes are unverified; result below is
      // authoritative, so only surface text that is not already covered.
      const text = String(record.text ?? record.message?.text ?? record.delta ?? '')
      if (text && !fullText) {
        fullText = text
        hooks.onTextDelta?.({ fullContent: fullText, delta: text })
      }
      return
    }

    if (event === 'tool_use' || event === 'tool') {
      toolCount += 1
      const toolName = String(record.tool ?? record.name ?? 'tool')
      hooks.onToolCall?.({ toolName, toolCallId: record.id, input: record.input })
      hooks.onToolStart?.({
        toolId: String(record.id ?? toolName),
        toolName,
        toolDescription: '',
        args: record.input,
        startedAt: Date.now(),
      })
      return
    }

    if (event === 'tool_result') {
      const toolName = String(record.tool ?? record.name ?? 'tool')
      hooks.onToolResult?.({ toolName, toolCallId: record.id, output: record.output })
      hooks.onToolEnd?.({
        toolId: String(record.id ?? toolName),
        toolName,
        args: undefined,
        startedAt: Date.now(),
        output: record.output,
        ...(record.error ? { error: String(record.error) } : {}),
      })
      return
    }

    if (event === 'result') {
      turnComplete = true
      const r = record.result ?? {}
      if (typeof r.conversation_id === 'string' && r.conversation_id) {
        nativeSessionId = r.conversation_id
      }
      const ok = String(r.status ?? 'OK') === 'OK' && !r.error
      if (r.usage && (r.usage.input_tokens || r.usage.output_tokens)) {
        usage = {
          inputTokens: r.usage.input_tokens,
          outputTokens: r.usage.output_tokens,
          reasoningTokens: r.usage.thinking_tokens,
        }
      }
      finishReason = ok ? 'stop' : 'error'
      if (ok && typeof r.response === 'string' && r.response) {
        fullText = r.response
        hooks.onTextDelta?.({ fullContent: fullText, delta: r.response })
      }
      if (!ok) lastError = String(r.error ?? r.status ?? 'unknown error')
      hooks.onStepFinish?.({
        stepNumber: 1,
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
  }

  function snapshot(): AntigravitySnapshot {
    return {
      fullText,
      usage,
      finishReason,
      toolCount,
      nativeSessionId,
      lastError,
      turnComplete,
    }
  }

  return { handleRecord, snapshot }
}
