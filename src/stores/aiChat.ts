import { createAnthropic } from '@ai-sdk/anthropic'
import { createOpenAI } from '@ai-sdk/openai'
import { createOpenAICompatible } from '@ai-sdk/openai-compatible'
import { invoke } from '@tauri-apps/api/core'
import { jsonSchema, stepCountIs, streamText, tool } from 'ai'
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
  type: 'think' | 'tool' | 'finish' | 'error'
  status: 'running' | 'done' | 'error'
  title: string
  content?: string
  input?: unknown
  output?: unknown
  timestamp: number
  durationMs?: number
}

export interface MessageMetadata {
  reasoning?: string
  trace?: AgentTraceDetail[]
  toolCalls?: AgentTraceDetail[]
  finishReason?: string
  usage?: unknown
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
      },
    }

    session.messages.push(userMessage)
    session.messages.push(assistantMessage)

    isLoading.value = true
    streamingContent.value = ''
    error.value = null
    agentStore.reset()
    agentStore.tracePanelOpen = true
    agentStore.setStatus('planning')

    const pushTrace = (entry: AgentTraceDetail) => {
      assistantMessage.metadata?.trace?.push(entry)
      if (entry.type === 'tool') {
        assistantMessage.metadata?.toolCalls?.push(entry)
      }
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
      const tools = Object.fromEntries(
        enabledSkills.map(skill => [
          skill.id,
          tool({
            description: `${skill.name}: ${skill.description}`,
            inputSchema: jsonSchema(skill.parameters as any),
            execute: async args => {
              const startedAt = Date.now()
              const traceEntry: AgentTraceDetail = {
                id: `${skill.id}-${startedAt}`,
                type: 'tool',
                status: 'running',
                title: skill.name,
                content: skill.description,
                input: args,
                timestamp: startedAt,
              }
              pushTrace(traceEntry)
              const step = agentStore.startStep(
                'tool',
                `Calling ${skill.name}`,
                summarizeValue(args, 600)
              )

              try {
                agentStore.setStatus('executing')
                const output = await skill.execute(args, { sessionId: session.id })
                traceEntry.status = 'done'
                traceEntry.output = output
                traceEntry.durationMs = Date.now() - startedAt
                agentStore.completeStep(step, summarizeValue(output, 1000))
                return output
              } catch (e) {
                const message = e instanceof Error ? e.message : String(e)
                traceEntry.status = 'error'
                traceEntry.output = message
                traceEntry.durationMs = Date.now() - startedAt
                agentStore.failStep(step, message)
                throw e
              }
            },
          }),
        ])
      )

      const planTrace: AgentTraceDetail = {
        id: `think-${Date.now()}`,
        type: 'think',
        status: 'running',
        title: 'Planning',
        content: '理解问题、选择是否需要工具，并准备验证路径。',
        timestamp: Date.now(),
      }
      pushTrace(planTrace)
      const planStep = agentStore.startStep('think', 'Planning response', content)

      // Convert messages to AI SDK format
      type AIMessage = { role: 'user' | 'assistant' | 'system'; content: string }
      const aiMessages: AIMessage[] = session.messages
        .filter(m => !m.isStreaming)
        .map(m => ({
          role: m.role as AIMessage['role'],
          content: m.content,
        }))

      // Use streamText for streaming
      const result = streamText({
        model: providerClient.languageModel(model),
        system: createAgentSystemPrompt(enabledSkills.map(skill => skill.name)),
        messages: aiMessages,
        tools,
        stopWhen: stepCountIs(8),
        onStepFinish: step => {
          assistantMessage.metadata!.finishReason = step.finishReason
          assistantMessage.metadata!.usage = step.usage
        },
      })

      // Collect streaming response
      let fullContent = ''
      let reasoningSummary = ''
      let planCompleted = false
      for await (const chunk of result.fullStream) {
        if (chunk.type === 'text-delta') {
          fullContent += (chunk as any).delta ?? (chunk as any).text ?? ''
          streamingContent.value = fullContent
          assistantMessage.content = fullContent
        } else if (chunk.type === 'reasoning-delta') {
          reasoningSummary += (chunk as any).delta ?? (chunk as any).text ?? ''
          assistantMessage.metadata!.reasoning = reasoningSummary
          if (!planCompleted && reasoningSummary.trim()) {
            agentStore.completeStep(planStep, summarizeValue(reasoningSummary, 800))
            planTrace.status = 'done'
            planTrace.output = summarizeValue(reasoningSummary, 800)
            planCompleted = true
          }
        } else if (chunk.type === 'tool-call') {
          if (!planCompleted) {
            agentStore.completeStep(planStep, 'Selected a tool for verification.')
            planTrace.status = 'done'
            planCompleted = true
          }
        } else if (chunk.type === 'finish') {
          assistantMessage.metadata!.finishReason = chunk.finishReason
        }
      }

      if (!planCompleted) {
        agentStore.completeStep(planStep, 'Answered directly.')
        planTrace.status = 'done'
      }

      assistantMessage.content = fullContent
      assistantMessage.isStreaming = false
      pushTrace({
        id: `finish-${Date.now()}`,
        type: 'finish',
        status: 'done',
        title: 'Completed',
        content: '已生成回答并完成本轮对话。',
        timestamp: Date.now(),
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
      pushTrace({
        id: `error-${Date.now()}`,
        type: 'error',
        status: 'error',
        title: 'Error',
        content: assistantMessage.error,
        timestamp: Date.now(),
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
