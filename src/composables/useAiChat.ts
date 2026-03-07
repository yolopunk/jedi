// AI Chat 状态管理
// Phase 2: MVP 核心功能

import { ref, computed } from 'vue'

// 消息角色类型
export type MessageRole = 'system' | 'user' | 'assistant'

// 消息接口
export interface ChatMessage {
  id: string
  role: MessageRole
  content: string
  createdAt: string
}

// 会话接口
export interface ChatSession {
  id: string
  title: string
  createdAt: string
  updatedAt: string
  messages: ChatMessage[]
  provider: string
  model: string
}

// 提供商信息
export interface ProviderInfo {
  provider: string
  hasKey: boolean
}

// API Key 信息
export interface ApiKeyInfo {
  provider: string
  maskedKey: string
  endpoint?: string
}

export function useAiChat() {
  // 状态
  const activeTab = ref<'chat' | 'settings'>('settings')
  const sessions = ref<ChatSession[]>([])
  const currentSessionId = ref<string | null>(null)
  const isLoading = ref(false)
  const isSending = ref(false)
  const providers = ref<ProviderInfo[]>([])
  const apiKeys = ref<ApiKeyInfo[]>([])

  // 当前会话
  const currentSession = computed(() => {
    if (!currentSessionId.value) return null
    return sessions.value.find(s => s.id === currentSessionId.value) || null
  })

  // 当前会话的消息
  const currentMessages = computed(() => {
    return currentSession.value?.messages || []
  })

  // 切换 Tab
  function setActiveTab(tab: 'chat' | 'settings') {
    activeTab.value = tab
  }

  // 加载会话列表
  async function loadSessions() {
    isLoading.value = true
    try {
      // TODO: 调用 Tauri command 加载会话
      sessions.value = []
    } finally {
      isLoading.value = false
    }
  }

  // 创建新会话
  async function createSession(title: string, provider: string, model: string) {
    isLoading.value = true
    try {
      // TODO: 调用 Tauri command 创建会话
      const newSession: ChatSession = {
        id: Date.now().toString(),
        title,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        messages: [],
        provider,
        model
      }
      sessions.value.unshift(newSession)
      currentSessionId.value = newSession.id
      return newSession
    } finally {
      isLoading.value = false
    }
  }

  // 选择会话
  function selectSession(sessionId: string) {
    currentSessionId.value = sessionId
  }

  // 删除会话
  async function deleteSession(sessionId: string) {
    isLoading.value = true
    try {
      // TODO: 调用 Tauri command 删除会话
      sessions.value = sessions.value.filter(s => s.id !== sessionId)
      if (currentSessionId.value === sessionId) {
        currentSessionId.value = sessions.value[0]?.id || null
      }
    } finally {
      isLoading.value = false
    }
  }

  // 发送消息
  async function sendMessage(content: string) {
    if (!currentSessionId.value || !content.trim()) return

    isSending.value = true
    try {
      const session = currentSession.value
      if (!session) return

      // 添加用户消息
      const userMessage: ChatMessage = {
        id: Date.now().toString(),
        role: 'user',
        content: content.trim(),
        createdAt: new Date().toISOString()
      }
      session.messages.push(userMessage)

      // TODO: 调用 Tauri command 发送消息到 AI
      // 模拟 AI 响应
      const aiMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: '这是一个模拟的 AI 响应，实际功能开发中...',
        createdAt: new Date().toISOString()
      }
      session.messages.push(aiMessage)
      session.updatedAt = new Date().toISOString()
    } finally {
      isSending.value = false
    }
  }

  // 加载提供商列表
  async function loadProviders() {
    try {
      // TODO: 调用 Tauri command 加载提供商列表
      providers.value = []
    } catch (error) {
      console.error('Failed to load providers:', error)
    }
  }

  // 加载 API Keys
  async function loadApiKeys() {
    try {
      // TODO: 调用 Tauri command 加载 API Keys
      apiKeys.value = []
    } catch (error) {
      console.error('Failed to load API keys:', error)
    }
  }

  // 初始化
  async function initialize() {
    await Promise.all([
      loadSessions(),
      loadProviders(),
      loadApiKeys()
    ])
  }

  return {
    // 状态
    activeTab,
    sessions,
    currentSessionId,
    currentSession,
    currentMessages,
    isLoading,
    isSending,
    providers,
    apiKeys,

    // 方法
    setActiveTab,
    loadSessions,
    createSession,
    selectSession,
    deleteSession,
    sendMessage,
    loadProviders,
    loadApiKeys,
    initialize
  }
}
