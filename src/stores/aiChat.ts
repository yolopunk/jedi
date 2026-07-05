import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  type ChatTurn,
  DEFAULT_STEP_LIMIT,
  distillTitle,
  runAgent,
  summarizeUsage,
  summarizeValue,
} from '@/agent/runAgent'
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

// ========== Trace id helper ==========

let traceCounter = 0

function nextTraceId(prefix: string): string {
  traceCounter += 1
  return `${prefix}-${Date.now()}-${traceCounter}`
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

  // Normalize a session coming back from the SQLite store into the UI shape:
  // restore message timestamps and parse the persisted agent metadata JSON.
  function normalizeSession(raw: any): Session {
    const messages: ChatMessage[] = (raw.messages || []).map((m: any) => {
      let metadata: MessageMetadata | undefined
      if (m.metadata) {
        try {
          metadata = typeof m.metadata === 'string' ? JSON.parse(m.metadata) : m.metadata
        } catch {
          metadata = undefined
        }
      }
      return {
        role: m.role,
        content: m.content,
        timestamp: m.created_at ? Date.parse(m.created_at) : undefined,
        metadata,
      }
    })
    return {
      id: raw.id,
      title: raw.title,
      provider: raw.provider,
      model: raw.model,
      created_at: raw.created_at,
      updated_at: raw.updated_at,
      messages,
    }
  }

  // Actions
  async function loadSessions() {
    try {
      const raw = await invoke<any[]>('list_sessions')
      sessions.value = raw.map(normalizeSession)
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

  // Titles we treat as "unnamed" and are free to overwrite with a distilled one.
  const DEFAULT_TITLES = new Set(['新对话', '新会话', 'New Chat', ''])

  /** Quick provisional title from the first user message (instant, no LLM). */
  function heuristicTitle(text: string): string {
    const oneLine = text.replace(/\s+/g, ' ').trim()
    return oneLine.slice(0, 18) || '新对话'
  }

  /** Rename a session locally and persist it (best-effort). */
  async function updateSessionTitle(sessionId: string, title: string) {
    const clean = title.trim()
    if (!clean) return
    const session = sessions.value.find(s => s.id === sessionId)
    if (session) session.title = clean
    try {
      await invoke('update_session_title', {
        request: { session_id: sessionId, title: clean },
      })
    } catch (e) {
      console.error('Failed to update session title:', e)
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
    const stepLimit = DEFAULT_STEP_LIMIT

    // Create session if needed
    if (!currentSession.value) {
      await createSession('新对话', provider, model)
    }

    const session = currentSession.value!
    // Only auto-name a session whose title is still the placeholder.
    const shouldAutoTitle = DEFAULT_TITLES.has(session.title)
    const userMessage: ChatMessage = { role: 'user', content, timestamp: Date.now() }
    const assistantDraft: ChatMessage = {
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
    session.messages.push(assistantDraft)
    // Vue wraps a pushed plain object in a reactive *proxy*; the local
    // `assistantDraft` still points at the raw target. Mutating the raw target's
    // `content` / `metadata` during streaming bypasses the proxy's set trap, so no
    // re-render is triggered and the bubble only updates once at the very end.
    // Re-bind to the stored proxy so every streaming mutation is reactive.
    const assistantMessage = session.messages[session.messages.length - 1]

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

      // Instant provisional title from the first message so the sidebar stops
      // showing "新会话"; refined into a distilled title once the answer lands.
      if (shouldAutoTitle) {
        void updateSessionTitle(session.id, heuristicTitle(content))
      }

      // Get API key for provider
      const apiKeyInfo = await providerConfigStore.getApiKey(provider)
      if (!apiKeyInfo) {
        throw new Error(`API key not configured for provider: ${provider}`)
      }

      const enabledToolNames = skillRegistry.listAutoCallable().map(skill => skill.name)
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

      // Correlate per-tool execution between runAgent's onToolStart / onToolEnd hooks.
      const toolStepMap = new Map<
        string,
        { traceEntry: AgentTraceDetail; step: ReturnType<typeof agentStore.startStep> }
      >()

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

      // Convert messages to the shared executor format
      const aiMessages: ChatTurn[] = session.messages
        .filter(m => !m.isStreaming)
        .map(m => ({
          role: m.role as ChatTurn['role'],
          content: m.content,
        }))

      let planCompleted = false
      let streamingHasText = false
      const ensurePlanCompleted = (summary: string) => {
        if (!planCompleted) {
          agentStore.completeStep(planStep, summary)
          completeTrace(planTrace!, summary)
          planCompleted = true
        }
      }

      // Drive the run through the shared executor. All streamText plumbing and the
      // final-answer resolution live in runAgent; here we only map its events onto
      // the trace/agent-store bookkeeping that the chat UI renders.
      const runResult = await runAgent(
        {
          provider,
          model,
          apiKey: apiKeyInfo.key,
          endpoint: apiKeyInfo.endpoint,
          messages: aiMessages,
          sessionId: session.id,
          stepLimit,
        },
        {
          onToolStart: ({ skillId, skillName, skillDescription, args, startedAt }) => {
            const relatedDecisionTrace = [...toolDecisionTraces.values()]
              .reverse()
              .find(trace => trace.toolName === skillName && trace.status === 'running')
            const traceEntry = pushTrace({
              id: `${skillId}-${startedAt}`,
              type: 'tool',
              status: 'running',
              title: `Executing ${skillName}`,
              content: `Executing ${skillName}. ${skillDescription}`,
              input: args,
              timestamp: startedAt,
              toolName: skillName,
              toolCallId: relatedDecisionTrace?.toolCallId,
            })
            if (relatedDecisionTrace?.toolCallId) {
              const traces = toolExecutionTraces.get(relatedDecisionTrace.toolCallId) || []
              traces.push(traceEntry)
              toolExecutionTraces.set(relatedDecisionTrace.toolCallId, traces)
            }
            const step = agentStore.startStep(
              'tool',
              `Calling ${skillName}`,
              summarizeValue(args, 600)
            )
            toolStepMap.set(`${skillId}-${startedAt}`, { traceEntry, step })
            agentStore.setStatus('executing')
            if (assistantMessage.metadata?.run) {
              assistantMessage.metadata.run.toolCount += 1
            }
          },
          onToolEnd: ({ skillId, startedAt, output, error: toolError }) => {
            const rec = toolStepMap.get(`${skillId}-${startedAt}`)
            if (!rec) return
            if (toolError) {
              failTrace(rec.traceEntry, toolError)
              agentStore.failStep(rec.step, toolError)
            } else {
              completeTrace(rec.traceEntry, output)
              agentStore.completeStep(rec.step, summarizeValue(output, 1000))
            }
          },
          onStepFinish: ({
            stepNumber,
            finishReason,
            usage,
            stepKind,
            stepText,
            toolCalls,
            toolResults,
            text,
          }) => {
            stepCounter += 1
            assistantMessage.metadata!.finishReason = finishReason
            assistantMessage.metadata!.usage = usage
            if (assistantMessage.metadata?.run) {
              assistantMessage.metadata.run.finishReason = finishReason
              assistantMessage.metadata.run.usage = usage
            }
            if (stepText) {
              assistantMessage.content = stepText
              if (!streamingHasText) {
                streamingContent.value = stepText
              }
            }
            pushTrace({
              id: nextTraceId('step'),
              type: 'step',
              status: 'done',
              title: `LLM step ${stepCounter}`,
              content: `Step ${stepCounter} finished as ${stepKind} with ${finishReason}.`,
              output: {
                type: stepKind,
                finishReason,
                usage,
                usageSummary: summarizeUsage(usage),
                toolCalls,
                toolResults,
                text,
              },
              timestamp: Date.now(),
              stepIndex: currentStreamStepIndex,
            })
            currentStreamStepIndex = stepNumber + 2
          },
          onTextDelta: ({ fullContent }) => {
            streamingHasText = true
            streamingContent.value = fullContent
            assistantMessage.content = fullContent
            updateTrace(answerTrace!, {
              content: fullContent
                ? `Streaming ${fullContent.length} chars of assistant output.`
                : 'Streaming assistant response...',
            })
          },
          onReasoningDelta: ({ fullReasoning }) => {
            assistantMessage.metadata!.reasoning = fullReasoning
            const now = Date.now()
            if (
              now - lastReasoningTraceUpdateAt > 250 ||
              fullReasoning.length < 240 ||
              fullReasoning.length % 320 < 40
            ) {
              updateTrace(reasoningTrace!, {
                content: summarizeValue(fullReasoning, 1400),
              })
              lastReasoningTraceUpdateAt = now
            }
            if (fullReasoning.trim()) {
              ensurePlanCompleted(summarizeValue(fullReasoning, 800))
            }
          },
          onToolCall: ({ toolName, toolCallId, input }) => {
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
            ensurePlanCompleted('Selected a tool for verification.')
          },
          onToolResult: ({ toolName, toolCallId, output }) => {
            const decisionTrace = toolCallId ? toolDecisionTraces.get(toolCallId) : undefined
            if (decisionTrace?.status === 'running') {
              completeTrace(decisionTrace, output)
            }
            const executionTrace = toolCallId
              ? (toolExecutionTraces.get(toolCallId) || []).find(
                  trace => trace.status === 'running'
                )
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
          },
          onFinish: ({ finishReason }) => {
            assistantMessage.metadata!.finishReason = finishReason
            if (assistantMessage.metadata?.run) {
              assistantMessage.metadata.run.finishReason = finishReason
            }
          },
        }
      )

      ensurePlanCompleted('Answered directly.')
      completeTrace(
        reasoningTrace!,
        summarizeValue(runResult.reasoning || 'No reasoning summary emitted.', 1400)
      )
      let fullContent = runResult.text
      if (!fullContent.trim()) {
        fullContent =
          '模型本轮没有返回可展示的文本内容。请在 Trace 中查看 finish reason、usage 和原始事件。'
      }
      completeTrace(
        answerTrace!,
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

      // Persist to backend, including the agent run metadata (trace / usage /
      // reasoning) so it survives a restart.
      let metadataJson: string | undefined
      try {
        metadataJson = assistantMessage.metadata
          ? JSON.stringify(assistantMessage.metadata)
          : undefined
      } catch {
        metadataJson = undefined
      }
      await invoke('append_message', {
        request: {
          session_id: session.id,
          role: 'assistant',
          content: fullContent,
          metadata: metadataJson,
        },
      })

      // Distill a concise title from the first exchange. Best-effort and
      // non-blocking: the provisional heuristic title stays if this fails.
      if (shouldAutoTitle && fullContent.trim()) {
        void distillTitle({
          provider,
          model,
          apiKey: apiKeyInfo.key,
          endpoint: apiKeyInfo.endpoint,
          userMessage: content,
          assistantMessage: fullContent,
        })
          .then(title => {
            if (title) return updateSessionTitle(session.id, title)
          })
          .catch(e => console.warn('Session title distillation failed:', e))
      }
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
      // Persist the failed turn so the real error is visible in history (and the DB).
      try {
        await invoke('append_message', {
          request: {
            session_id: session.id,
            role: 'assistant',
            content: assistantMessage.content,
            metadata: assistantMessage.metadata
              ? JSON.stringify(assistantMessage.metadata)
              : undefined,
          },
        })
      } catch {}
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
    updateSessionTitle,
    sendMessage,
    toggleMcpServer,
    loadSettings,
    saveSettings,
    addMessage,
  }
})
