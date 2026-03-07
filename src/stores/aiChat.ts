import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// 提供商信息（后端返回格式）
export interface ProviderInfo {
  provider: string
  has_key: boolean
}

// 消息格式（后端期望格式）
export interface ChatMessage {
  role: 'system' | 'user' | 'assistant'
  content: string
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
export const PROVIDER_CONFIGS = {
  openai: {
    name: 'OpenAI',
    models: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-3.5-turbo'],
  },
  anthropic: {
    name: 'Anthropic',
    models: ['claude-sonnet-4-20250514', 'claude-3-5-sonnet-20241022', 'claude-3-opus-20240229'],
  },
  google: {
    name: 'Google (Gemini)',
    models: ['gemini-2.0-flash', 'gemini-1.5-pro', 'gemini-1.5-flash'],
  },
  deepseek: {
    name: 'DeepSeek',
    models: ['deepseek-chat', 'deepseek-coder'],
  },
} as const

export const useAiChatStore = defineStore('aiChat', () => {
  // State
  const providers = ref<ProviderInfo[]>([])
  const sessions = ref<Session[]>([])
  const currentSessionId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const streamingContent = ref<string>('')

  // Computed
  const currentSession = computed(() => 
    sessions.value.find(s => s.id === currentSessionId.value) || null
  )

  const configuredProviders = computed(() => 
    providers.value.filter(p => p.has_key)
  )

  // Actions
  async function loadProviders() {
    try {
      providers.value = await invoke('list_api_key_providers')
      error.value = null
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

  async function createSession(title: string = '新对话', provider: string = 'openai', model: string = 'gpt-4o-mini') {
    try {
      const session = await invoke('create_session', { 
        title, 
        provider, 
        model 
      })
      sessions.value.unshift(session)
      currentSessionId.value = session.id
      error.value = null
      return session
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
      throw new Error('没有活动的会话')
    }

    const session = currentSession.value
    const userMessage: ChatMessage = { role: 'user', content }
    
    // 添加用户消息
    session.messages.push(userMessage)
    
    isLoading.value = true
    streamingContent.value = ''
    error.value = null

    try {
      if (options?.stream !== false) {
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
            temperature: options?.temperature,
            maxTokens: options?.maxTokens,
            requestId,
          })
          
          const assistantMessage: ChatMessage = { 
            role: 'assistant', 
            content: response as string 
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
          temperature: options?.temperature,
          maxTokens: options?.maxTokens,
        })
        
        const assistantMessage: ChatMessage = { 
          role: 'assistant', 
          content: response as string 
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

  return {
    // State
    providers,
    sessions,
    currentSessionId,
    isLoading,
    error,
    streamingContent,
    
    // Computed
    currentSession,
    configuredProviders,
    
    // Actions
    loadProviders,
    saveApiKey,
    deleteApiKey,
    loadSessions,
    createSession,
    deleteSession,
    sendMessage,
  }
})