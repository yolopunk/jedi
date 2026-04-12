import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

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
  const sessions = ref<Session[]>([])
  const currentSessionId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const streamingContent = ref<string>('')

  // MCP State
  const enabledMcpServers = ref<string[]>([])
  const mcpServers = ref<McpServer[]>([...DEFAULT_MCP_SERVERS])

  // Computed
  const currentSession = computed(() =>
    sessions.value.find(s => s.id === currentSessionId.value) || null
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

  async function createSession(title: string = '新对话', provider: string = 'openai', model: string = 'gpt-4o-mini') {
    try {
      const session = await invoke('create_session', {
        title,
        provider,
        model
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

  async function sendMessage(content: string): Promise<void> {
    if (!currentSession.value) {
      // Create a new session if none exists
      await createSession()
      if (!currentSession.value) {
        throw new Error('没有活动的会话')
      }
    }

    const session = currentSession.value
    const userMessage: ChatMessage = { role: 'user', content, timestamp: Date.now() }

    // 添加用户消息
    session.messages.push(userMessage)

    isLoading.value = true
    streamingContent.value = ''
    error.value = null

    // 流式响应收集
    let fullContent = ''
    let streamDone = false
    let streamError: string | null = null

    try {
      // 流式响应
      const requestId = `req-${Date.now()}`

      // 创建一个 Promise，在收到 Done 事件时 resolve
      const waitForStreamDone = new Promise<void>((resolve, reject) => {
        listen(
          `chat-stream-${requestId}`,
          (event) => {
            const payload = event.payload as { Content?: { text?: string } } | { Done?: null } | { Error?: { message?: string } }
            if ('Content' in payload && payload.Content?.text) {
              fullContent += payload.Content.text
              streamingContent.value = fullContent
            }
            if ('Done' in payload) {
              streamDone = true
              resolve()
            }
            if ('Error' in payload && payload.Error?.message) {
              streamError = payload.Error.message
              reject(new Error(streamError))
            }
          }
        ).then((unlisten) => {
          // Store unlisten for cleanup
          ;(window as any).__streamUnlisten = unlisten
        })
      })

      try {
        // 启动流式请求
        await invoke('send_chat_message_stream', {
          provider: session.provider,
          model: session.model,
          messages: session.messages,
          requestId,
        })

        // 等待流结束（通过 Done 事件触发）
        // 添加超时保护
        await Promise.race([
          waitForStreamDone,
          new Promise((_, reject) => setTimeout(() => reject(new Error('Stream timeout (30s)')), 30000))
        ])

        if (!streamDone && !streamError) {
          // 兜底：如果没收到 Done 事件但有内容，也允许通过
          console.warn('Stream finished without explicit Done event')
        }

        const assistantMessage: ChatMessage = {
          role: 'assistant',
          content: fullContent || streamingContent.value,
          timestamp: Date.now()
        }
        session.messages.push(assistantMessage)
      } finally {
        // 清理监听器
        const unlisten = (window as any).__streamUnlisten
        if (unlisten) unlisten()
        delete (window as any).__streamUnlisten
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
