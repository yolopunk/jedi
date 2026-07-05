// Reusable single-agent executor.
//
// This is the shared core that drives one LLM "run": it builds the provider
// client (routed through Tauri HTTP so it is not blocked by the webview CORS
// policy), exposes enabled skills as tools, streams the response, and emits
// granular events through `hooks`. Both the interactive chat (`useAiChatStore`)
// and the background worker pool (`useAgentPoolStore`) consume this so they
// share identical behavior.

import { createAnthropic } from '@ai-sdk/anthropic'
import { createOpenAI } from '@ai-sdk/openai'
import { createOpenAICompatible } from '@ai-sdk/openai-compatible'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { generateText, isLoopFinished, jsonSchema, stepCountIs, streamText, tool } from 'ai'
import { skillRegistry } from '@/skills/registry'

export const DEFAULT_STEP_LIMIT = 8

export interface ChatTurn {
  role: 'user' | 'assistant' | 'system'
  content: string
}

export interface RunAgentSpec {
  provider: string
  model: string
  apiKey: string
  endpoint?: string
  messages: ChatTurn[]
  sessionId: string
  stepLimit?: number
  signal?: AbortSignal
}

export interface RunAgentHooks {
  onToolStart?: (e: {
    skillId: string
    skillName: string
    skillDescription: string
    args: unknown
    startedAt: number
  }) => void
  onToolEnd?: (e: {
    skillId: string
    skillName: string
    args: unknown
    startedAt: number
    output?: unknown
    error?: string
  }) => void
  onStepFinish?: (e: {
    stepNumber: number
    finishReason: string
    usage: unknown
    text: string
    toolCalls: unknown
    toolResults: unknown
    stepKind: 'tool-result' | 'initial' | 'continue'
    stepText: string
  }) => void
  onTextDelta?: (e: { fullContent: string; delta: string }) => void
  onReasoningDelta?: (e: { fullReasoning: string; delta: string }) => void
  onToolCall?: (e: { toolName: string; toolCallId?: string; input: unknown }) => void
  onToolResult?: (e: { toolName: string; toolCallId?: string; output: unknown }) => void
  onFinish?: (e: { finishReason?: string; usage?: unknown }) => void
}

export interface RunAgentResult {
  text: string
  reasoning: string
  finishReason?: string
  usage?: unknown
  toolCount: number
  enabledToolNames: string[]
}

// ========== Provider factory ==========

export function createProviderClient(
  providerId: string,
  settings: { apiKey: string; baseURL?: string }
) {
  // Route all LLM requests through Tauri's HTTP plugin (Rust side) so they are
  // not subject to the webview's CORS policy. Browsers block direct calls to
  // api.anthropic.com / api.openai.com from a non-origin; the Rust client does not.
  //
  // Also force `Accept-Encoding: identity`: otherwise the provider may gzip/br the
  // SSE response, and reqwest buffers to decode it, so the whole reply lands in one
  // burst and the UI shows it all at once instead of streaming token-by-token.
  // Headers must be passed as a plain object (not a Headers instance) because the
  // webview's Headers API silently drops the forbidden `Accept-Encoding` name; a
  // plain object is forwarded to the Rust client untouched.
  const fetch = ((input: RequestInfo | URL, init: RequestInit = {}) => {
    const headers: Record<string, string> = {}
    new Headers(init.headers as HeadersInit | undefined).forEach((value, key) => {
      headers[key] = value
    })
    headers['accept-encoding'] = 'identity'
    return (tauriFetch as unknown as typeof globalThis.fetch)(input, { ...init, headers })
  }) as unknown as typeof globalThis.fetch

  if (settings.baseURL) {
    if (settings.baseURL.includes('anthropic') || providerId === 'anthropic') {
      return createAnthropic({ apiKey: settings.apiKey, baseURL: settings.baseURL, fetch })
    }
    return createOpenAICompatible({
      name: providerId,
      apiKey: settings.apiKey,
      baseURL: settings.baseURL,
      fetch,
    })
  }

  switch (providerId) {
    case 'openai':
      return createOpenAI({ apiKey: settings.apiKey, fetch })
    case 'anthropic':
      return createAnthropic({ apiKey: settings.apiKey, fetch })
    default:
      throw new Error(`Provider ${providerId} requires a baseURL endpoint to be configured`)
  }
}

export function createAgentSystemPrompt(enabledToolNames: string[]): string {
  const toolHint =
    enabledToolNames.length > 0
      ? `Available tools: ${enabledToolNames.join(', ')}. Use tools when they materially improve accuracy or can verify the user's request.`
      : 'No tools are currently enabled.'

  return [
    'You are Jedi, a desktop AI agent for developers.',
    'Work autonomously: understand the task, make a brief plan, use tools when useful, verify results, and then answer clearly.',
    'Expose concise progress summaries, tool choices, and verification outcomes. Do not reveal hidden chain-of-thought; provide short reasoning summaries instead.',
    'When a tool result is relevant, incorporate it into the final answer. If a tool fails, recover when possible and explain the useful part.',
    'For anything time-sensitive, real-time, or that you are not certain about (weather, prices, news, current events, library docs), prefer web_search / web_fetch over guessing or claiming you cannot access it.',
    toolHint,
  ].join('\n')
}

// ========== Shared formatting helpers ==========

export function summarizeValue(value: unknown, maxLength = 1600): string {
  const text = typeof value === 'string' ? value : JSON.stringify(value, null, 2)
  if (!text) return ''
  return text.length > maxLength ? `${text.slice(0, maxLength)}\n...` : text
}

export function summarizeUsage(usage: unknown): string {
  if (!usage || typeof usage !== 'object') return 'No usage data'
  const u = usage as Record<string, unknown>
  const pick = (...keys: string[]): number | null => {
    for (const k of keys) {
      if (typeof u[k] === 'number') return u[k] as number
    }
    return null
  }
  const inputTokens = pick('inputTokens', 'promptTokens', 'input_tokens')
  const outputTokens = pick('outputTokens', 'completionTokens', 'output_tokens')
  const reasoningTokens = pick('reasoningTokens')
  const totalTokens = pick('totalTokens', 'total_tokens')

  return [
    inputTokens !== null ? `in ${inputTokens}` : null,
    outputTokens !== null ? `out ${outputTokens}` : null,
    reasoningTokens !== null ? `reason ${reasoningTokens}` : null,
    totalTokens !== null ? `total ${totalTokens}` : null,
  ]
    .filter(Boolean)
    .join(' / ')
}

export function extractTextFromContentParts(content: unknown): string {
  if (!Array.isArray(content)) return ''
  return content
    .map(part => {
      if (!part || typeof part !== 'object') return ''
      const record = part as Record<string, unknown>
      if (typeof record.text === 'string') return record.text
      if (typeof record.content === 'string') return record.content
      if (typeof record.value === 'string') return record.value
      return ''
    })
    .filter(Boolean)
    .join('')
    .trim()
}

export function extractStepText(step: unknown): string {
  if (!step || typeof step !== 'object') return ''
  const record = step as Record<string, unknown>
  if (typeof record.text === 'string' && record.text.trim()) return record.text.trim()
  return extractTextFromContentParts(record.content)
}

/**
 * Sanitize a skill's parameter schema into valid JSON Schema before sending it
 * as a tool definition. The skills use a non-standard per-property
 * `required: boolean` convention; JSON Schema only allows `required` as an
 * object-level array, so strict providers (e.g. DeepSeek) reject it with
 * "Invalid schema for function: true is not of type array". We drop boolean
 * `required` keys recursively and keep array `required` lists.
 */
export function sanitizeJsonSchema(schema: unknown): any {
  if (Array.isArray(schema)) return schema.map(sanitizeJsonSchema)
  if (schema && typeof schema === 'object') {
    const out: Record<string, unknown> = {}
    for (const [key, value] of Object.entries(schema)) {
      if (key === 'required' && typeof value === 'boolean') continue
      out[key] = sanitizeJsonSchema(value)
    }
    return out
  }
  return schema
}

/** Turn an AI SDK / provider error into a readable Error (status + body when present). */
export function asError(err: unknown): Error {
  if (err instanceof Error && !(err as any).statusCode && !(err as any).responseBody) {
    return err
  }
  const e = err as Record<string, unknown> | null
  const parts: string[] = []
  if (e && typeof e.message === 'string') parts.push(e.message)
  if (e && (typeof e.statusCode === 'number' || typeof e.statusCode === 'string')) {
    parts.push(`HTTP ${e.statusCode}`)
  }
  const body = e?.responseBody ?? e?.data ?? e?.cause
  if (body) {
    const bodyText = typeof body === 'string' ? body : JSON.stringify(body)
    if (bodyText && !parts.join(' ').includes(bodyText.slice(0, 40))) {
      parts.push(bodyText.slice(0, 600))
    }
  }
  const message = parts.filter(Boolean).join(' — ') || String(err)
  return new Error(message)
}

// ========== Session title distillation ==========

export interface DistillTitleSpec {
  provider: string
  model: string
  apiKey: string
  endpoint?: string
  userMessage: string
  assistantMessage: string
}

/** Strip quotes / trailing punctuation and cap length so the result fits a sidebar row. */
function sanitizeTitle(raw: string): string {
  let title = (raw || '').trim().split('\n')[0].trim()
  title = title
    .replace(/^["'“”「『]+/, '')
    .replace(/["'“”」』]+$/, '')
    .replace(/[。.!！?？、,，;；:：\s]+$/, '')
    .trim()
  return title.length > 24 ? title.slice(0, 24) : title
}

/**
 * Ask the model for a short title that captures what the conversation is about.
 * Best-effort: callers should fall back to a heuristic title on failure. Routes
 * through the same Tauri-backed provider client as the main run.
 */
export async function distillTitle(spec: DistillTitleSpec): Promise<string> {
  const providerClient = createProviderClient(spec.provider, {
    apiKey: spec.apiKey,
    baseURL: spec.endpoint,
  })
  const prompt = [
    '为下面这段对话提炼一个简短的会话标题。',
    '要求：概括对话主题；不超过 16 个字符；只输出标题本身，不要引号、标点、序号或任何解释。',
    '',
    `用户：${spec.userMessage.slice(0, 800)}`,
    `助手：${spec.assistantMessage.slice(0, 800)}`,
    '',
    '标题：',
  ].join('\n')

  const { text } = await generateText({
    model: providerClient.languageModel(spec.model),
    prompt,
  })
  return sanitizeTitle(text)
}

// ========== Executor ==========

export async function runAgent(
  spec: RunAgentSpec,
  hooks: RunAgentHooks = {}
): Promise<RunAgentResult> {
  const stepLimit = spec.stepLimit ?? DEFAULT_STEP_LIMIT
  const providerClient = createProviderClient(spec.provider, {
    apiKey: spec.apiKey,
    baseURL: spec.endpoint,
  })

  const enabledSkills = skillRegistry.listAutoCallable()
  const enabledToolNames = enabledSkills.map(skill => skill.name)
  let toolCount = 0

  const tools = Object.fromEntries(
    enabledSkills.map(skill => [
      skill.id,
      tool({
        description: `${skill.name}: ${skill.description}`,
        inputSchema: jsonSchema(sanitizeJsonSchema(skill.parameters)),
        execute: async args => {
          const startedAt = Date.now()
          hooks.onToolStart?.({
            skillId: skill.id,
            skillName: skill.name,
            skillDescription: skill.description,
            args,
            startedAt,
          })
          try {
            toolCount += 1
            const output = await skill.execute(args, { sessionId: spec.sessionId })
            hooks.onToolEnd?.({ skillId: skill.id, skillName: skill.name, args, startedAt, output })
            return output
          } catch (e) {
            const message = e instanceof Error ? e.message : String(e)
            hooks.onToolEnd?.({
              skillId: skill.id,
              skillName: skill.name,
              args,
              startedAt,
              error: message,
            })
            throw e
          }
        },
      }),
    ])
  )

  let fullContent = ''
  let reasoningSummary = ''
  let lastStepText = ''
  let finishText = ''
  let finishReason: string | undefined
  let usage: unknown
  // The AI SDK surfaces network / provider errors as stream "error" parts (and
  // via onError) rather than throwing, so we capture them and rethrow below —
  // otherwise a failed request silently looks like an empty answer.
  let streamError: unknown = null

  const result = streamText({
    model: providerClient.languageModel(spec.model),
    system: createAgentSystemPrompt(enabledToolNames),
    messages: spec.messages,
    tools,
    abortSignal: spec.signal,
    stopWhen: [stepCountIs(stepLimit), isLoopFinished()],
    onError: ({ error }) => {
      streamError = error
    },
    onStepFinish: step => {
      const stepKind =
        step.toolResults.length > 0 ? 'tool-result' : step.stepNumber === 0 ? 'initial' : 'continue'
      const stepText = extractStepText(step)
      finishReason = step.finishReason
      usage = step.usage
      if (stepText) {
        lastStepText = stepText
      }
      hooks.onStepFinish?.({
        stepNumber: step.stepNumber,
        finishReason: step.finishReason,
        usage: step.usage,
        text: step.text,
        toolCalls: step.toolCalls,
        toolResults: step.toolResults,
        stepKind,
        stepText,
      })
    },
    onFinish: event => {
      finishText = extractStepText(event)
      finishReason = event.finishReason
      usage = event.totalUsage
    },
  })

  let chunkCount = 0
  for await (const chunk of result.fullStream) {
    chunkCount += 1
    if (chunk.type === 'text-delta') {
      const delta = (chunk as any).delta ?? (chunk as any).text ?? ''
      fullContent += delta
      hooks.onTextDelta?.({ fullContent, delta })
    } else if (chunk.type === 'reasoning-delta') {
      const delta = (chunk as any).delta ?? (chunk as any).text ?? ''
      reasoningSummary += delta
      hooks.onReasoningDelta?.({ fullReasoning: reasoningSummary, delta })
    } else if (chunk.type === 'tool-call') {
      const toolName = (chunk as any).toolName || (chunk as any).toolCall?.toolName || 'unknown'
      const toolCallId = (chunk as any).toolCallId || (chunk as any).toolCall?.toolCallId
      const input =
        (chunk as any).input ??
        (chunk as any).args ??
        (chunk as any).toolCall?.args ??
        (chunk as any).toolCall?.input
      hooks.onToolCall?.({ toolName, toolCallId, input })
    } else if (chunk.type === 'tool-result') {
      const toolName = (chunk as any).toolName || (chunk as any).toolResult?.toolName || 'unknown'
      const toolCallId = (chunk as any).toolCallId || (chunk as any).toolResult?.toolCallId
      const output =
        (chunk as any).output ??
        (chunk as any).result ??
        (chunk as any).toolResult?.output ??
        (chunk as any).toolResult?.result
      hooks.onToolResult?.({ toolName, toolCallId, output })
    } else if (chunk.type === 'error') {
      streamError = (chunk as any).error ?? streamError
    } else if (chunk.type === 'finish') {
      finishReason = (chunk as any).finishReason ?? finishReason
      hooks.onFinish?.({ finishReason, usage })
    }
  }

  // A provider/network error means there is no usable answer — surface the real
  // reason instead of letting it look like an empty completion.
  if (streamError) {
    throw asError(streamError)
  }

  // The stream produced no events at all and never finished: this is a transport
  // failure (e.g. the Tauri HTTP fetch), not a real empty answer. Fail loudly.
  if (chunkCount === 0 && finishReason === undefined && !fullContent.trim() && !lastStepText) {
    throw new Error(
      'LLM 流未返回任何事件（HTTP/fetch 传输层失败）。已验证目标接口本身可用，请检查 tauri-plugin-http / 网络代理。'
    )
  }

  // Resolve the final answer. Some providers only surface the answer on step
  // finish or in the aggregate result instead of via text-delta chunks.
  if (!fullContent.trim() && lastStepText) {
    fullContent = lastStepText
  } else if (!fullContent.trim() && finishText.trim()) {
    fullContent = finishText.trim()
  } else if (!fullContent.trim()) {
    const resultText = await Promise.resolve(result.text).catch(() => '')
    const resultContent = await Promise.resolve(result.content).catch(() => [])
    fullContent = resultText.trim() || extractTextFromContentParts(resultContent)
  }

  return {
    text: fullContent,
    reasoning: reasoningSummary,
    finishReason,
    usage,
    toolCount,
    enabledToolNames,
  }
}
