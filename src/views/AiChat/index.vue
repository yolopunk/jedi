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
                :placeholder="'Enter your command...'"
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted, watch } from 'vue'
import { useAiChatStore } from '@/stores/aiChat'
import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'
import DOMPurify from 'dompurify'

const store = useAiChatStore()

// UI State
const inputText = ref('')
const messagesContainer = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLTextAreaElement | null>(null)
const showScrollButton = ref(false)
const activeSkill = ref<string | null>(null)

// Skills - Teach procedures that can be invoked in chat
// These are like "recipes" that Claude can follow when activated
const mcpSkills = ref([
  { id: 'terminal', name: 'TERMINAL', enabled: true, hotkey: 'F1', desc: 'Execute system commands' },
  { id: 'filesystem', name: 'FILE_SYS', enabled: true, hotkey: 'F2', desc: 'Read/write files' },
  { id: 'hosts', name: 'HOSTS_MGR', enabled: true, hotkey: 'F3', desc: 'Manage hosts file' },
  { id: 'podcast', name: 'PODCAST', enabled: true, hotkey: 'F4', desc: 'Manage podcasts' },
  { id: 'wallpaper', name: 'WALLPAPER', enabled: false, hotkey: 'F5', desc: 'Change wallpapers' },
  { id: 'browser', name: 'BROWSER', enabled: false, hotkey: 'F6', desc: 'Web browsing' },
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

// Markdown renderer
const md = MarkdownIt({
  highlight: (str, lang) => {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(str, { language: lang }).value
      } catch (e) {}
    }
    return ''
  },
  linkify: true,
  breaks: true,
})

function renderMessage(content: string) {
  if (!content) return ''
  const html = md.render(content)
  return DOMPurify.sanitize(html)
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
  if (skill) {
    skill.enabled = !skill.enabled
    activeSkill.value = skill.enabled ? skillId : null
  }
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

onMounted(() => {
  scrollToBottom()
})
</script>

<style scoped>
/* 主容器 */
.chat-console-page {
  height: 100%;
  width: 100%;
  display: flex;
  overflow: hidden;
  background: #0a0a0f;
  position: relative;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

/* 扫描线效果 */
.scanlines {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  z-index: 1000;
  background: repeating-linear-gradient(
    0deg,
    rgba(0, 0, 0, 0.15),
    rgba(0, 0, 0, 0.15) 1px,
    transparent 1px,
    transparent 2px
  );
}

/* CRT 晕影 */
.crt-vignette {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  z-index: 999;
  background: radial-gradient(ellipse at center, transparent 0%, rgba(0,0,0,0.4) 100%);
}

/* 主布局 */
.chat-console-layout {
  display: flex;
  width: 100%;
  height: 100%;
  position: relative;
  z-index: 1;
}

/* MCP 技能面板 */
.mcp-panel {
  width: 200px;
  background: linear-gradient(180deg, #0d0d12 0%, #0a0a0f 100%);
  border-right: 1px solid #1a1a24;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.panel-header {
  padding: 16px 12px;
  border-bottom: 1px solid #1a1a24;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.panel-title {
  font-size: 10px;
  letter-spacing: 2px;
  color: #60a5fa;
  text-transform: uppercase;
  font-weight: 700;
  text-shadow: 0 0 10px rgba(96, 165, 250, 0.5);
}

.panel-status {
  font-size: 9px;
  padding: 2px 6px;
  border-radius: 2px;
  font-weight: 700;
  letter-spacing: 1px;
}

.panel-status.online {
  color: #4ade80;
  background: rgba(74, 222, 128, 0.1);
  box-shadow: 0 0 8px rgba(74, 222, 128, 0.3);
}

.mcp-skills-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.mcp-skill-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  cursor: pointer;
  transition: all 0.15s ease;
  border-left: 2px solid transparent;
}

.mcp-skill-item:hover {
  background: rgba(96, 165, 250, 0.05);
  border-left-color: rgba(96, 165, 250, 0.3);
}

.mcp-skill-item.active {
  background: rgba(96, 165, 250, 0.1);
  border-left-color: #60a5fa;
}

.skill-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #3f3f46;
  transition: all 0.2s ease;
}

.skill-indicator.enabled {
  background: #4ade80;
  box-shadow: 0 0 10px rgba(74, 222, 128, 0.6);
}

.skill-name {
  flex: 1;
  font-size: 11px;
  color: #a1a1aa;
  letter-spacing: 0.5px;
}

.mcp-skill-item:hover .skill-name {
  color: #e4e4e7;
}

.skill-hotkey {
  font-size: 9px;
  color: #52525b;
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 3px;
  border: 1px solid #27272a;
}

.panel-footer {
  padding: 12px;
  border-top: 1px solid #1a1a24;
}

.footer-text {
  font-size: 9px;
  color: #60a5fa;
  letter-spacing: 2px;
  text-transform: uppercase;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* 主聊天区域 */
.chat-console-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: linear-gradient(135deg, #0a0a0f 0%, #0d0d12 50%, #0a0a0f 100%);
  position: relative;
}

/* 网格背景 */
.chat-console-area::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image:
    linear-gradient(rgba(96, 165, 250, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(96, 165, 250, 0.03) 1px, transparent 1px);
  background-size: 50px 50px;
  pointer-events: none;
  z-index: 0;
}

/* 控制台头部 */
.console-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: linear-gradient(180deg, rgba(13, 13, 18, 0.95) 0%, rgba(10, 10, 15, 0.9) 100%);
  border-bottom: 1px solid #1a1a24;
  position: relative;
  z-index: 2;
}

.header-left, .header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.terminal-prompt {
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.prompt-user { color: #60a5fa; font-weight: 700; }
.prompt-separator { color: #52525b; }
.prompt-host { color: #a855f7; font-weight: 700; }
.prompt-path { color: #71717a; }
.prompt-cursor { color: #4ade80; animation: blink 1s step-end infinite; }

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: rgba(74, 222, 128, 0.1);
  border: 1px solid rgba(74, 222, 128, 0.2);
  border-radius: 4px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #4ade80;
  box-shadow: 0 0 8px rgba(74, 222, 128, 0.6);
}

.status-text {
  font-size: 10px;
  color: #4ade80;
  font-weight: 700;
  letter-spacing: 1px;
}

.model-display {
  display: flex;
  align-items: center;
  gap: 6px;
}

.model-label {
  font-size: 9px;
  color: #52525b;
  letter-spacing: 1px;
  text-transform: uppercase;
}

.model-name {
  font-size: 11px;
  color: #60a5fa;
  font-weight: 700;
  padding: 3px 8px;
  background: rgba(96, 165, 250, 0.1);
  border: 1px solid rgba(96, 165, 250, 0.2);
  border-radius: 3px;
}

/* 消息容器 */
.messages-container {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
  z-index: 1;
  scroll-behavior: smooth;
}

/* 启动界面 */
.boot-screen {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
}

.boot-content {
  max-width: 600px;
  width: 100%;
}

/* BB-8 动画 */
.boot-logo {
  display: flex;
  justify-content: center;
  margin-bottom: 40px;
}

.bb8-container {
  width: 120px;
  height: 120px;
  position: relative;
  animation: float 3s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0) rotate(0deg); }
  50% { transform: translateY(-10px) rotate(5deg); }
}

.bb8-body {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  background: linear-gradient(135deg, #f5f5f5 0%, #d4d4d4 50%, #a1a1aa 100%);
  position: relative;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.4), inset 0 -5px 20px rgba(0, 0, 0, 0.2);
  overflow: hidden;
}

.bb8-head {
  position: absolute;
  top: -15px;
  left: 50%;
  transform: translateX(-50%);
  width: 60px;
  height: 35px;
  background: linear-gradient(180deg, #f5f5f5 0%, #d4d4d4 100%);
  border-radius: 60px 60px 20px 20px;
  box-shadow: 0 5px 15px rgba(0, 0, 0, 0.3);
  animation: headRotate 4s ease-in-out infinite;
}

@keyframes headRotate {
  0%, 100% { transform: translateX(-50%) rotate(-15deg); }
  50% { transform: translateX(-50%) rotate(15deg); }
}

.bb8-eye {
  position: absolute;
  top: 12px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #1a1a24;
  box-shadow: 0 0 10px rgba(96, 165, 250, 0.5);
}

.bb8-eye.left { left: 14px; }
.bb8-eye.right { right: 14px; }

.bb8-circle {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: 4px solid #60a5fa;
  box-shadow: 0 0 20px rgba(96, 165, 250, 0.4);
}

.bb8-line {
  position: absolute;
  background: #71717a;
}

.bb8-line.horizontal {
  top: 45%;
  left: 10%;
  right: 10%;
  height: 3px;
}

.bb8-line.vertical {
  left: 48%;
  top: 10%;
  bottom: 10%;
  width: 3px;
}

.boot-text {
  margin-bottom: 32px;
}

.boot-line {
  font-size: 11px;
  color: #71717a;
  margin-bottom: 8px;
  display: flex;
  gap: 8px;
  opacity: 0;
  animation: bootIn 0.5s ease forwards;
}

.boot-line:nth-child(1) { animation-delay: 0.2s; }
.boot-line:nth-child(2) { animation-delay: 0.4s; }
.boot-line:nth-child(3) { animation-delay: 0.6s; }
.boot-line:nth-child(4) { animation-delay: 0.8s; }
.boot-line:nth-child(5) { animation-delay: 1s; }

@keyframes bootIn {
  to { opacity: 1; }
}

.boot-prefix {
  color: #60a5fa;
  font-weight: 700;
}

.boot-content { color: #a1a1aa; }

.quick-commands {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.command-card {
  background: linear-gradient(135deg, rgba(96, 165, 250, 0.05) 0%, rgba(96, 165, 250, 0.02) 100%);
  border: 1px solid rgba(96, 165, 250, 0.15);
  border-radius: 8px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.command-card:hover {
  border-color: rgba(96, 165, 250, 0.4);
  background: rgba(96, 165, 250, 0.1);
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(96, 165, 250, 0.1);
}

.command-icon {
  font-size: 24px;
}

.command-text { flex: 1; }
.command-title {
  font-size: 12px;
  font-weight: 700;
  color: #e4e4e7;
  margin-bottom: 4px;
}
.command-desc {
  font-size: 11px;
  color: #71717a;
}

/* 消息列表 */
.messages-list {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px 20px;
}

.console-message {
  margin-bottom: 28px;
  position: relative;
}

.message-timestamp {
  margin-bottom: 8px;
}

.timestamp {
  font-size: 10px;
  color: #52525b;
  font-family: 'JetBrains Mono', monospace;
  letter-spacing: 1px;
}

.message-row {
  display: flex;
  gap: 16px;
}

.user-avatar, .ai-avatar {
  flex-shrink: 0;
  margin-top: 4px;
}

.avatar-container {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  background: linear-gradient(135deg, #a855f7 0%, #7c3aed 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  box-shadow: 0 0 20px rgba(168, 85, 247, 0.3);
}

.avatar-glow {
  position: absolute;
  inset: -2px;
  border-radius: 10px;
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.4), transparent);
  z-index: -1;
  animation: avatarPulse 2s ease-in-out infinite;
}

@keyframes avatarPulse {
  0%, 100% { opacity: 0.5; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.1); }
}

.avatar-text {
  color: white;
  font-weight: 900;
  font-size: 11px;
  letter-spacing: 1px;
}

/* R2-D2 头像 */
.r2d2-avatar {
  width: 48px;
  height: 56px;
  position: relative;
}

.r2d2-body {
  width: 100%;
  height: 100%;
  position: relative;
}

.r2d2-dome {
  position: absolute;
  top: 0;
  left: 4px;
  right: 4px;
  height: 24px;
  background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
  border-radius: 24px 24px 8px 8px;
  box-shadow: 0 4px 15px rgba(96, 165, 250, 0.4);
}

.r2d2-sensor {
  position: absolute;
  border-radius: 50%;
  background: #1a1a24;
}

.r2d2-sensor.main {
  top: 6px;
  left: 50%;
  transform: translateX(-50%);
  width: 12px;
  height: 12px;
  box-shadow: 0 0 10px rgba(74, 222, 128, 0.6);
}

.r2d2-sensor.small {
  width: 6px;
  height: 6px;
  top: 8px;
}

.r2d2-sensor.small.one { left: 8px; }
.r2d2-sensor.small.two { right: 8px; }

.r2d2-panel {
  position: absolute;
  bottom: 4px;
  left: 6px;
  right: 6px;
  height: 28px;
  background: linear-gradient(180deg, #f5f5f5 0%, #d4d4d4 100%);
  border-radius: 4px;
  box-shadow: inset 0 -2px 8px rgba(0, 0, 0, 0.2);
}

.r2d2-arm {
  position: absolute;
  bottom: 12px;
  width: 4px;
  height: 16px;
  background: #71717a;
  border-radius: 2px;
}

.r2d2-arm.left { left: 0; }
.r2d2-arm.right { right: 0; }

/* 思考状态动画 */
.r2d2-avatar.thinking .r2d2-dome {
  animation: domeGlow 1s ease-in-out infinite;
}

@keyframes domeGlow {
  0%, 100% { box-shadow: 0 4px 15px rgba(96, 165, 250, 0.4); }
  50% { box-shadow: 0 4px 25px rgba(96, 165, 250, 0.8); }
}

.sound-waves {
  position: absolute;
  bottom: -8px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 3px;
}

.wave {
  width: 4px;
  height: 12px;
  background: #60a5fa;
  border-radius: 2px;
  animation: waveBounce 0.6s ease-in-out infinite;
}

@keyframes waveBounce {
  0%, 100% { transform: scaleY(0.3); opacity: 0.3; }
  50% { transform: scaleY(1); opacity: 1; }
}

/* 消息内容 */
.message-content {
  flex: 1;
  min-width: 0;
}

.message-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.message-role {
  font-size: 10px;
  color: #60a5fa;
  font-weight: 700;
  letter-spacing: 1px;
}

.user-message .message-role {
  color: #a855f7;
}

.message-model {
  font-size: 9px;
  color: #52525b;
  padding: 2px 6px;
  background: rgba(96, 165, 250, 0.1);
  border-radius: 2px;
}

.message-body {
  background: rgba(255, 255, 255, 0.02);
  border-radius: 8px;
  padding: 16px;
  border-left: 2px solid;
}

.ai-message .message-body {
  border-left-color: #60a5fa;
  box-shadow: 0 0 20px rgba(96, 165, 250, 0.05);
}

.user-message .message-body {
  border-left-color: #a855f7;
  background: rgba(168, 85, 247, 0.05);
}

.message-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: transparent;
  border: 1px solid #27272a;
  border-radius: 4px;
  color: #71717a;
  font-size: 9px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: inherit;
}

.action-btn:hover {
  border-color: #60a5fa;
  color: #60a5fa;
  background: rgba(96, 165, 250, 0.1);
}

.action-icon {
  font-size: 11px;
}

/* 流式输出指示器 */
.streaming-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 0;
  margin-left: 64px;
}

.streaming-cursor {
  width: 8px;
  height: 16px;
  background: #60a5fa;
  animation: blink 0.8s step-end infinite;
}

.streaming-text {
  font-size: 11px;
  color: #60a5fa;
  font-weight: 700;
  letter-spacing: 2px;
}

.streaming-dots {
  display: flex;
  gap: 3px;
}

.streaming-dots .dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #60a5fa;
  animation: dotPulse 1s ease-in-out infinite;
}

.streaming-dots .dot:nth-child(2) { animation-delay: 0.2s; }
.streaming-dots .dot:nth-child(3) { animation-delay: 0.4s; }

@keyframes dotPulse {
  0%, 100% { opacity: 0.3; transform: scale(0.8); }
  50% { opacity: 1; transform: scale(1); }
}

/* 输入区域 */
.input-console {
  padding: 16px 20px 20px;
  background: linear-gradient(0deg, rgba(10, 10, 15, 0.95) 0%, rgba(10, 10, 15, 0.8) 100%);
  border-top: 1px solid #1a1a24;
  position: relative;
  z-index: 2;
}

.input-row {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  max-width: 800px;
  margin: 0 auto;
}

.input-prompt {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding-bottom: 8px;
  flex-shrink: 0;
}

.input-wrapper {
  flex: 1;
  min-width: 0;
}

.console-input {
  width: 100%;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid #27272a;
  border-radius: 6px;
  padding: 12px 14px;
  color: #e4e4e7;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  line-height: 1.5;
  resize: none;
  outline: none;
  transition: all 0.15s ease;
}

.console-input:focus {
  border-color: #60a5fa;
  box-shadow: 0 0 20px rgba(96, 165, 250, 0.15), inset 0 0 0 1px rgba(96, 165, 250, 0.3);
}

.console-input::placeholder {
  color: #52525b;
}

.input-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 4px;
  flex-shrink: 0;
}

.send-btn, .stop-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  border: none;
  border-radius: 6px;
  font-family: inherit;
  font-size: 11px;
  font-weight: 900;
  letter-spacing: 1px;
  text-transform: uppercase;
  cursor: pointer;
  transition: all 0.15s ease;
}

.send-btn {
  background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
  color: #0a0a0f;
  box-shadow: 0 4px 15px rgba(96, 165, 250, 0.3);
}

.send-btn:hover:not(.disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 25px rgba(96, 165, 250, 0.4);
}

.send-btn.disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.stop-btn {
  background: linear-gradient(135deg, #f87171 0%, #ef4444 100%);
  color: #0a0a0f;
  box-shadow: 0 4px 15px rgba(248, 113, 113, 0.3);
}

.stop-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 25px rgba(248, 113, 113, 0.4);
}

.input-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  max-width: 800px;
  margin: 10px auto 0;
  padding: 0 64px 0 0;
}

.footer-hint {
  font-size: 10px;
  color: #52525b;
}

.footer-shortcuts {
  display: flex;
  gap: 16px;
}

.shortcut {
  font-size: 10px;
  color: #3f3f46;
}

/* 历史面板 */
.history-panel {
  width: 240px;
  background: linear-gradient(180deg, #0d0d12 0%, #0a0a0f 100%);
  border-left: 1px solid #1a1a24;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.new-session-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: rgba(96, 165, 250, 0.1);
  border: 1px solid rgba(96, 165, 250, 0.2);
  border-radius: 4px;
  color: #60a5fa;
  font-family: inherit;
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s ease;
}

.new-session-btn:hover {
  background: rgba(96, 165, 250, 0.2);
}

.session-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.session-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  cursor: pointer;
  transition: all 0.15s ease;
  border-right: 2px solid transparent;
}

.session-item:hover {
  background: rgba(96, 165, 250, 0.05);
}

.session-item.active {
  background: rgba(96, 165, 250, 0.1);
  border-right-color: #60a5fa;
}

.session-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #3f3f46;
  flex-shrink: 0;
}

.session-item.active .session-dot {
  background: #60a5fa;
  box-shadow: 0 0 8px rgba(96, 165, 250, 0.5);
}

.session-info {
  flex: 1;
  min-width: 0;
}

.session-title {
  font-size: 11px;
  color: #a1a1aa;
  margin-bottom: 3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.session-item.active .session-title {
  color: #e4e4e7;
}

.session-time {
  font-size: 9px;
  color: #52525b;
}

.session-menu {
  flex-shrink: 0;
}

.menu-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #52525b;
  font-size: 14px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.menu-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #a1a1aa;
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(63, 63, 70, 0.5);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(96, 165, 250, 0.3);
}
</style>
