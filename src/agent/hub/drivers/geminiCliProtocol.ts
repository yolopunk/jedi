// Gemini-CLI-family translator — the pure half of GeminiCliDriver.
//
// Covers the gemini-cli lineage (Gemini CLI / Qwen Code / Cursor CLI share
// the same CLI surface). Output shapes captured on a real machine
// (docs §10, M4 Tier B session):
//  - gemini-cli 0.65.0 `gemini -p <prompt> -o json` prints ONE pretty-printed
//    JSON document: {session_id, response?, stats?, error?:{type,message,code}}
//    (captured live: auth error code 41 listing GEMINI_API_KEY etc).
//  - qwen-code 0.26.1 `qwen -p <prompt> -o json` prints a JSON ARRAY of
//    Claude-Code-style records (captured live):
//    [{type:"result",subtype:"error_during_execution",session_id,is_error,
//      usage:{input_tokens,output_tokens},error:{message}}]
//    Success records carry `result` text the same way.
//  - gemini stream-json: stdout stays empty pre-auth (observed), so the
//    driver consumes the whole stdout document at process exit instead of
//    token streaming. Continuity is conversation replay in the prompt
//    (`-r/--resume` is index-based, not id-based — unsuitable cross-process).
//
// Approval: `-p` headless with --approval-mode default makes unapproved tools
// fail with an error record (their gate); no approval roundtrip exists.

import type { ChatTurn, RunAgentHooks } from '../../runAgent'

export interface GeminiFamilyResult {
  text: string
  sessionId?: string
  usage?: unknown
  finishReason: string
  error: string | null
}

/** Parse the whole accumulated stdout of a `-o json` run (either variant). */
export function parseGeminiFamilyOutput(raw: string): GeminiFamilyResult {
  const trimmed = raw.trim()
  if (!trimmed) {
    return { text: '', finishReason: 'error', error: 'empty output from CLI' }
  }
  let parsed: unknown
  try {
    parsed = JSON.parse(trimmed)
  } catch {
    // Non-JSON output (e.g. plain-text failure on stderr that leaked in):
    return { text: trimmed, finishReason: 'error', error: null }
  }

  // qwen-style: array of Claude-Code-like records.
  if (Array.isArray(parsed)) {
    const result = [...parsed]
      .reverse()
      .find((r: any) => r && typeof r === 'object' && r.type === 'result')
    if (!result) {
      return { text: trimmed, finishReason: 'error', error: 'no result record in output' }
    }
    const isError = result.is_error === true || result.subtype !== 'success'
    return {
      text: typeof result.result === 'string' ? result.result : '',
      sessionId: typeof result.session_id === 'string' ? result.session_id : undefined,
      usage: result.usage
        ? {
            inputTokens: result.usage.input_tokens,
            outputTokens: result.usage.output_tokens,
          }
        : undefined,
      finishReason: isError ? 'error' : 'end_turn',
      error: isError ? String(result.error?.message ?? result.subtype ?? 'unknown error') : null,
    }
  }

  // gemini-style: single document.
  const doc = parsed as Record<string, any>
  if (doc && typeof doc === 'object') {
    const err = doc.error
    if (err && (err.message || err.code)) {
      return {
        text: '',
        sessionId: typeof doc.session_id === 'string' ? doc.session_id : undefined,
        finishReason: 'error',
        error: String(err.message ?? `error code ${err.code}`),
      }
    }
    const statsTotal = Object.values(doc.stats?.models ?? {}).reduce(
      (sum: number, m: any) => sum + (Number(m?.['models.totalTokenCount']) || 0),
      0
    )
    return {
      text: typeof doc.response === 'string' ? doc.response : '',
      sessionId: typeof doc.session_id === 'string' ? doc.session_id : undefined,
      usage: statsTotal > 0 ? { totalTokens: statsTotal } : undefined,
      finishReason: 'end_turn',
      error: null,
    }
  }
  return { text: trimmed, finishReason: 'error', error: 'unrecognized output shape' }
}

/**
 * Format the conversation for prompt-replay continuity: the engines above
 * have no id-based cross-process resume, so prior turns travel as transcript
 * text and the new user message is appended.
 */
export function buildConversationPrompt(messages: ChatTurn[]): string {
  const lines: string[] = []
  let lastUser = ''
  for (const m of messages) {
    const text =
      typeof m.content === 'string'
        ? m.content
        : (m.content as Array<{ type: string; text?: string }>)
            .map(p => (p.type === 'text' ? (p.text ?? '') : '[图片]'))
            .join('\n')
    if (!text.trim()) continue
    if (m.role === 'user') lastUser = text
    else if (m.role === 'assistant') lines.push(`Assistant: ${text}`)
  }
  if (lines.length === 0) return lastUser
  return ['Conversation so far (for context):', ...lines, '', `User: ${lastUser}`].join('\n')
}

/** Fire the unified hooks for a parsed final result (single-shot engines). */
export function emitGeminiFamilyResult(result: GeminiFamilyResult, hooks: RunAgentHooks): void {
  if (result.text) {
    hooks.onTextDelta?.({ fullContent: result.text, delta: result.text })
  }
  hooks.onStepFinish?.({
    stepNumber: 1,
    finishReason: result.finishReason,
    usage: result.usage,
    text: result.text,
    toolCalls: [],
    toolResults: [],
    stepKind: 'initial',
    stepText: result.text,
  })
  hooks.onFinish?.({ finishReason: result.finishReason, usage: result.usage })
}
