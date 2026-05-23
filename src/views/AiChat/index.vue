<template>
  <div class="chat-console-page">
    <!-- 扫描线覆盖层 -->
    <div class="scanlines"></div>
    <!-- CRT 晕影 -->
    <div class="crt-vignette"></div>

    <div class="chat-console-layout">
      <!-- 主聊天区域 -->
      <div class="chat-console-area">
        <!-- Hologram Header -->
        <div class="chat-header">
          <div class="header-logo">
            <span class="menu-icon">☰</span>
            <span class="holocron">HOLOCRON</span>
            <span class="path">/chat</span>
          </div>
          <div class="header-right">
            <div class="status-badge">
              <span class="status-dot"></span>
              <span class="status-text">{{ connectionStatus }}</span>
            </div>
            <div class="provider-display" @click="showModelSettings = true">
              <span class="provider-label">PROVIDER:</span>
              <span class="provider-name">{{ currentProviderName }}</span>
            </div>
            <button class="workers-btn" @click="showWorkersPanel = true" title="Show workers">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="2" y="7" width="20" height="10" rx="2"/>
                <path d="M6 7V5a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v2"/>
                <line x1="12" y1="12" x2="12" y2="12.01" stroke-width="2" stroke-linecap="round"/>
              </svg>
              <span v-if="workers.length > 0" class="workers-count">{{ workers.length }}</span>
            </button>
          </div>
        </div>

        <!-- 消息容器 -->
        <div ref="messagesContainer" class="messages-container" @scroll="handleScroll">
          <!-- 欢迎界面 -->
          <div v-if="!store.currentSession || store.currentSession.messages.length === 0" class="boot-screen">
            <div class="boot-content">
              <div class="boot-messages-layout">
                <!-- 左侧消息列 -->
                <div class="message-column left">
                  <div
                    v-for="(msg, i) in displayedLeftMessages"
                    :key="`left-${i}`"
                    class="boot-message-bubble left"
                    :style="{ animationDelay: `${i * 0.5}s` }"
                  >
                    <span class="msg-prefix">[SYSTEM]</span>
                    <span class="msg-text">{{ msg }}</span>
                    <span class="typing-cursor" v-if="i === displayedLeftMessages.length - 1 && isTyping"></span>
                  </div>
                </div>

                <!-- 中心BB-8机器人 -->
                <div class="boot-logo">
                  <!-- BB-8 风格动画 -->
                  <div class="bb8-container">
                    <div class="bb8-body">
                      <div class="bb8-head">
                        <div class="bb8-eye left"></div>
                        <div class="bb8-eye right"></div>
                      </div>
                      <div class="bb8-circle"></div>
                      <div class="bb8-line horizontal"></div>
                      <div class="bb8-line vertical"></div>
                    </div>
                    <!-- 思考时的声波动画 -->
                    <div class="sound-waves">
                      <span v-for="i in 3" :key="i" class="wave" :style="{ animationDelay: `${i * 0.1}s` }"></span>
                    </div>
                  </div>
                </div>

                <!-- 右侧消息列 -->
                <div class="message-column right">
                  <div
                    v-for="(msg, i) in displayedRightMessages"
                    :key="`right-${i}`"
                    class="boot-message-bubble right"
                    :style="{ animationDelay: `${i * 0.5}s` }"
                  >
                    <span class="msg-prefix">[SYSTEM]</span>
                    <span class="msg-text">{{ msg }}</span>
                    <span class="typing-cursor" v-if="i === displayedRightMessages.length - 1 && isTyping"></span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 消息列表 -->
          <div v-else class="messages-list">
            <div
              v-for="(message, index) in displayMessages"
              :key="index"
              class="console-message"
              :class="message.role"
            >
              <!-- 消息时间戳 -->
              <div class="message-timestamp">
                <span class="timestamp">[{{ formatTimestamp(message.timestamp) }}]</span>
              </div>

              <div class="message-row">
                <!-- 用户消息 -->
                <template v-if="message.role === 'user'">
                  <div class="user-avatar">
                    <div class="avatar-container">
                      <div class="avatar-glow"></div>
                      <span class="avatar-text">YOU</span>
                    </div>
                  </div>
                  <div class="message-content user-message">
                    <div class="message-body">
                      <div class="markdown-body" v-html="renderMessage(message.content)"></div>
                    </div>
                  </div>
                </template>

                <!-- AI 消息 - R2-D2 风格 -->
                <template v-else>
                  <div class="ai-avatar">
                    <div
                      class="r2d2-avatar"
                      :class="{ thinking: store.isLoading && index === displayMessages.length - 1 }"
                    >
                      <div class="r2d2-body">
                        <div class="r2d2-dome"></div>
                        <div class="r2d2-sensor main"></div>
                        <div class="r2d2-sensor small one"></div>
                        <div class="r2d2-sensor small two"></div>
                        <div class="r2d2-panel"></div>
                        <div class="r2d2-arm left"></div>
                        <div class="r2d2-arm right"></div>
                      </div>
                      <!-- 思考时的声波动画 -->
                      <div v-if="store.isLoading && index === displayMessages.length - 1" class="sound-waves">
                        <span v-for="i in 5" :key="i" class="wave" :style="{ animationDelay: `${i * 0.1}s` }"></span>
                      </div>
                    </div>
                  </div>
                  <div class="message-content ai-message">
                    <div class="message-meta">
                      <span class="model-badge">{{ currentModelName }}</span>
                    </div>
                    <div class="message-body">
                      <div class="markdown-body" v-html="renderMessage(message.content)"></div>
                    </div>
                    <div v-if="message.metadata?.trace?.length" class="agent-activity">
                      <div class="activity-header">
                        <span>AGENT RUN</span>
                        <button class="activity-toggle" @click="agentStore.tracePanelOpen = true">TRACE</button>
                      </div>
                      <div v-if="message.metadata?.run" class="activity-summary">
                        <div class="activity-chip">
                          <span class="chip-label">PROVIDER</span>
                          <span class="chip-value">{{ message.metadata.run.provider }}</span>
                        </div>
                        <div class="activity-chip">
                          <span class="chip-label">MODEL</span>
                          <span class="chip-value">{{ message.metadata.run.model }}</span>
                        </div>
                        <div class="activity-chip">
                          <span class="chip-label">TOOLS</span>
                          <span class="chip-value">{{ message.metadata.run.toolCount }}</span>
                        </div>
                        <div class="activity-chip">
                          <span class="chip-label">DURATION</span>
                          <span class="chip-value">{{ formatDuration(message.metadata.run.totalDurationMs) }}</span>
                        </div>
                        <div class="activity-chip">
                          <span class="chip-label">FINISH</span>
                          <span class="chip-value">{{ message.metadata.run.finishReason || 'pending' }}</span>
                        </div>
                      </div>
                      <div
                        v-for="highlight in getRunHighlights(message.metadata.trace)"
                        :key="highlight.id"
                        class="activity-highlight"
                        :class="highlight.status"
                      >
                        <span class="highlight-icon">{{ getHighlightIcon(highlight.kind, highlight.status) }}</span>
                        <span class="highlight-label">{{ highlight.label }}</span>
                        <span class="highlight-value">{{ highlight.value }}</span>
                      </div>
                    </div>
                    <div class="message-actions">
                      <button class="action-btn" @click="handleCopyMessage(message.content)" title="Copy">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <rect x="9" y="9" width="13" height="13" rx="2"/>
                          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                        </svg>
                      </button>
                      <button
                        v-if="index === displayMessages.length - 1"
                        class="action-btn"
                        @click="handleRegenerate"
                        title="Regenerate"
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <path d="M23 4v6h-6M1 20v-6h6"/>
                          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
                        </svg>
                      </button>
                    </div>
                  </div>
                </template>
              </div>
            </div>

            <!-- 流式输出指示器 -->
            <div v-if="store.isLoading" class="streaming-indicator">
              <div class="streaming-cursor"></div>
              <span class="streaming-text">PROCESSING</span>
              <span class="streaming-dots">
                <span class="dot"></span>
                <span class="dot"></span>
                <span class="dot"></span>
              </span>
            </div>
          </div>
        </div>

        <!-- 输入区域 -->
        <div class="input-console" :class="inputConsoleState">
          <!-- 统一胶囊容器 -->
          <div class="input-bar">
            <!-- 左侧工具栏 -->
            <button class="toolbar-btn" @click="showCommands = !showCommands" title="Commands (/)">
              <span>/</span>
            </button>
            <button class="toolbar-btn" @click="showAttachmentMenu = !showAttachmentMenu" title="Add">
              <span>+</span>
            </button>

            <!-- 输入框（自动撑满） -->
            <textarea
              ref="inputRef"
              v-model="inputText"
              class="chat-input"
              :placeholder="$t('chat.commandPlaceholder')"
              rows="1"
              @keydown="handleKeydown"
              @input="autoResize"
            ></textarea>

            <!-- Model选择器 -->
            <div class="model-selector">
              <button class="model-dropdown-btn" @click="showModelDropdown = !showModelDropdown">
                <span class="model-dropdown-name">{{ currentModelName }}</span>
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                  <polyline points="6 9 12 15 18 9" stroke="currentColor" stroke-width="2"/>
                </svg>
              </button>
              <div v-if="showModelDropdown" class="model-dropdown-menu">
                <div
                  v-for="model in selectedProviderModels"
                  :key="model.id"
                  class="model-dropdown-item"
                  :class="{ selected: model.id === modelsDevStore.selectedModelId }"
                  @click="selectModelFromDropdown(model)"
                >
                  <span class="model-item-name">{{ model.name }}</span>
                  <span class="model-item-context">{{ formatContextShort(model.limit?.context) }}</span>
                </div>
              </div>
            </div>

            <!-- 发送按钮 -->
            <button
              class="send-btn"
              :class="{ disabled: !inputText.trim() || store.isLoading }"
              @click="handleSend"
              :disabled="!inputText.trim() || store.isLoading"
            >
              <span class="send-icon">↑</span>
            </button>
          </div>

          <!-- 浮层（独立于胶囊容器） -->
          <CommandPalette
            :visible="showCommands"
            @select="handleCommandSelect"
            @close="showCommands = false"
          />
          <AttachmentMenu
            v-if="showAttachmentMenu"
            @close="showAttachmentMenu = false"
            @select="handleAttachmentSelect"
          />
        </div>
      </div>

      <!-- 右侧：会话历史 -->
      <div class="history-panel" :class="{ collapsed: isHistoryCollapsed }">
        <div class="panel-header">
          <span class="panel-title">// SESSION LOG</span>
          <button class="toggle-btn" @click="isHistoryCollapsed = !isHistoryCollapsed" title="Toggle session panel">
            <svg v-if="isHistoryCollapsed" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="15 18 9 12 15 6"></polyline>
            </svg>
            <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
          </button>
          <button class="new-session-btn" @click="handleNewSession">
            <span class="btn-icon">+</span>
            <span class="btn-text" v-if="!isHistoryCollapsed">NEW</span>
          </button>
        </div>
        <div class="session-list" v-if="!isHistoryCollapsed">
          <div
            v-for="session in store.sessions"
            :key="session.id"
            class="session-item"
            :class="{ active: session.id === store.currentSessionId }"
            @click="handleSelectSession(session.id)"
          >
            <div class="session-dot"></div>
            <div class="session-info">
              <div class="session-title">{{ session.title }}</div>
              <div class="session-time">{{ formatSessionTime(session.updated_at) }}</div>
            </div>
            <div class="session-menu" @click.stop>
              <button class="menu-btn" @click="showSessionMenu(session)">⋮</button>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧：Agent Trace 面板 -->
      <AgentTrace v-if="agentStore.tracePanelOpen" />
    </div>

    <!-- Model Settings Dialog -->
    <ModelSettings v-model="showModelSettings" />

    <!-- Agent Pool Panel -->
    <AgentPoolPanel v-model:showPanel="showWorkersPanel" />
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { formatCommandPrompt, type SLASH_COMMANDS } from '@/agent/slashCommands'
import { setOnWorkerCompleteCallback, useAgentPool } from '@/agent/useAgentPool'
import AttachmentMenu from '@/components/AttachmentMenu.vue'
import AgentPoolPanel from '@/components/agent/AgentPoolPanel.vue'
import CommandPalette from '@/components/CommandPalette.vue'
import { useAgentStore } from '@/stores/agent'
import { type AgentTraceDetail, useAiChatStore } from '@/stores/aiChat'
import { useModelsDevStore } from '@/stores/modelsDev'
import { useProviderConfigStore } from '@/stores/providerConfig'
import { useSkillsStore } from '@/stores/skills'
import { renderSafe, sharedMd } from '@/utils/markdown'
import AgentTrace from './AgentTrace.vue'
import ModelSettings from './ModelSettings.vue'

const store = useAiChatStore()
const skillsStore = useSkillsStore()
const agentStore = useAgentStore()
const modelsDevStore = useModelsDevStore()
const providerConfigStore = useProviderConfigStore()

// UI State
const inputText = ref('')
const messagesContainer = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLTextAreaElement | null>(null)
const showCommands = ref(false)
const showScrollButton = ref(false)
const showModelSettings = ref(false)
const showAttachmentMenu = ref(false)
const showModelDropdown = ref(false)
const isHistoryCollapsed = ref(true)
const showWorkersPanel = ref(false)

// Agent Pool
const { workers } = useAgentPool()

// Boot state animation
const displayedLeftMessages = ref<string[]>([])
const displayedRightMessages = ref<string[]>([])
const isTyping = ref(false)
const currentMessageIndex = ref(0)
const currentCharIndex = ref(0)
const typingSpeed = 50 // ms per character
const messagePause = 2000 // ms pause after message completes

// Boot sequence messages
const bootSequence = [
  'Initializing Holocron Interface...',
  'Loading R2-D2 Neural Core...',
  'Connecting to Jedi Archives...',
  'Calibrating Lightsaber Matrix...',
  'System online. Awaiting input.',
  'May the Force be with you.',
  'What can I help you build today?',
  'Need assistance with code debugging?',
  'I can help optimize your workflow.',
]

// Computed
const selectedProviderModels = computed(() => {
  return modelsDevStore.selectedProviderModels
})

const currentModelName = computed(() => {
  return modelsDevStore.selectedModel?.name || 'SELECT MODEL'
})

function selectModelFromDropdown(model: any) {
  modelsDevStore.selectModel(model.id)
  showModelDropdown.value = false
}

function handleAttachmentSelect(_action: string) {
  showAttachmentMenu.value = false
  // Handle: attachment, skills, web-search - can be implemented later
}

function formatContextShort(len?: number): string {
  if (!len) return 'N/A'
  if (len >= 1000000) return `${(len / 1000000).toFixed(0)}M`
  if (len >= 1000) return `${(len / 1000).toFixed(0)}K`
  return len.toString()
}

// Computed
const currentProviderName = computed(() => {
  return modelsDevStore.selectedProvider?.name?.toUpperCase() || 'SELECT PROVIDER'
})
const connectionStatus = computed(() => {
  if (modelsDevStore.selectedModel) return 'CONNECTED'
  if (modelsDevStore.allProviders.length === 0) return 'OFFLINE'
  return 'NO MODEL'
})

const displayMessages = computed(() => {
  return store.currentSession?.messages || []
})

const inputConsoleState = computed(() => {
  return displayMessages.value.length > 0 ? 'state-chatting' : 'state-new-session'
})

function renderMessage(content: string) {
  return renderSafe(sharedMd, content)
}

function formatDuration(ms?: number): string {
  if (ms === undefined) return '--'
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`
  return `${(ms / 60000).toFixed(2)}m`
}

type RunHighlight = {
  id: string
  kind: 'plan' | 'tool' | 'finish' | 'error'
  label: string
  value: string
  status: AgentTraceDetail['status']
}

function getRunHighlights(trace: AgentTraceDetail[] = []): RunHighlight[] {
  const highlights: RunHighlight[] = []

  const planTrace = trace.find(item => item.type === 'think')
  if (planTrace?.output || planTrace?.content) {
    highlights.push({
      id: `${planTrace.id}-plan`,
      kind: 'plan',
      label: 'PLAN',
      value: summarizeText(String(planTrace.output || planTrace.content || ''), 120),
      status: planTrace.status,
    })
  }

  const toolNames = [
    ...new Set(
      trace.filter(item => item.type === 'tool' && item.toolName).map(item => item.toolName!)
    ),
  ]
  if (toolNames.length > 0) {
    highlights.push({
      id: 'tools-highlight',
      kind: 'tool',
      label: toolNames.length > 1 ? 'TOOLS' : 'TOOL',
      value: toolNames.join(', '),
      status: 'done',
    })
  }

  const finishTrace = [...trace]
    .reverse()
    .find(item => item.type === 'finish' || item.type === 'error')
  if (finishTrace) {
    highlights.push({
      id: `${finishTrace.id}-finish`,
      kind: finishTrace.type === 'error' ? 'error' : 'finish',
      label: finishTrace.type === 'error' ? 'ERROR' : 'RESULT',
      value: summarizeText(String(finishTrace.content || finishTrace.output || ''), 120),
      status: finishTrace.status,
    })
  }

  return highlights
}

function getHighlightIcon(kind: RunHighlight['kind'], status: AgentTraceDetail['status']): string {
  if (status === 'error') return '!'
  if (kind === 'tool') return '$'
  if (kind === 'finish') return '+'
  return '~'
}

function summarizeText(text: string, maxLength = 120): string {
  const normalized = text.replace(/\s+/g, ' ').trim()
  return normalized.length > maxLength ? `${normalized.slice(0, maxLength)}...` : normalized
}

// Formatting
function formatTimestamp(timestamp?: number) {
  if (!timestamp) return '00:00:00'
  const date = new Date(timestamp)
  return date.toLocaleTimeString('en-US', { hour12: false })
}

function formatSessionTime(dateStr: string) {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  if (diff < 60000) return 'JUST NOW'
  if (diff < 3600000) return `${Math.floor(diff / 60000)} MIN AGO`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} HRS AGO`
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' }).toUpperCase()
}

function handleSelectSession(sessionId: string) {
  store.currentSessionId = sessionId
  nextTick(() => {
    scrollToBottom()
  })
}

function showSessionMenu(session: any) {
  console.log('Session menu:', session)
}

// Actions
function handleCommandSelect(cmd: (typeof SLASH_COMMANDS)[0]) {
  if (!store.currentSession) {
    store.createSession(
      '新对话',
      modelsDevStore.selectedProviderId || 'openai',
      modelsDevStore.selectedModelId || 'gpt-4o-mini'
    )
  }
  inputText.value = `${cmd.name} `
  nextTick(() => {
    inputRef.value?.focus()
  })
}

function handleNewSession() {
  store.createSession(
    '新对话',
    modelsDevStore.selectedProviderId || 'openai',
    modelsDevStore.selectedModelId || 'gpt-4o-mini'
  )
  nextTick(() => {
    inputRef.value?.focus()
  })
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    handleSend()
  }
}

async function handleSend() {
  if (!inputText.value.trim() || store.isLoading) return

  const content = inputText.value.trim()
  const prompt = formatCommandPrompt(content)

  if (content.startsWith('/agent')) {
    const desc = content.slice(6).trim() || 'Worker task'
    inputText.value = ''
    autoResize()
    try {
      await agentStore.runWithPool(prompt, desc)
      scrollToBottom()
    } catch (e) {
      console.error('Failed to run with pool:', e)
    }
  } else {
    inputText.value = ''
    autoResize()
    // Create session with selected provider/model if needed
    if (!store.currentSession) {
      await store.createSession(
        '新对话',
        modelsDevStore.selectedProviderId || 'openai',
        modelsDevStore.selectedModelId || 'gpt-4o-mini'
      )
    }
    try {
      await store.sendMessage(content)
      scrollToBottom()
    } catch (e) {
      console.error('Failed to send message:', e)
    }
  }
}

function handleCopyMessage(content: string) {
  navigator.clipboard.writeText(content)
}

function handleRegenerate() {
  if (!store.currentSession || store.currentSession.messages.length < 2) return

  const lastUserMessage = [...store.currentSession.messages].reverse().find(m => m.role === 'user')

  if (lastUserMessage) {
    store.currentSession.messages.pop()
    store.sendMessage(lastUserMessage.content)
  }
}

// Scroll handling
function handleScroll() {
  if (!messagesContainer.value) return
  const { scrollTop, scrollHeight, clientHeight } = messagesContainer.value
  showScrollButton.value = scrollHeight - scrollTop - clientHeight > 200
}

function scrollToBottom() {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

function autoResize() {
  nextTick(() => {
    if (inputRef.value) {
      inputRef.value.style.height = 'auto'
      inputRef.value.style.height = `${Math.min(inputRef.value.scrollHeight, 200)}px`
    }
  })
}

// Watchers
watch(
  () => store.currentSession?.messages.length,
  () => {
    scrollToBottom()
  }
)

watch(
  () => store.streamingContent,
  () => {
    scrollToBottom()
  }
)

// Boot typing animation
let typingInterval: number | null = null

function startTypingAnimation() {
  // Reset state
  displayedLeftMessages.value = []
  displayedRightMessages.value = []
  currentMessageIndex.value = 0
  currentCharIndex.value = 0

  function typeNextCharacter() {
    const currentMessage = bootSequence[currentMessageIndex.value]
    const isLeftSide = currentMessageIndex.value % 2 === 0 // Even index on left, odd on right
    const targetArray = isLeftSide ? displayedLeftMessages : displayedRightMessages

    // Start new message if needed
    if (currentCharIndex.value === 0) {
      isTyping.value = true
      // 最多保留3条消息，多了就删掉最旧的
      if (targetArray.value.length >= 3) {
        targetArray.value.shift()
      }
      targetArray.value.push('')
    }

    // Add next character
    if (currentCharIndex.value < currentMessage.length) {
      targetArray.value[targetArray.value.length - 1] = currentMessage.slice(
        0,
        currentCharIndex.value + 1
      )
      currentCharIndex.value++
      typingInterval = window.setTimeout(typeNextCharacter, typingSpeed)
    } else {
      // Message complete
      isTyping.value = false
      currentCharIndex.value = 0
      currentMessageIndex.value = (currentMessageIndex.value + 1) % bootSequence.length

      // Pause before next message
      typingInterval = window.setTimeout(typeNextCharacter, messagePause)
    }
  }

  typeNextCharacter()
}

function stopTypingAnimation() {
  if (typingInterval) {
    clearTimeout(typingInterval)
    typingInterval = null
  }
}

onMounted(async () => {
  skillsStore.loadFromStorage()
  await Promise.all([
    modelsDevStore.fetchProviders(),
    providerConfigStore.loadConfiguredProviders(),
  ])
  scrollToBottom()

  // Start boot animation
  startTypingAnimation()

  // Set up worker completion callback
  setOnWorkerCompleteCallback(worker => {
    const status = worker.status === 'completed' ? 'completed' : 'failed'
    const result = worker.result || worker.error || 'Finished'
    store.addMessage({
      role: 'system',
      content: `Worker "${worker.description}" ${status}: ${result}`,
    })
  })
})

onUnmounted(() => {
  stopTypingAnimation()
})
</script>

<style src="./chat.css" scoped></style>
