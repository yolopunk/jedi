import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import {
  agentChat,
  agentCancel,
  toolConfirm,
  toolUndo,
  turnUndo,
  mcpConnect,
  mcpDisconnect,
  getModelsForProvider,
  type AgentEvent,
  type McpServerConfig,
} from '../api/ai-chat'

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

// 预定义的提供商配置
export const PROVIDER_CONFIGS: Record<string, { name: string; models: Model[] }> = {
  openai: {
    name: 'OpenAI',
    models: [
      { id: 'gpt-4o', name: 'GPT-4o', provider: 'openai', contextLength: 128000 },
      { id: 'gpt-4o-mini', name: 'GPT-4o Mini', provider: 'openai', contextLength: 128000 },
      { id: 'gpt-4-turbo', name: 'GPT-4 Turbo', provider: 'openai', contextLength: 128000 },
      { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo', provider: 'openai', contextLength: 16384 },
    ],
  },
  anthropic: {
    name: 'Anthropic',
    models: [
      { id: 'claude-sonnet-4-20250514', name: 'Claude Sonnet 4', provider: 'anthropic', contextLength: 200000 },
      { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet', provider: 'anthropic', contextLength: 200000 },
      { id: 'claude-3-opus-20240229', name: 'Claude 3 Opus', provider: 'anthropic', contextLength: 200000 },
    ],
  },
  google: {
    name: 'Google (Gemini)',
    models: [
      { id: 'gemini-2.0-flash', name: 'Gemini 2.0 Flash', provider: 'google', contextLength: 1048576 },
      { id: 'gemini-1.5-pro', name: 'Gemini 1.5 Pro', provider: 'google', contextLength: 1048576 },
      { id: 'gemini-1.5-flash', name: 'Gemini 1.5 Flash', provider: 'google', contextLength: 1048576 },
    ],
  },
  deepseek: {
    name: 'DeepSeek',
    models: [
      { id: 'deepseek-chat', name: 'DeepSeek Chat', provider: 'deepseek', contextLength: 64000 },
      { id: 'deepseek-coder', name: 'DeepSeek Coder', provider: 'deepseek', contextLength: 128000 },
    ],
  },
}

// 内置工具分组（对应后端 native 工具的 group）
export const DEFAULT_MCP_SERVERS: McpServer[] = [
  { id: 'hosts', name: 'Hosts Manager', description: '管理系统 Hosts 文件', enabled: false, icon: 'mdi-dns' },
  { id: 'wallpaper', name: 'Wallpaper', description: '知识壁纸浏览与设置', enabled: false, icon: 'mdi-image' },
  { id: 'podcast', name: 'Podcast', description: '播客订阅与剧集', enabled: false, icon: 'mdi-podcast' },
  { id: 'system', name: 'System Info', description: '系统信息查询', enabled: false, icon: 'mdi-monitor' },
]

export const useAiChatStore = defineStore('aiChat', () => {
  // State
  const providers = ref<ProviderInfo[]>([])
  const sessions = ref<Session[]>([])
  const currentSessionId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const streamingContent = ref<string>('')

  // Agent 执行追踪（工具调用过程），用于 Agent Trace 面板
  const agentTrace = ref<AgentEvent[]>([])
  // 当前 Agent 回路的 request_id（用于确认/取消/回滚）
  const agentRequestId = ref<string | null>(null)

  // UI State
  const selectedModelId = ref<string | null>(null)
  const selectedProvider = ref<string>('openai')
  const enabledMcpServers = ref<string[]>([])
  const mcpServers = ref<McpServer[]>([...DEFAULT_MCP_SERVERS])
  // 第三方 MCP server 配置（持久化于 localStorage）与已连接 id
  const thirdPartyMcpServers = ref<McpServerConfig[]>([])
  const mcpConnectedIds = ref<string[]>([])
  // 模型是否支持 function calling（来自 models.dev，best-effort 缓存）
  const modelToolSupport = ref<Record<string, boolean>>({})

  // 惰性查询所选模型是否支持工具调用（未知返回 undefined → 后端按默认注入工具）
  async function resolveModelToolSupport(provider: string, model: string): Promise<boolean | undefined> {
    if (model in modelToolSupport.value) return modelToolSupport.value[model]
    try {
      const models = await getModelsForProvider(provider)
      models.forEach(m => { modelToolSupport.value[m.id] = m.tool_call })
    } catch (e) {
      console.warn('无法获取模型能力:', e)
    }
    return modelToolSupport.value[model]
  }

  // Chat settings
  const temperature = ref<number>(0.7)
  const maxTokens = ref<number>(4096)
  const streamEnabled = ref<boolean>(true)
  // 确认策略：normal=写操作需确认 / auto=仅系统级确认
  const confirmMode = ref<'normal' | 'auto'>('normal')

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
      const config = PROVIDER_CONFIGS[provider.provider]
      if (config) {
        config.models.forEach(model => {
          models.push({ ...model, providerName: config.name })
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
    agentTrace.value = []
    error.value = null

    try {
      // 当启用了工具分组或连接了第三方 MCP 时，走 Agent 工具调用回路
      if (enabledMcpServers.value.length > 0 || mcpConnectedIds.value.length > 0) {
        const requestId = `agent-${Date.now()}`
        agentRequestId.value = requestId

        const unlisten = await listen<AgentEvent>(`agent-event-${requestId}`, (event) => {
          const payload = event.payload
          // 流式增量：直接累加到 streamingContent，不进 Trace
          if (payload.type === 'content_delta') {
            streamingContent.value += payload.text
            return
          }
          agentTrace.value.push(payload)
          if (payload.type === 'content') {
            // 最终回答（覆盖流式累加，去掉可能的中间预备文本）
            streamingContent.value = payload.text
          }
        })

        const supportsTools = await resolveModelToolSupport(session.provider, session.model)

        try {
          const finalContent = await agentChat({
            provider: session.provider,
            model: session.model,
            messages: session.messages.map(m => ({ role: m.role, content: m.content })),
            servers: [...enabledMcpServers.value, ...mcpConnectedIds.value],
            temperature: options?.temperature ?? temperature.value,
            maxTokens: options?.maxTokens ?? maxTokens.value,
            requestId,
            confirmMode: confirmMode.value,
            supportsTools,
          })

          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: finalContent,
            timestamp: Date.now()
          }
          session.messages.push(assistantMessage)
        } finally {
          unlisten()
        }

        await invoke('append_message', {
          sessionId: session.id,
          message: session.messages[session.messages.length - 1],
        })
        return
      }

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

  // 对挂起的工具调用做确认
  async function confirmTool(callId: string, approve: boolean, editedArgs?: Record<string, unknown>) {
    if (!agentRequestId.value) return
    await toolConfirm(agentRequestId.value, callId, approve, editedArgs)
  }

  // 取消当前 Agent 回路
  async function cancelAgent() {
    if (!agentRequestId.value) return
    await agentCancel(agentRequestId.value)
  }

  // 单步回滚
  async function undoTool(undoToken: string) {
    if (!agentRequestId.value) return
    return await toolUndo(agentRequestId.value, undoToken)
  }

  // 整回合回滚
  async function undoTurn() {
    if (!agentRequestId.value) return
    return await turnUndo(agentRequestId.value)
  }

  // ===== 第三方 MCP server 管理 =====
  function loadMcpServers() {
    try {
      const saved = localStorage.getItem('mcp-third-party')
      if (saved) thirdPartyMcpServers.value = JSON.parse(saved)
    } catch (e) {
      console.error('Failed to load MCP servers:', e)
    }
  }

  function saveMcpServers() {
    try {
      localStorage.setItem('mcp-third-party', JSON.stringify(thirdPartyMcpServers.value))
    } catch (e) {
      console.error('Failed to save MCP servers:', e)
    }
  }

  function addMcpServer(config: McpServerConfig) {
    const idx = thirdPartyMcpServers.value.findIndex(s => s.id === config.id)
    if (idx >= 0) thirdPartyMcpServers.value[idx] = config
    else thirdPartyMcpServers.value.push(config)
    saveMcpServers()
  }

  async function removeMcpServer(id: string) {
    await disconnectMcp(id).catch(() => {})
    thirdPartyMcpServers.value = thirdPartyMcpServers.value.filter(s => s.id !== id)
    saveMcpServers()
  }

  async function connectMcp(config: McpServerConfig) {
    const status = await mcpConnect(config)
    if (!mcpConnectedIds.value.includes(status.id)) {
      mcpConnectedIds.value.push(status.id)
    }
    return status
  }

  async function disconnectMcp(id: string) {
    await mcpDisconnect(id)
    mcpConnectedIds.value = mcpConnectedIds.value.filter(x => x !== id)
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
    agentTrace,
    agentRequestId,
    confirmMode,
    selectedModelId,
    selectedProvider,
    enabledMcpServers,
    mcpServers,
    thirdPartyMcpServers,
    mcpConnectedIds,
    temperature,
    maxTokens,
    streamEnabled,

    // Computed
    currentSession,
    configuredProviders,
    availableModels,
    selectedModel,

    // Actions
    loadProviders,
    saveApiKey,
    deleteApiKey,
    loadSessions,
    createSession,
    deleteSession,
    sendMessage,
    confirmTool,
    cancelAgent,
    undoTool,
    undoTurn,
    loadMcpServers,
    saveMcpServers,
    addMcpServer,
    removeMcpServer,
    connectMcp,
    disconnectMcp,
    toggleMcpServer,
    setSelectedModel,
    loadSettings,
    saveSettings,
  }
})
