// opencode run --format json translator — the pure half of OpencodeDriver.
//
// Record shapes captured live on opencode 1.19.4 (docs §10, Tier B2 session);
// the engine's free default model makes this an end-to-end working engine.
// Each stdout line is one event:
//  - {"type":"step_start", sessionID, part:{type:"step-start"}}
//  - {"type":"text", sessionID, part:{id, messageID, type:"text", text}}
//    (per-part full chunks; several parts per message; dedupe by part.id)
//  - {"type":"tool_use", sessionID, part:{type:"tool", tool:"bash", callID,
//      state:{status:"completed"|"pending", input, output, metadata:{exit}}}}
//  - {"type":"step_finish", sessionID, part:{reason:"stop"|"tool-calls",
//      tokens:{total,input,output,reasoning,cache:{write,read}}, cost}}
// Session continuity: every event carries sessionID; `opencode run -s <id>`
// resumes it (verified live: recalled a tool command across processes).
// Failures print non-JSON "Error: …" lines (handled by the driver).

import type { RunAgentHooks } from '../../runAgent'

export interface OpencodeSnapshot {
  /** Concat of every streamed text part (includes step narration). */
  allText: string
  /** Text of the final assistant message only (the canonical answer). */
  finalText: string
  reasoning: string
  usage: unknown
  finishReason: string | undefined
  toolCount: number
  stepCount: number
  nativeSessionId: string | undefined
  lastError: string | null
}

export function createOpencodeTranslator(hooks: RunAgentHooks) {
  let allText = ''
  let reasoning = ''
  let usage: unknown
  let finishReason: string | undefined
  let stepCount = 0
  let toolCount = 0
  let nativeSessionId: string | undefined
  let lastError: string | null = null
  let lastMessageId: string | undefined
  // messageID -> ordered text parts, so the final message can be rebuilt.
  const partsByMessage = new Map<string, string[]>()
  const emittedPartIds = new Set<string>()

  function handleRecord(record: any): void {
    const type = record?.type as string | undefined
    if (!type) return
    if (typeof record.sessionID === 'string' && record.sessionID) {
      nativeSessionId = record.sessionID
    }

    if (type === 'text') {
      const part = record.part ?? {}
      const partId = String(part.id ?? '')
      const text = typeof part.text === 'string' ? part.text : ''
      if (!text || (partId && emittedPartIds.has(partId))) return
      if (partId) emittedPartIds.add(partId)
      allText += text
      if (typeof part.messageID === 'string' && part.messageID) {
        lastMessageId = part.messageID
        const list = partsByMessage.get(part.messageID) ?? []
        list.push(text)
        partsByMessage.set(part.messageID, list)
      }
      hooks.onTextDelta?.({ fullContent: allText, delta: text })
      return
    }

    if (type === 'reasoning') {
      const text = typeof record.part?.text === 'string' ? record.part.text : ''
      if (text) {
        reasoning += text
        hooks.onReasoningDelta?.({ fullReasoning: reasoning, delta: text })
      }
      return
    }

    if (type === 'tool_use') {
      const part = record.part ?? {}
      const state = part.state ?? {}
      const toolName = String(part.tool ?? 'tool')
      const callId = String(part.callID ?? part.id ?? toolName)
      const status = String(state.status ?? 'completed')
      if (status === 'pending') {
        toolCount += 1
        hooks.onToolCall?.({ toolName, toolCallId: callId, input: state.input })
        hooks.onToolStart?.({
          toolId: callId,
          toolName,
          toolDescription: state.title ?? '',
          args: state.input,
          startedAt: Date.now(),
        })
      } else {
        // Completed events may arrive without a prior pending broadcast:
        // fire the whole chain once.
        if (!emittedPartIds.has(`tool:${callId}`)) {
          emittedPartIds.add(`tool:${callId}`)
          toolCount += 1
          hooks.onToolCall?.({ toolName, toolCallId: callId, input: state.input })
          hooks.onToolStart?.({
            toolId: callId,
            toolName,
            toolDescription: state.title ?? '',
            args: state.input,
            startedAt: Date.now(),
          })
        }
        const output = state.output ?? state.metadata?.output
        const exit = state.metadata?.exit
        hooks.onToolResult?.({ toolName, toolCallId: callId, output })
        hooks.onToolEnd?.({
          toolId: callId,
          toolName,
          args: undefined,
          startedAt: Date.now(),
          output,
          ...(exit !== undefined && Number(exit) !== 0 ? { error: `exit code ${exit}` } : {}),
        })
      }
      return
    }

    if (type === 'step_start') {
      return
    }

    if (type === 'step_finish') {
      stepCount += 1
      const part = record.part ?? {}
      const reason = String(part.reason ?? 'stop')
      finishReason = reason === 'stop' ? 'stop' : reason
      if (part.tokens) {
        usage = {
          inputTokens: part.tokens.input,
          outputTokens: part.tokens.output,
          reasoningTokens: part.tokens.reasoning,
          ...(part.cost !== undefined ? { totalCostUsd: part.cost } : {}),
        }
      }
      hooks.onStepFinish?.({
        stepNumber: stepCount,
        finishReason,
        usage,
        text: allText,
        toolCalls: [],
        toolResults: [],
        stepKind: toolCount > 0 ? 'tool-result' : stepCount === 1 ? 'initial' : 'continue',
        stepText: allText,
      })
      return
    }
    // Unknown event types: ignore.
  }

  /** Note a non-JSON line (e.g. "Error: …") surfaced by the driver. */
  function noteErrorLine(line: string): void {
    const trimmed = line.trim()
    if (trimmed && !lastError) lastError = trimmed
  }

  function snapshot(): OpencodeSnapshot {
    const finalParts = lastMessageId ? partsByMessage.get(lastMessageId) : undefined
    const finalText = (finalParts ?? []).join('')
    return {
      allText,
      finalText,
      reasoning,
      usage,
      finishReason,
      toolCount,
      stepCount,
      nativeSessionId,
      lastError,
    }
  }

  return { handleRecord, noteErrorLine, snapshot }
}
