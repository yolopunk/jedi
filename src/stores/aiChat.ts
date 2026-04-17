import { createAnthropic } from '@ai-sdk/anthropic'
import { createOpenAI } from '@ai-sdk/openai'
import { createOpenAICompatible } from '@ai-sdk/openai-compatible'
import { invoke } from '@tauri-apps/api/core'
import { streamText } from 'ai'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
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

    // Add user message locally
    session.messages.push(userMessage)

    isLoading.value = true
    streamingContent.value = ''
    error.value = null

    try {
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

      // Convert messages to AI SDK format
      type AIMessage = { role: 'user' | 'assistant' | 'system'; content: string }
      const aiMessages: AIMessage[] = session.messages.map(m => ({
        role: m.role as AIMessage['role'],
        content: m.content,
      }))

      // Use streamText for streaming
      const result = streamText({
        model: providerClient.languageModel(model),
        messages: aiMessages,
      })

      // Collect streaming response
      let fullContent = ''
      for await (const chunk of result.fullStream) {
        if (chunk.type === 'text-delta') {
          fullContent += (chunk as any).delta ?? (chunk as any).text ?? ''
          streamingContent.value = fullContent
        }
      }

      // Add assistant message
      const assistantMessage: ChatMessage = {
        role: 'assistant',
        content: fullContent,
        timestamp: Date.now(),
      }
      session.messages.push(assistantMessage)

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
      // Remove failed user message
      session.messages.pop()
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
