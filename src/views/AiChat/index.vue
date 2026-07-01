<template>
  <div class="chat-console-page">
    <!-- 扫描线覆盖层 -->
    <div class="scanlines"></div>
    <!-- CRT 晕影 -->
    <div class="crt-vignette"></div>

    <div class="chat-console-layout">
      <!-- 左侧：Skills 面板 -->
      <div class="mcp-panel">
        <div class="panel-header">
          <span class="panel-title">// SKILLS</span>
          <span class="panel-status online">READY</span>
        </div>
        <div class="mcp-skills-list">
          <div
            v-for="skill in mcpSkills"
            :key="skill.id"
            class="mcp-skill-item"
            :class="{ active: activeSkill === skill.id }"
            @click="toggleSkill(skill.id)"
          >
            <div class="skill-indicator" :class="{ enabled: skill.enabled }"></div>
            <span class="skill-name">{{ skill.name }}</span>
            <span class="skill-hotkey">{{ skill.hotkey }}</span>
          </div>
        </div>
        <div class="panel-footer">
          <span class="footer-text">TYPE /SKILL FOR HELP</span>
        </div>
      </div>

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
            <div class="model-display">
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

      <!-- 右侧：会话历史 / Agent 追踪 -->
      <div class="history-panel">
        <div class="panel-header rt-header">
          <div class="rt-tabs">
            <button
              class="rt-tab"
              :class="{ active: rightTab === 'sessions' }"
              @click="rightTab = 'sessions'"
            >SESSIONS</button>
            <button
              class="rt-tab"
              :class="{ active: rightTab === 'trace' }"
              @click="rightTab = 'trace'"
            >
              TRACE
              <span v-if="store.agentTrace.length" class="rt-badge">{{ store.agentTrace.length }}</span>
            </button>
          </div>
          <button
            v-if="rightTab === 'sessions'"
            class="new-session-btn icon-only"
            title="New session"
            @click="handleNewSession"
          >
            <span class="btn-icon">+</span>
          </button>
        </div>

        <div v-show="rightTab === 'sessions'" class="session-list">
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

        <AgentTrace
          v-show="rightTab === 'trace'"
          :trace="store.agentTrace"
          :running="store.isLoading && store.enabledMcpServers.length > 0"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted, watch } from 'vue'
import { useAiChatStore } from '@/stores/aiChat'
import { sharedMd, renderSafe } from '@/utils/markdown'
import AgentTrace from './AgentTrace.vue'

const store = useAiChatStore()

// UI State
const inputText = ref('')
const messagesContainer = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLTextAreaElement | null>(null)
const showScrollButton = ref(false)
const activeSkill = ref<string | null>(null)
const rightTab = ref<'sessions' | 'trace'>('sessions')

// 有后端 MCP 服务支撑的技能 id（可真正驱动 Agent 工具调用）
const mcpBackedIds = computed(() => new Set(store.mcpServers.map(s => s.id)))

// Skills - Teach procedures that can be invoked in chat
// These are like "recipes" that Claude can follow when activated
const mcpSkills = ref([
  { id: 'hosts', name: 'HOSTS_MGR', enabled: false, hotkey: 'F1', desc: 'Manage hosts file' },
  { id: 'wallpaper', name: 'WALLPAPER', enabled: false, hotkey: 'F2', desc: 'Knowledge wallpapers' },
  { id: 'podcast', name: 'PODCAST', enabled: false, hotkey: 'F3', desc: 'Manage podcasts' },
  { id: 'system', name: 'SYSTEM', enabled: false, hotkey: 'F4', desc: 'System info' },
  { id: 'terminal', name: 'TERMINAL', enabled: false, hotkey: 'F5', desc: 'Coming soon' },
  { id: 'browser', name: 'BROWSER', enabled: false, hotkey: 'F6', desc: 'Coming soon' },
])

// Boot sequence
const bootSequence = [
  'Initializing Holocron Interface...',
  'Loading R2-D2 Neural Core...',
  'Connecting to Jedi Archives...',
  'Calibrating Lightsaber Matrix...',
  'System online. Awaiting input.'
]

// Quick commands
const quickCommands = ref([
  { icon: '🧠', title: 'Explain Concept', desc: 'Teach me something new', text: 'Please explain this concept in detail: ' },
  { icon: '⚡', title: 'Write Code', desc: 'Generate implementation', text: 'Help me write code for: ' },
  { icon: '📝', title: 'Summarize', desc: 'Condense information', text: 'Summarize the following content:\n\n' },
  { icon: '💡', title: 'Brainstorm', desc: 'Generate ideas', text: 'Help me brainstorm ideas for: ' },
])

// Computed
const currentModelName = computed(() => {
  const model = store.availableModels.find(m => m.id === store.selectedModelId)
  return model?.name || 'UNKNOWN'
})
const connectionStatus = computed(() => store.availableModels.length > 0 ? 'CONNECTED' : 'OFFLINE')

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
function toggleSkill(skillId: string) {
  const skill = mcpSkills.value.find(s => s.id === skillId)
  if (!skill) return

  // 若该技能背后有真实的 MCP 服务，则同步到 store（驱动 Agent 工具调用）
  if (mcpBackedIds.value.has(skillId)) {
    store.toggleMcpServer(skillId)
    skill.enabled = store.enabledMcpServers.includes(skillId)
    store.saveSettings()
  } else {
    skill.enabled = !skill.enabled
  }
  activeSkill.value = skill.enabled ? skillId : null
}

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
  inputText.value = ''
  autoResize()

  try {
    await store.sendMessage(content)
    scrollToBottom()
  } catch (e) {
    console.error('Failed to send message:', e)
  }
}

async function handleStop() {
  try {
    await store.cancelAgent()
  } catch (e) {
    console.error('Failed to cancel agent:', e)
  }
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

// Agent 有新动作时自动切到 TRACE 标签
watch(() => store.agentTrace.length, (len) => {
  if (len > 0) rightTab.value = 'trace'
})

onMounted(() => {
  // 用 store 中真实的 MCP 启用状态同步左侧技能面板
  mcpSkills.value.forEach(skill => {
    if (mcpBackedIds.value.has(skill.id)) {
      skill.enabled = store.enabledMcpServers.includes(skill.id)
    }
  })
  scrollToBottom()
})
</script>

<style src="./chat.css" scoped></style>

<style scoped>
/* 右侧面板标签栏（SESSIONS / TRACE） */
.rt-header {
  gap: 8px;
}

.rt-tabs {
  display: flex;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.rt-tab {
  flex: 1;
  padding: 5px 6px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: #52525b;
  font-family: inherit;
  font-size: 9.5px;
  font-weight: 700;
  letter-spacing: 1px;
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
}

.rt-tab:hover {
  color: #a1a1aa;
  background: rgba(96, 165, 250, 0.05);
}

.rt-tab.active {
  color: #60a5fa;
  background: rgba(96, 165, 250, 0.1);
  border-color: rgba(96, 165, 250, 0.25);
  text-shadow: 0 0 8px rgba(96, 165, 250, 0.4);
}

.rt-badge {
  font-size: 8.5px;
  min-width: 15px;
  padding: 1px 4px;
  border-radius: 8px;
  background: rgba(34, 211, 238, 0.15);
  color: #22d3ee;
  font-weight: 700;
}

.new-session-btn.icon-only {
  flex-shrink: 0;
  padding: 4px 8px;
}

/* 浅色主题 */
:global(.light-theme) .rt-tab { color: #9c7a4d; }
:global(.light-theme) .rt-tab.active {
  color: #b8860b;
  background: rgba(184, 134, 11, 0.12);
  border-color: rgba(184, 134, 11, 0.3);
}
</style>
