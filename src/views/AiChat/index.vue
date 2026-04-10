<template>
  <div class="chat-console-page">
    <!-- 扫描线覆盖层 -->
    <div class="scanlines"></div>
    <!-- CRT 晕影 -->
    <div class="crt-vignette"></div>

    <div class="chat-console-layout">
      <!-- 左侧：Skills 面板 -->
      <SkillPanel />

      <!-- 主聊天区域 -->
      <div class="chat-console-area">
        <!-- 顶部状态栏 -->
        <div class="console-header">
          <div class="header-left">
            <div class="terminal-prompt">
              <span class="prompt-user">jedi</span>
              <span class="prompt-separator">@</span>
              <span class="prompt-host">holocron</span>
              <span class="prompt-path">~/chat</span>
              <span class="prompt-cursor">▶</span>
            </div>
          </div>
          <div class="header-right">
            <div class="status-badge">
              <span class="status-dot"></span>
              <span class="status-text">{{ connectionStatus }}</span>
            </div>
            <div class="model-display" @click="showModelSettings = true">
              <span class="model-label">MODEL:</span>
              <span class="model-name">{{ currentModelName }}</span>
            </div>
          </div>
        </div>

        <!-- 消息容器 -->
        <div ref="messagesContainer" class="messages-container" @scroll="handleScroll">
          <!-- 欢迎界面 -->
          <div v-if="!store.currentSession || store.currentSession.messages.length === 0" class="boot-screen">
            <div class="boot-content">
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
                </div>
              </div>
              <div class="boot-text">
                <div class="boot-line" v-for="(line, i) in bootSequence" :key="i">
                  <span class="boot-prefix">[SYSTEM]</span>
                  <span class="boot-content">{{ line }}</span>
                </div>
              </div>
              <div class="quick-commands">
                <div
                  v-for="cmd in quickCommands"
                  :key="cmd.text"
                  class="command-card"
                  @click="executeCommand(cmd)"
                >
                  <div class="command-icon">{{ cmd.icon }}</div>
                  <div class="command-text">
                    <div class="command-title">{{ cmd.title }}</div>
                    <div class="command-desc">{{ cmd.desc }}</div>
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
                    <div class="message-header">
                      <span class="message-role">&lt;USER_INPUT&gt;</span>
                    </div>
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
                    <div class="message-header">
                      <span class="message-role">&lt;R2D2_OUTPUT&gt;</span>
                      <span class="message-model">[{{ currentModelName }}]</span>
                    </div>
                    <div class="message-body">
                      <div class="markdown-body" v-html="renderMessage(message.content)"></div>
                    </div>
                    <div class="message-actions">
                      <button class="action-btn" @click="handleCopyMessage(message.content)">
                        <span class="action-icon">⧉</span>
                        <span class="action-label">COPY</span>
                      </button>
                      <button
                        v-if="index === displayMessages.length - 1"
                        class="action-btn"
                        @click="handleRegenerate"
                      >
                        <span class="action-icon">↻</span>
                        <span class="action-label">REGEN</span>
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

        <!-- 输入区域 - 终端风格 -->
        <div class="input-console">
          <div class="input-row">
            <div class="input-prompt">
              <span class="prompt-user">jedi</span>
              <span class="prompt-separator">:</span>
              <span class="prompt-path">~</span>
              <span class="prompt-cursor">█</span>
            </div>
            <div class="input-wrapper">
              <textarea
                ref="inputRef"
                v-model="inputText"
                class="console-input"
                :placeholder="$t('chat.commandPlaceholder')"
                rows="1"
                @keydown="handleKeydown"
                @input="autoResize"
              ></textarea>
            </div>
            <div class="input-actions">
              <button
                class="send-btn"
                :class="{ disabled: !inputText.trim() || store.isLoading }"
                @click="handleSend"
                :disabled="!inputText.trim() || store.isLoading"
              >
                <span class="send-icon">⚡</span>
                <span class="send-label">TRANSMIT</span>
              </button>
              <button
                v-if="store.isLoading"
                class="stop-btn"
                @click="handleStop"
              >
                <span class="stop-icon">⬛</span>
                <span class="stop-label">ABORT</span>
              </button>
            </div>
          </div>
          <div class="input-footer">
            <span class="footer-hint">Press ENTER to send, SHIFT+ENTER for new line</span>
            <span class="footer-shortcuts">
              <span class="shortcut">↑↓ History</span>
              <span class="shortcut">Ctrl+C Stop</span>
            </span>
          </div>
        </div>
      </div>

      <!-- 右侧：会话历史 -->
      <div class="history-panel">
        <div class="panel-header">
          <span class="panel-title">// SESSION LOG</span>
          <button class="new-session-btn" @click="handleNewSession">
            <span class="btn-icon">+</span>
            <span class="btn-text">NEW</span>
          </button>
        </div>
        <div class="session-list">
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
    <AgentPoolPanel />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted, watch } from 'vue'
import { useAiChatStore } from '@/stores/aiChat'
import { useSkillsStore } from '@/stores/skills'
import { useAgentStore } from '@/stores/agent'
import { useModelsDevStore } from '@/stores/modelsDev'
import { setOnWorkerCompleteCallback } from '@/agent/useAgentPool'
import { SLASH_COMMANDS, formatCommandPrompt } from '@/agent/slashCommands'
import { sharedMd, renderSafe } from '@/utils/markdown'
import SkillPanel from './SkillPanel.vue'
import AgentTrace from './AgentTrace.vue'
import ModelSettings from './ModelSettings.vue'
import AgentPoolPanel from '@/components/agent/AgentPoolPanel.vue'

const store = useAiChatStore()
const skillsStore = useSkillsStore()
const agentStore = useAgentStore()
const modelsDevStore = useModelsDevStore()

// UI State
const inputText = ref('')
const messagesContainer = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLTextAreaElement | null>(null)
const showScrollButton = ref(false)
const showModelSettings = ref(false)

// Boot sequence
const bootSequence = [
  'Initializing Holocron Interface...',
  'Loading R2-D2 Neural Core...',
  'Connecting to Jedi Archives...',
  'Calibrating Lightsaber Matrix...',
  'System online. Awaiting input.'
]

// Quick commands
const quickCommands = computed(() =>
  SLASH_COMMANDS.map(cmd => ({
    icon: cmd.icon,
    title: cmd.name,
    desc: cmd.description,
    text: cmd.name + ' '
  }))
)

// Computed
const currentModelName = computed(() => {
  return modelsDevStore.selectedModel?.name || 'SELECT MODEL'
})
const connectionStatus = computed(() => modelsDevStore.allProviders.length > 0 ? 'CONNECTED' : 'OFFLINE')

const displayMessages = computed(() => {
  return store.currentSession?.messages || []
})

function renderMessage(content: string) {
  return renderSafe(sharedMd, content)
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
function executeCommand(cmd: typeof quickCommands.value[0]) {
  if (!store.currentSession) {
    store.createSession()
  }
  inputText.value = cmd.text
  nextTick(() => {
    inputRef.value?.focus()
  })
}

function handleNewSession() {
  store.createSession()
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
    try {
      await store.sendMessage(content)
      scrollToBottom()
    } catch (e) {
      console.error('Failed to send message:', e)
    }
  }
}

function handleStop() {
  console.log('Stop generation')
}

function handleCopyMessage(content: string) {
  navigator.clipboard.writeText(content)
}

function handleRegenerate() {
  if (!store.currentSession || store.currentSession.messages.length < 2) return

  const lastUserMessage = [...store.currentSession.messages]
    .reverse()
    .find(m => m.role === 'user')

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
      inputRef.value.style.height = Math.min(inputRef.value.scrollHeight, 200) + 'px'
    }
  })
}

// Watchers
watch(() => store.currentSession?.messages.length, () => {
  scrollToBottom()
})

watch(() => store.streamingContent, () => {
  scrollToBottom()
})

onMounted(async () => {
  skillsStore.loadFromStorage()
  await modelsDevStore.fetchProviders()
  scrollToBottom()

  // Set up worker completion callback
  setOnWorkerCompleteCallback((worker) => {
    const status = worker.status === 'completed' ? 'completed' : 'failed'
    const result = worker.result || worker.error || 'Finished'
    store.addMessage({
      role: 'system',
      content: `Worker "${worker.description}" ${status}: ${result}`
    })
  })
})
</script>

<style src="./chat.css" scoped></style>
