import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ModelsDevResponse, ProviderInfo as ModelsDevProviderInfo, ModelInfo } from '@/types/models.dev'

// 提供商信息（后端返回格式）
export interface ProviderInfo {
  provider: string
  has_key: boolean
}

// Model 接口
export interface Model {
  id: string
  name: string
  provider: string
  contextLength?: number
}

// MCP Server 接口
export interface McpServer {
  id: string
  name: string
  description?: string
  enabled: boolean
  icon?: string
}

// 消息格式（后端期望格式）
export interface ChatMessage {
  role: 'system' | 'user' | 'assistant'
  content: string
  timestamp?: number
  isStreaming?: boolean
  error?: string
}

// 兼容旧组件的类型导出
export interface Message extends ChatMessage { }

// 兼容旧组件的Provider类型
export interface Provider {
  id: string
  name: string
  provider?: string
  providerId?: string
  apiKey?: string
  endpoint?: string
  baseUrl?: string
  enabled: boolean
  isActive?: boolean
  type?: string
  models?: any[]
}

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
  { id: 'hosts', name: 'Hosts Manager', description: '管理系统Hosts文件', enabled: false, icon: 'mdi-dns' },
  { id: 'filesystem', name: 'Filesystem', description: '文件系统操作', enabled: false, icon: 'mdi-folder' },
  { id: 'browser', name: 'Browser', description: '网页浏览和搜索', enabled: false, icon: 'mdi-web' },
]

export const useAiChatStore = defineStore('aiChat', () => {
  // State
  const providers = ref<ProviderInfo[]>([])
  const sessions = ref<Session[]>([])
  const currentSessionId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const streamingContent = ref<string>('')
  const modelsDevData = ref<ModelsDevResponse | null>(null)
  const modelsDevLoading = ref(false)
  const modelsDevError = ref<string | null>(null)

  // UI State
  const selectedModelId = ref<string | null>(null)
  const selectedProvider = ref<string>('openai')
  const enabledMcpServers = ref<string[]>([])
  const mcpServers = ref<McpServer[]>([...DEFAULT_MCP_SERVERS])

  // Chat settings
  const temperature = ref<number>(0.7)
  const maxTokens = ref<number>(4096)
  const streamEnabled = ref<boolean>(true)

  // Models.dev data
  async function fetchModelsDev() {
    modelsDevLoading.value = true
    modelsDevError.value = null
    try {
      const response = await fetch('https://models.dev/api.json')
      if (!response.ok) throw new Error('Failed to fetch')
      modelsDevData.value = await response.json()
    } catch (e) {
      modelsDevError.value = 'Failed to load providers'
      console.error('fetchModelsDev error:', e)
    } finally {
      modelsDevLoading.value = false
    }
  }

  function getProvidersFromModelsDev(): ModelsDevProviderInfo[] {
    if (!modelsDevData.value) return []
    return Object.values(modelsDevData.value)
  }

  function getModelsForProvider(providerId: string): ModelInfo[] {
    if (!modelsDevData.value) return []
    const provider = modelsDevData.value[providerId]
    if (!provider) return []
    return Object.values(provider.models)
  }

  // Computed
  const currentSession = computed(() =>
    sessions.value.find(s => s.id === currentSessionId.value) || null
  )

  const configuredProviders = computed(() =>
    providers.value.filter(p => p.has_key)
  )

  const availableModels = computed(() => {
    const models: (Model & { providerName: string })[] = []
    for (const provider of configuredProviders.value) {
      const providerData = getProvidersFromModelsDev().find(p => p.id === provider.provider)
      if (providerData) {
        const providerModels = getModelsForProvider(provider.provider)
        providerModels.forEach(model => {
          models.push({
            id: model.id,
            name: model.name,
            provider: provider.provider,
            contextLength: model.limit?.context,
            providerName: providerData.name,
          })
        })
      }
    }
    return models
  })

  const selectedModel = computed(() => {
    if (!selectedModelId.value) return null
    return availableModels.value.find(m => m.id === selectedModelId.value) || null
  })

  // Initialize with default model
  function initializeDefaultModel() {
    if (availableModels.value.length > 0 && !selectedModelId.value) {
      selectedModelId.value = availableModels.value[0].id
      selectedProvider.value = availableModels.value[0].provider
    }
  }

  // Actions
  async function loadProviders() {
    try {
      providers.value = await invoke('list_api_key_providers')
      error.value = null
      initializeDefaultModel()
    } catch (e) {
      error.value = `加载提供商失败: ${e}`
      console.error('Failed to load providers:', e)
    }
  }

  async function saveApiKey(provider: string, key: string, endpoint?: string) {
    try {
      await invoke('store_api_key', {
        request: {
          provider,
          key,
          endpoint: endpoint || null
        }
      })
      await loadProviders()
      error.value = null
    } catch (e) {
      error.value = `保存 API Key 失败: ${e}`
      console.error('Failed to save API key:', e)
      throw e
    }
  }

  async function deleteApiKey(provider: string) {
    try {
      await invoke('delete_api_key', { provider })
      await loadProviders()
      error.value = null
    } catch (e) {
      error.value = `删除 API Key 失败: ${e}`
      console.error('Failed to delete API key:', e)
      throw e
    }
  }

  async function loadSessions() {
    try {
      sessions.value = await invoke('list_sessions')
      error.value = null
    } catch (e) {
      error.value = `加载会话失败: ${e}`
      console.error('Failed to load sessions:', e)
    }
  }

  async function createSession(title: string = '新对话', provider?: string, model?: string) {
    const useProvider = provider || selectedProvider.value
    const useModel = model || selectedModelId.value || 'gpt-4o-mini'

    try {
      const session = await invoke('create_session', {
        title,
        provider: useProvider,
        model: useModel
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
      await invoke('delete_session', { sessionId })
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

  async function sendMessage(content: string, options?: {
    stream?: boolean
    temperature?: number
    maxTokens?: number
  }) {
    if (!currentSession.value) {
      // Create a new session if none exists
      await createSession()
      if (!currentSession.value) {
        throw new Error('没有活动的会话')
      }
    }

    const session = currentSession.value
    const userMessage: ChatMessage = { role: 'user', content, timestamp: Date.now() }

    // Update session with selected model and provider
    if (selectedModelId.value) {
      session.model = selectedModelId.value
    }
    if (selectedProvider.value) {
      session.provider = selectedProvider.value
    }

    // 添加用户消息
    session.messages.push(userMessage)

    isLoading.value = true
    streamingContent.value = ''
    error.value = null

    try {
      const useStream = options?.stream ?? streamEnabled.value
      if (useStream) {
        // 流式响应
        const requestId = `req-${Date.now()}`

        // 监听流式事件
        const unlisten = await listen<string>('chat-stream-chunk', (event) => {
          streamingContent.value += event.payload
        })

        try {
          const response = await invoke('send_chat_message_stream', {
            provider: session.provider,
            model: session.model,
            messages: session.messages,
            temperature: options?.temperature ?? temperature.value,
            maxTokens: options?.maxTokens ?? maxTokens.value,
            requestId,
          })

          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: response as string,
            timestamp: Date.now()
          }
          session.messages.push(assistantMessage)
        } finally {
          unlisten()
        }
      } else {
        // 非流式响应
        const response = await invoke('send_chat_message', {
          provider: session.provider,
          model: session.model,
          messages: session.messages,
          temperature: options?.temperature ?? temperature.value,
          maxTokens: options?.maxTokens ?? maxTokens.value,
        })

        const assistantMessage: ChatMessage = {
          role: 'assistant',
          content: response as string,
          timestamp: Date.now()
        }
        session.messages.push(assistantMessage)
      }

      // 更新会话
      await invoke('append_message', {
        sessionId: session.id,
        message: session.messages[session.messages.length - 1],
      })

    } catch (e) {
      error.value = `发送消息失败: ${e}`
      console.error('Failed to send message:', e)
      // 移除失败的用户消息
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

  // Set selected model
  function setSelectedModel(modelId: string) {
    selectedModelId.value = modelId
    const model = availableModels.value.find(m => m.id === modelId)
    if (model) {
      selectedProvider.value = model.provider
    }
  }

  // Load settings from localStorage
  function loadSettings() {
    try {
      const saved = localStorage.getItem('chat-settings')
      if (saved) {
        const settings = JSON.parse(saved)
        if (settings.temperature !== undefined) temperature.value = settings.temperature
        if (settings.maxTokens !== undefined) maxTokens.value = settings.maxTokens
        if (settings.streamEnabled !== undefined) streamEnabled.value = settings.streamEnabled
        if (settings.selectedModelId) selectedModelId.value = settings.selectedModelId
        if (settings.selectedProvider) selectedProvider.value = settings.selectedProvider
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

  // Save settings to localStorage
  function saveSettings() {
    try {
      const settings = {
        temperature: temperature.value,
        maxTokens: maxTokens.value,
        streamEnabled: streamEnabled.value,
        selectedModelId: selectedModelId.value,
        selectedProvider: selectedProvider.value,
        enabledMcpServers: enabledMcpServers.value,
      }
      localStorage.setItem('chat-settings', JSON.stringify(settings))
    } catch (e) {
      console.error('Failed to save AI chat settings:', e)
    }
  }

  return {
    // State
    providers,
    sessions,
    currentSessionId,
    isLoading,
    error,
    streamingContent,
    modelsDevData,
    modelsDevLoading,
    modelsDevError,
    selectedModelId,
    selectedProvider,
    enabledMcpServers,
    mcpServers,
    temperature,
    maxTokens,
    streamEnabled,

    // Computed
    currentSession,
    configuredProviders,
    availableModels,
    selectedModel,

    // Actions
    fetchModelsDev,
    getProvidersFromModelsDev,
    getModelsForProvider,
    loadProviders,
    saveApiKey,
    deleteApiKey,
    loadSessions,
    createSession,
    deleteSession,
    sendMessage,
    toggleMcpServer,
    setSelectedModel,
    loadSettings,
    saveSettings,
  }
})
