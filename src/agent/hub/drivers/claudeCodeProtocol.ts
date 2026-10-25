// Claude Code stream-json translator — the pure half of ClaudeCodeDriver.
//
// Record shapes verified against Claude Code 2.1.299 on a real machine
// (docs §10, M3 session):
//  - stdin (per turn): {"type":"user","message":{"role":"user","content":[{"type":"text","text":…}]}}
//  - system/init: session_id / model / permissionMode
//  - stream_event (with --include-partial-messages): event.type
//    content_block_delta, delta.type text_delta|thinking_delta carries tokens
//  - assistant: Anthropic envelope, content blocks text | tool_use{id,name,input}
//  - user: tool_result blocks {tool_use_id, content, is_error}
//  - result: terminal per turn — subtype / is_error / result / session_id /
//    usage / total_cost_usd
//  - control_request / sdk_control_request (permission): NOT emitted by
//    2.1.299 in -p mode (tools auto-approve); translator handles the shapes
//    documented by the Agent SDK so the gate activates when a version emits
//    them. Verified resume: --resume <sid> composes with stream-json input.

import type { RunAgentHooks } from '../../runAgent'

export interface ClaudeApprovalRequest {
  requestId: string
  toolName: string
  toolInput: unknown
}

export interface ClaudeTranslatorOptions {
  hooks: RunAgentHooks
  /** Fired when a permission control_request arrives (dormant in 2.1.299 -p). */
  onApprovalRequest?: (req: ClaudeApprovalRequest) => void
}

export interface ClaudeSnapshot {
  fullText: string
  reasoning: string
  usage: unknown
  finishReason: string | undefined
  toolCount: number
  stepCount: number
  nativeSessionId: string | undefined
  lastError: string | null
  /** True once a `result` record arrived (one per turn in -p mode). */
  turnComplete: boolean
  isTerminalError: boolean
}

function blockText(content: unknown): string {
  if (typeof content === 'string') return content
  if (!Array.isArray(content)) return ''
  return (content as Array<Record<string, unknown>>)
    .map(part => (typeof part?.text === 'string' ? part.text : ''))
    .join('')
}

export function createClaudeTranslator(options: ClaudeTranslatorOptions) {
  const { hooks, onApprovalRequest } = options
  let fullText = ''
  let reasoning = ''
  let lastError: string | null = null
  let usage: unknown
  let finishReason: string | undefined
  let stepCount = 0
  let toolCount = 0
  let nativeSessionId: string | undefined
  let turnComplete = false
  let isTerminalError = false
  let sawTextDelta = false
  // tool_result blocks carry only tool_use_id; remember the name from the
  // matching tool_use block so hooks get a human-readable toolName.
  const toolNameById = new Map<string, string>()

  function handleRecord(record: any): void {
    const type = record?.type as string | undefined
    if (!type) return

    if (type === 'system' && record.subtype === 'init') {
      if (typeof record.session_id === 'string' && record.session_id) {
        nativeSessionId = record.session_id
      }
      return
    }

    // Token-level streaming (--include-partial-messages).
    if (type === 'stream_event') {
      const event = record.event ?? {}
      if (event.type === 'content_block_delta') {
        const delta = event.delta ?? {}
        if (delta.type === 'text_delta' && typeof delta.text === 'string') {
          sawTextDelta = true
          fullText += delta.text
          hooks.onTextDelta?.({ fullContent: fullText, delta: delta.text })
        } else if (
          (delta.type === 'thinking_delta' || delta.type === 'reasoning_delta') &&
          typeof delta.thinking === 'string'
        ) {
          reasoning += delta.thinking
          hooks.onReasoningDelta?.({ fullReasoning: reasoning, delta: delta.thinking })
        }
      }
      return
    }

    if (type === 'assistant') {
      const message = record.message ?? {}
      const blocks: any[] = Array.isArray(message.content) ? message.content : []
      for (const block of blocks) {
        if (block?.type === 'tool_use') {
          toolCount += 1
          const toolName = String(block.name ?? 'unknown')
          if (typeof block.id === 'string') toolNameById.set(block.id, toolName)
          hooks.onToolCall?.({ toolName, toolCallId: block.id, input: block.input })
          hooks.onToolStart?.({
            toolId: String(block.id ?? toolName),
            toolName,
            toolDescription: '',
            args: block.input,
            startedAt: Date.now(),
          })
        }
      }
      // No-delta providers: surface full text once from the envelope.
      if (!sawTextDelta) {
        const text = blockText(message.content)
        if (text && text !== fullText) {
          fullText = text
          hooks.onTextDelta?.({ fullContent: fullText, delta: text })
        }
      }
      if (message.usage?.output_tokens !== undefined) usage = message.usage
      if (message.stop_reason) finishReason = String(message.stop_reason)
      return
    }

    if (type === 'user') {
      const blocks: any[] = Array.isArray(record.message?.content) ? record.message.content : []
      for (const block of blocks) {
        if (block?.type === 'tool_result') {
          const toolName = String(
            toolNameById.get(String(block.tool_use_id)) ?? block.tool_use_id ?? 'tool'
          )
          const isError = block.is_error === true
          const output =
            typeof block.content === 'string' ? block.content : blockText(block.content)
          hooks.onToolResult?.({ toolName, toolCallId: block.tool_use_id, output })
          hooks.onToolEnd?.({
            toolId: String(block.tool_use_id ?? toolName),
            toolName,
            args: undefined,
            startedAt: Date.now(),
            output,
            ...(isError ? { error: String(output).slice(0, 200) } : {}),
          })
        }
      }
      return
    }

    // Permission gate — dormant in 2.1.299 -p (auto-approve), shapes per the
    // Agent SDK control protocol so the confirmation card lights up the day a
    // version starts emitting them.
    if (type === 'control_request' || type === 'sdk_control_request') {
      const req = record.request ?? record
      const subtype = req?.subtype ?? req?.payload?.subtype
      if (
        subtype === 'permission' ||
        subtype === 'can_use_tool' ||
        req?.type === 'tool_permission_request' ||
        typeof req?.tool_name === 'string'
      ) {
        const requestId = String(req.request_id ?? req.id ?? record.request_id ?? record.id ?? '')
        const toolName = String(req.tool_name ?? req.payload?.tool_name ?? 'unknown')
        const toolInput = req.tool_input ?? req.payload?.tool_input ?? req.input
        onApprovalRequest?.({ requestId, toolName, toolInput })
      }
      return
    }

    if (type === 'result') {
      turnComplete = true
      stepCount += 1
      if (typeof record.session_id === 'string' && record.session_id) {
        nativeSessionId = record.session_id
      }
      const resultText = typeof record.result === 'string' ? record.result : ''
      // Prefer the final result text: it is the turn's canonical answer.
      if (resultText.trim()) {
        if (resultText !== fullText) {
          fullText = resultText
          hooks.onTextDelta?.({ fullContent: fullText, delta: resultText })
        }
      }
      const subtype = String(record.subtype ?? 'success')
      if (
        record.is_error === true ||
        subtype === 'error_during_execution' ||
        subtype === 'error_max_turns'
      ) {
        isTerminalError = true
        lastError = resultText || subtype
        finishReason = subtype
      } else {
        finishReason = subtype === 'success' ? (finishReason ?? 'end_turn') : subtype
      }
      if (record.usage) {
        usage = {
          inputTokens: record.usage.input_tokens,
          outputTokens: record.usage.output_tokens,
          ...(record.total_cost_usd !== undefined ? { totalCostUsd: record.total_cost_usd } : {}),
        }
      }
      hooks.onStepFinish?.({
        stepNumber: stepCount,
        finishReason: finishReason ?? '',
        usage,
        text: fullText,
        toolCalls: [],
        toolResults: [],
        stepKind: toolCount > 0 ? 'tool-result' : 'initial',
        stepText: fullText,
      })
      return
    }
    // system (hooks/status), rate_limit_event: no hook slot yet.
  }

  function snapshot(): ClaudeSnapshot {
    return {
      fullText,
      reasoning,
      usage,
      finishReason,
      toolCount,
      stepCount,
      nativeSessionId,
      lastError,
      turnComplete,
      isTerminalError,
    }
  }

  return { handleRecord, snapshot }
}

/** The control_response envelope for an approval decision (Agent SDK shape). */
export function controlResponseFor(
  requestId: string,
  allow: boolean,
  denyReason?: string
): Record<string, unknown> {
  return {
    type: 'control_response',
    request_id: requestId,
    response: allow
      ? { behavior: 'allow' }
      : { behavior: 'deny', message: denyReason ?? '用户拒绝了该操作。' },
  }
}
