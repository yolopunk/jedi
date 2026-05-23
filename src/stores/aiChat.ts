import { createAnthropic } from '@ai-sdk/anthropic'
import { createOpenAI } from '@ai-sdk/openai'
import { createOpenAICompatible } from '@ai-sdk/openai-compatible'
import { invoke } from '@tauri-apps/api/core'
import { isLoopFinished, jsonSchema, stepCountIs, streamText, tool } from 'ai'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { skillRegistry } from '@/skills/registry'
import { useAgentStore } from './agent'
import { useModelsDevStore } from './modelsDev'
import { useProviderConfigStore } from './providerConfig'

// MCP Server 接口
export interface McpServer {
  id: string
  name: string
  description?: string
  enabled: boolean
  icon?: string
}

// 消息格式
export interface ChatMessage {
  role: 'system' | 'user' | 'assistant'
  content: string
  timestamp?: number
  isStreaming?: boolean
  error?: string
  metadata?: MessageMetadata
}

// 兼容旧组件的类型导出
export interface Message extends ChatMessage {}

// 会话格式
export interface Session {
  id: string
  title: string
  messages: ChatMessage[]
  provider: string
  model: string
  created_at: string
  updated_at: string
}

export interface AgentTraceDetail {
  id: string
  type: 'session' | 'step' | 'think' | 'reasoning' | 'tool' | 'answer' | 'finish' | 'error'
  status: 'running' | 'done' | 'error'
  title: string
  content?: string
  input?: unknown
  output?: unknown
  timestamp: number
  durationMs?: number
  stepIndex?: number
  toolName?: string
  toolCallId?: string
}

export interface AgentRunMetadata {
  provider: string
  model: string
  startedAt: number
  completedAt?: number
  totalDurationMs?: number
  stepLimit: number
  enabledTools: string[]
  toolCount: number
  finishReason?: string
  usage?: unknown
}

export interface MessageMetadata {
  reasoning?: string
  trace?: AgentTraceDetail[]
  toolCalls?: AgentTraceDetail[]
  finishReason?: string
  usage?: unknown
  run?: AgentRunMetadata
}

// 预定义的MCP服务器
export const DEFAULT_MCP_SERVERS: McpServer[] = [
  {
    id: 'hosts',
    name: 'Hosts Manager',
    description: '管理系统Hosts文件',
    enabled: false,
    icon: 'mdi-dns',
  },
  {
    id: 'filesystem',
    name: 'Filesystem',
    description: '文件系统操作',
    enabled: false,
    icon: 'mdi-folder',
  },
  {
    id: 'browser',
    name: 'Browser',
    description: '网页浏览和搜索',
    enabled: false,
    icon: 'mdi-web',
  },
]

// ========== AI SDK Provider 工厂 ==========

function createProviderClient(providerId: string, settings: { apiKey: string; baseURL?: string }) {
  // If there's a custom baseURL, determine which SDK to use based on the API format
  if (settings.baseURL) {
    // Check if the endpoint uses Anthropic API format
    if (settings.baseURL.includes('anthropic') || providerId === 'anthropic') {
      return createAnthropic({
        apiKey: settings.apiKey,
        baseURL: settings.baseURL,
      })
    }
    // Otherwise use OpenAI-compatible
    return createOpenAICompatible({
      name: providerId,
      apiKey: settings.apiKey,
      baseURL: settings.baseURL,
    })
  }

  // For providers without custom baseURL, use official SDKs
  switch (providerId) {
    case 'openai':
      return createOpenAI({
        apiKey: settings.apiKey,
      })
    case 'anthropic':
      return createAnthropic({
        apiKey: settings.apiKey,
      })
    default:
      // For unknown providers without baseURL, throw an error - they need to specify an endpoint
      throw new Error(`Provider ${providerId} requires a baseURL endpoint to be configured`)
  }
}

function createAgentSystemPrompt(enabledToolNames: string[]): string {
  const toolHint =
    enabledToolNames.length > 0
      ? `Available tools: ${enabledToolNames.join(', ')}. Use tools when they materially improve accuracy or can verify the user's request.`
      : 'No tools are currently enabled.'

  return [
    'You are Jedi, a desktop AI agent for developers.',
    'Work autonomously: understand the task, make a brief plan, use tools when useful, verify results, and then answer clearly.',
    'Expose concise progress summaries, tool choices, and verification outcomes. Do not reveal hidden chain-of-thought; provide short reasoning summaries instead.',
    'When a tool result is relevant, incorporate it into the final answer. If a tool fails, recover when possible and explain the useful part.',
    toolHint,
  ].join('\n')
}

function summarizeValue(value: unknown, maxLength = 1600): string {
  const text = typeof value === 'string' ? value : JSON.stringify(value, null, 2)
  if (!text) return ''
  return text.length > maxLength ? `${text.slice(0, maxLength)}\n...` : text
}

let traceCounter = 0

function nextTraceId(prefix: string): string {
  traceCounter += 1
  return `${prefix}-${Date.now()}-${traceCounter}`
}

function summarizeUsage(usage: unknown): string {
  if (!usage || typeof usage !== 'object') return 'No usage data'
  const usageRecord = usage as Record<string, unknown>
  const inputTokens =
    typeof usageRecord.inputTokens === 'number'
      ? usageRecord.inputTokens
      : typeof usageRecord.promptTokens === 'number'
        ? usageRecord.promptTokens
        : typeof usageRecord.input_tokens === 'number'
          ? usageRecord.input_tokens
          : null
  const outputTokens =
    typeof usageRecord.outputTokens === 'number'
      ? usageRecord.outputTokens
      : typeof usageRecord.completionTokens === 'number'
        ? usageRecord.completionTokens
        : typeof usageRecord.output_tokens === 'number'
          ? usageRecord.output_tokens
          : null
  const reasoningTokens =
    typeof usageRecord.reasoningTokens === 'number' ? usageRecord.reasoningTokens : null
  const totalTokens =
    typeof usageRecord.totalTokens === 'number'
      ? usageRecord.totalTokens
      : typeof usageRecord.total_tokens === 'number'
        ? usageRecord.total_tokens
        : null

  return [
    inputTokens !== null ? `in ${inputTokens}` : null,
    outputTokens !== null ? `out ${outputTokens}` : null,
    reasoningTokens !== null ? `reason ${reasoningTokens}` : null,
    totalTokens !== null ? `total ${totalTokens}` : null,
  ]
    .filter(Boolean)
    .join(' / ')
}

function extractTextFromContentParts(content: unknown): string {
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

function extractStepText(step: unknown): string {
  if (!step || typeof step !== 'object') return ''
  const record = step as Record<string, unknown>
  if (typeof record.text === 'string' && record.text.trim()) return record.text.trim()
  return extractTextFromContentParts(record.content)
}

// ========== Store ==========

export const useAiChatStore = defineStore('aiChat', () => {
  // State
  const sessions = ref<Session[]>([])
  const currentSessionId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const streamingContent = ref<string>('')

  // MCP State
  const enabledMcpServers = ref<string[]>([])
  const mcpServers = ref<McpServer[]>([...DEFAULT_MCP_SERVERS])

  // Stores
  const providerConfigStore = useProviderConfigStore()
  const modelsDevStore = useModelsDevStore()
  const agentStore = useAgentStore()

  // Computed
  const currentSession = computed(
    () => sessions.value.find(s => s.id === currentSessionId.value) || null
  )

  // Actions
  async function loadSessions() {
    try {
      sessions.value = await invoke('list_sessions')
      error.value = null
    } catch (e) {
      error.value = `加载会话失败: ${e}`
      console.error('Failed to load sessions:', e)
    }
  }

  async function createSession(
    title: string = '新对话',
    provider: string = 'openai',
    model: string = 'gpt-4o-mini'
  ) {
    try {
      const session = await invoke('create_session', {
        request: { title, provider, model },
      })
      sessions.value.unshift(session as Session)
      currentSessionId.value = (session as Session).id
      error.value = null
      return session as Session
    } catch (e) {
      error.value = `创建会话失败: ${e}`
      console.error('Failed to create session:', e)
      throw e
    }
  }

  async function deleteSession(sessionId: string) {
    try {
      await invoke('delete_session', { session_id: sessionId })
      sessions.value = sessions.value.filter(s => s.id !== sessionId)
      if (currentSessionId.value === sessionId) {
        currentSessionId.value = sessions.value[0]?.id || null
      }
      error.value = null
    } catch (e) {
      error.value = `删除会话失败: ${e}`
      console.error('Failed to delete session:', e)
      throw e
    }
  }

  async function sendMessage(content: string): Promise<void> {
    const provider = modelsDevStore.selectedProviderId || 'openai'
    const model = modelsDevStore.selectedModelId || 'gpt-4o-mini'
    const runStartedAt = Date.now()
    const stepLimit = 8

    // Create session if needed
    if (!currentSession.value) {
      await createSession('新对话', provider, model)
    }

    const session = currentSession.value!
    const userMessage: ChatMessage = { role: 'user', content, timestamp: Date.now() }
    const assistantMessage: ChatMessage = {
      role: 'assistant',
      content: '',
      timestamp: Date.now(),
      isStreaming: true,
      metadata: {
        reasoning: '',
        trace: [],
        toolCalls: [],
        run: {
          provider,
          model,
          startedAt: runStartedAt,
          stepLimit,
          enabledTools: [],
          toolCount: 0,
        },
      },
    }

    session.messages.push(userMessage)
    session.messages.push(assistantMessage)

    isLoading.value = true
    streamingContent.value = ''
    error.value = null
    agentStore.reset()
    agentStore.setStatus('planning')
    let planTrace: AgentTraceDetail | null = null
    let reasoningTrace: AgentTraceDetail | null = null
    let answerTrace: AgentTraceDetail | null = null
    let stepCounter = 0
    let currentStreamStepIndex = 1
    let lastReasoningTraceUpdateAt = 0
    const toolDecisionTraces = new Map<string, AgentTraceDetail>()
    const toolExecutionTraces = new Map<string, AgentTraceDetail[]>()

    const pushTrace = (entry: AgentTraceDetail) => {
      if (
        entry.stepIndex === undefined &&
        entry.type !== 'session' &&
        entry.type !== 'finish' &&
        entry.type !== 'error'
      ) {
        entry.stepIndex = currentStreamStepIndex
      }
      assistantMessage.metadata?.trace?.push(entry)
      if (entry.type === 'tool') {
        assistantMessage.metadata?.toolCalls?.push(entry)
      }
      return entry
    }

    const updateTrace = (entry: AgentTraceDetail, patch: Partial<AgentTraceDetail>) => {
      Object.assign(entry, patch)
    }

    const completeTrace = (entry: AgentTraceDetail, output?: unknown) => {
      entry.status = 'done'
      entry.output = output
      entry.durationMs = Date.now() - entry.timestamp
    }

    const failTrace = (entry: AgentTraceDetail, errorMessage: string) => {
      entry.status = 'error'
      entry.output = errorMessage
      entry.durationMs = Date.now() - entry.timestamp
    }

    try {
      await invoke('append_message', {
        request: {
          session_id: session.id,
          role: 'user',
          content,
        },
      })

      // Get API key for provider
      const apiKeyInfo = await providerConfigStore.getApiKey(provider)
      if (!apiKeyInfo) {
        throw new Error(`API key not configured for provider: ${provider}`)
      }

      // Create provider client
      const providerClient = createProviderClient(provider, {
        apiKey: apiKeyInfo.key,
        baseURL: apiKeyInfo.endpoint,
      })

      const enabledSkills = skillRegistry.listAutoCallable()
      const enabledToolNames = enabledSkills.map(skill => skill.name)
      if (assistantMessage.metadata?.run) {
        assistantMessage.metadata.run.enabledTools = enabledToolNames
      }

      pushTrace({
        id: nextTraceId('session'),
        type: 'session',
        status: 'done',
        title: 'Run initialized',
        content: `Provider ${provider} / Model ${model} / Step budget ${stepLimit}`,
        output: {
          provider,
          model,
          enabledTools: enabledToolNames,
          stepLimit,
        },
        timestamp: Date.now(),
        durationMs: 0,
      })

      const tools = Object.fromEntries(
        enabledSkills.map(skill => [
          skill.id,
          tool({
            description: `${skill.name}: ${skill.description}`,
            inputSchema: jsonSchema(skill.parameters as any),
            execute: async args => {
              const startedAt = Date.now()
              const relatedDecisionTrace = [...toolDecisionTraces.values()]
                .reverse()
                .find(trace => trace.toolName === skill.name && trace.status === 'running')
              const traceEntry = pushTrace({
                id: `${skill.id}-${startedAt}`,
                type: 'tool',
                status: 'running',
                title: `Executing ${skill.name}`,
                content: `Executing ${skill.name}. ${skill.description}`,
                input: args,
                timestamp: startedAt,
                toolName: skill.name,
                toolCallId: relatedDecisionTrace?.toolCallId,
              })
              if (relatedDecisionTrace?.toolCallId) {
                const traces = toolExecutionTraces.get(relatedDecisionTrace.toolCallId) || []
                traces.push(traceEntry)
                toolExecutionTraces.set(relatedDecisionTrace.toolCallId, traces)
              }
              const step = agentStore.startStep(
                'tool',
                `Calling ${skill.name}`,
                summarizeValue(args, 600)
              )

              try {
                agentStore.setStatus('executing')
                if (assistantMessage.metadata?.run) {
                  assistantMessage.metadata.run.toolCount += 1
                }
                const output = await skill.execute(args, { sessionId: session.id })
                completeTrace(traceEntry, output)
                agentStore.completeStep(step, summarizeValue(output, 1000))
                return output
              } catch (e) {
                const message = e instanceof Error ? e.message : String(e)
                failTrace(traceEntry, message)
                agentStore.failStep(step, message)
                throw e
              }
            },
          }),
        ])
      )

      planTrace = pushTrace({
        id: nextTraceId('think'),
        type: 'think',
        status: 'running',
        title: 'Planning',
        content: '理解问题、选择是否需要工具，并准备验证路径。',
        timestamp: Date.now(),
      })
      reasoningTrace = pushTrace({
        id: nextTraceId('reasoning'),
        type: 'reasoning',
        status: 'running',
        title: 'Reasoning summary stream',
        content: 'Waiting for model reasoning...',
        timestamp: Date.now(),
      })
      answerTrace = pushTrace({
        id: nextTraceId('answer'),
        type: 'answer',
        status: 'running',
        title: 'Drafting response',
        content: 'Streaming assistant response...',
        timestamp: Date.now(),
      })
      const planStep = agentStore.startStep('think', 'Planning response', content)

      // Convert messages to AI SDK format
      type AIMessage = { role: 'user' | 'assistant' | 'system'; content: string }
      const aiMessages: AIMessage[] = session.messages
        .filter(m => !m.isStreaming)
        .map(m => ({
          role: m.role as AIMessage['role'],
          content: m.content,
        }))

      // Collect model output across providers. Some providers emit the final answer
      // only on step finish instead of text-delta chunks.
      let fullContent = ''
      let reasoningSummary = ''
      let planCompleted = false
      let lastStepText = ''
      let finishText = ''

      // Use streamText for streaming
      const result = streamText({
        model: providerClient.languageModel(model),
        system: createAgentSystemPrompt(enabledToolNames),
        messages: aiMessages,
        tools,
        stopWhen: [stepCountIs(stepLimit), isLoopFinished()],
        onStepFinish: step => {
          stepCounter += 1
          const stepKind =
            step.toolResults.length > 0
              ? 'tool-result'
              : step.stepNumber === 0
                ? 'initial'
                : 'continue'
          const stepText = extractStepText(step)
          assistantMessage.metadata!.finishReason = step.finishReason
          assistantMessage.metadata!.usage = step.usage
          if (assistantMessage.metadata?.run) {
            assistantMessage.metadata.run.finishReason = step.finishReason
            assistantMessage.metadata.run.usage = step.usage
          }
          if (stepText) {
            lastStepText = stepText
            assistantMessage.content = stepText
            if (!fullContent.trim()) {
              streamingContent.value = stepText
            }
          }
          pushTrace({
            id: nextTraceId('step'),
            type: 'step',
            status: 'done',
            title: `LLM step ${stepCounter}`,
            content: `Step ${stepCounter} finished as ${stepKind} with ${step.finishReason}.`,
            output: {
              type: stepKind,
              finishReason: step.finishReason,
              usage: step.usage,
              usageSummary: summarizeUsage(step.usage),
              toolCalls: step.toolCalls,
              toolResults: step.toolResults,
              text: step.text,
            },
            timestamp: Date.now(),
            stepIndex: currentStreamStepIndex,
          })
          currentStreamStepIndex = step.stepNumber + 2
        },
        onFinish: event => {
          finishText = extractStepText(event)
          assistantMessage.metadata!.finishReason = event.finishReason
          assistantMessage.metadata!.usage = event.totalUsage
          if (assistantMessage.metadata?.run) {
            assistantMessage.metadata.run.finishReason = event.finishReason
            assistantMessage.metadata.run.usage = event.totalUsage
          }
        },
      })

      // Collect streaming response
      for await (const chunk of result.fullStream) {
        if (chunk.type === 'text-delta') {
          fullContent += (chunk as any).delta ?? (chunk as any).text ?? ''
          streamingContent.value = fullContent
          assistantMessage.content = fullContent
          updateTrace(answerTrace, {
            content: fullContent
              ? `Streaming ${fullContent.length} chars of assistant output.`
              : 'Streaming assistant response...',
          })
        } else if (chunk.type === 'reasoning-delta') {
          reasoningSummary += (chunk as any).delta ?? (chunk as any).text ?? ''
          assistantMessage.metadata!.reasoning = reasoningSummary
          const now = Date.now()
          if (
            now - lastReasoningTraceUpdateAt > 250 ||
            reasoningSummary.length < 240 ||
            reasoningSummary.length % 320 < 40
          ) {
            updateTrace(reasoningTrace, {
              content: summarizeValue(reasoningSummary, 1400),
            })
            lastReasoningTraceUpdateAt = now
          }
          if (!planCompleted && reasoningSummary.trim()) {
            agentStore.completeStep(planStep, summarizeValue(reasoningSummary, 800))
            completeTrace(planTrace, summarizeValue(reasoningSummary, 800))
            planCompleted = true
          }
        } else if (chunk.type === 'tool-call') {
          const toolName = (chunk as any).toolName || (chunk as any).toolCall?.toolName || 'unknown'
          const toolCallId = (chunk as any).toolCallId || (chunk as any).toolCall?.toolCallId
          const input =
            (chunk as any).input ??
            (chunk as any).args ??
            (chunk as any).toolCall?.args ??
            (chunk as any).toolCall?.input
          const decisionTrace = pushTrace({
            id: nextTraceId('tool-call'),
            type: 'tool',
            status: 'running',
            title: `Tool requested: ${toolName}`,
            content: `Model selected ${toolName} for verification.`,
            input,
            timestamp: Date.now(),
            toolName,
            toolCallId,
          })
          if (toolCallId) {
            toolDecisionTraces.set(toolCallId, decisionTrace)
          }
          if (!planCompleted) {
            agentStore.completeStep(planStep, 'Selected a tool for verification.')
            completeTrace(planTrace, 'Selected a tool for verification.')
            planCompleted = true
          }
        } else if (chunk.type === 'tool-result') {
          const toolName =
            (chunk as any).toolName || (chunk as any).toolResult?.toolName || 'unknown'
          const toolCallId = (chunk as any).toolCallId || (chunk as any).toolResult?.toolCallId
          const output =
            (chunk as any).output ??
            (chunk as any).result ??
            (chunk as any).toolResult?.output ??
            (chunk as any).toolResult?.result
          const decisionTrace = toolCallId ? toolDecisionTraces.get(toolCallId) : undefined
          if (decisionTrace?.status === 'running') {
            completeTrace(decisionTrace, output)
          }
          const executionTrace = toolCallId
            ? (toolExecutionTraces.get(toolCallId) || []).find(trace => trace.status === 'running')
            : undefined
          if (executionTrace?.status === 'running') {
            completeTrace(executionTrace, output)
          }
          pushTrace({
            id: nextTraceId('tool-result'),
            type: 'tool',
            status: 'done',
            title: `Tool result received: ${toolName}`,
            content: `Model received ${toolName} output and can continue reasoning.`,
            output,
            timestamp: Date.now(),
            toolName,
            toolCallId,
          })
        } else if (chunk.type === 'finish') {
          assistantMessage.metadata!.finishReason = chunk.finishReason
          if (assistantMessage.metadata?.run) {
            assistantMessage.metadata.run.finishReason = chunk.finishReason
          }
        }
      }

      lastStepText = assistantMessage.content.trim()

      if (!planCompleted) {
        agentStore.completeStep(planStep, 'Answered directly.')
        completeTrace(planTrace, 'Answered directly.')
      }
      completeTrace(
        reasoningTrace,
        summarizeValue(reasoningSummary || 'No reasoning summary emitted.', 1400)
      )
      if (!fullContent.trim() && lastStepText) {
        fullContent = lastStepText
      } else if (!fullContent.trim() && finishText.trim()) {
        fullContent = finishText.trim()
      } else if (!fullContent.trim()) {
        const resultText = await Promise.resolve(result.text).catch(() => '')
        const resultContent = await Promise.resolve(result.content).catch(() => [])
        fullContent = resultText.trim() || extractTextFromContentParts(resultContent)
      }
      if (!fullContent.trim()) {
        fullContent =
          '模型本轮没有返回可展示的文本内容。请在 Trace 中查看 finish reason、usage 和原始事件。'
      }
      completeTrace(
        answerTrace,
        summarizeValue(fullContent || 'No final answer text emitted.', 1800)
      )

      assistantMessage.content = fullContent
      assistantMessage.isStreaming = false
      const runCompletedAt = Date.now()
      if (assistantMessage.metadata?.run) {
        assistantMessage.metadata.run.completedAt = runCompletedAt
        assistantMessage.metadata.run.totalDurationMs = runCompletedAt - runStartedAt
      }
      pushTrace({
        id: nextTraceId('finish'),
        type: 'finish',
        status: 'done',
        title: 'Completed',
        content: '已生成回答并完成本轮对话。',
        output: {
          finishReason: assistantMessage.metadata?.finishReason || 'unknown',
          usage: assistantMessage.metadata?.usage,
          totalDurationMs: assistantMessage.metadata?.run?.totalDurationMs,
          toolCount: assistantMessage.metadata?.run?.toolCount || 0,
        },
        timestamp: Date.now(),
        durationMs: runCompletedAt - runStartedAt,
      })
      agentStore.setStatus('done')

      // Persist to backend
      await invoke('append_message', {
        request: {
          session_id: session.id,
          role: 'assistant',
          content: fullContent,
        },
      })
    } catch (e: any) {
      error.value = `发送消息失败: ${e}`
      console.error('Failed to send message:', e)
      assistantMessage.isStreaming = false
      assistantMessage.error = e instanceof Error ? e.message : String(e)
      assistantMessage.content = `发送消息失败：${assistantMessage.error}`
      if (planTrace?.status === 'running') {
        failTrace(planTrace, assistantMessage.error)
      }
      if (reasoningTrace?.status === 'running') {
        failTrace(reasoningTrace, assistantMessage.error)
      }
      if (answerTrace?.status === 'running') {
        failTrace(answerTrace, assistantMessage.error)
      }
      if (assistantMessage.metadata?.run) {
        assistantMessage.metadata.run.completedAt = Date.now()
        assistantMessage.metadata.run.totalDurationMs =
          assistantMessage.metadata.run.completedAt - runStartedAt
      }
      pushTrace({
        id: nextTraceId('error'),
        type: 'error',
        status: 'error',
        title: 'Error',
        content: assistantMessage.error,
        timestamp: Date.now(),
        durationMs: Date.now() - runStartedAt,
      })
      agentStore.setStatus('error')
      throw e
    } finally {
      isLoading.value = false
      streamingContent.value = ''
    }
  }

  // Toggle MCP server
  function toggleMcpServer(serverId: string) {
    const server = mcpServers.value.find(s => s.id === serverId)
    if (server) {
      server.enabled = !server.enabled
      if (server.enabled) {
        if (!enabledMcpServers.value.includes(serverId)) {
          enabledMcpServers.value.push(serverId)
        }
      } else {
        enabledMcpServers.value = enabledMcpServers.value.filter(id => id !== serverId)
      }
    }
  }

  // Load settings from localStorage
  function loadSettings() {
    try {
      const saved = localStorage.getItem('chat-settings')
      if (saved) {
        const settings = JSON.parse(saved)
        if (settings.enabledMcpServers) enabledMcpServers.value = settings.enabledMcpServers

        // Sync MCP server enabled states
        mcpServers.value.forEach(server => {
          server.enabled = enabledMcpServers.value.includes(server.id)
        })
      }
    } catch (e) {
      console.error('Failed to load AI chat settings:', e)
    }
  }

  // Add a message to the current session
  function addMessage(message: ChatMessage) {
    if (currentSession.value) {
      currentSession.value.messages.push(message)
    }
  }

  // Save settings to localStorage
  function saveSettings() {
    try {
      const settings = {
        enabledMcpServers: enabledMcpServers.value,
      }
      localStorage.setItem('chat-settings', JSON.stringify(settings))
    } catch (e) {
      console.error('Failed to save AI chat settings:', e)
    }
  }

  return {
    // State
    sessions,
    currentSessionId,
    isLoading,
    error,
    streamingContent,
    enabledMcpServers,
    mcpServers,

    // Computed
    currentSession,

    // Actions
    loadSessions,
    createSession,
    deleteSession,
    sendMessage,
    toggleMcpServer,
    loadSettings,
    saveSettings,
    addMessage,
  }
})
