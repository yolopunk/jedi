<template>
  <div class="chat-console-page">
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
        <div ref="messagesContainer" class="messages-container" @scroll="handleScroll" @click="handleMessagesClick">
          <!-- 空状态（专业版，保留 R2 / HOLOCRON 识别度，去除动画特效） -->
          <div v-if="!store.currentSession || store.currentSession.messages.length === 0" class="chat-empty">
            <div class="empty-mark">R2</div>
            <h1 class="empty-title">Jedi Agent</h1>
            <p class="empty-subtitle">面向开发者的 AI 助手 · 可调用终端 / 文件系统 / Hosts 等技能</p>
            <div class="empty-examples">
              <button
                v-for="ex in examplePrompts"
                :key="ex"
                class="example-chip"
                @click="applyExample(ex)"
              >
                {{ ex }}
              </button>
            </div>
            <div class="empty-hint">
              <kbd>/</kbd> 命令面板 · <kbd>Enter</kbd> 发送 · <kbd>Shift</kbd>+<kbd>Enter</kbd> 换行
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
                  <div class="msg-avatar user" aria-hidden="true">
                    <span class="avatar-monogram">YOU</span>
                  </div>
                  <div class="message-content user-message">
                    <div class="message-body">
                      <div class="markdown-body" v-html="renderMessage(message.content)"></div>
                    </div>
                  </div>
                </template>

                <!-- AI 消息 -->
                <template v-else>
                  <div
                    class="msg-avatar ai"
                    :class="{ thinking: store.isLoading && index === displayMessages.length - 1 }"
                    aria-hidden="true"
                  >
                    <svg class="avatar-mark" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round">
                      <path d="M12 2.5 21.5 12 12 21.5 2.5 12 12 2.5Z" />
                      <path d="M12 7.5 16.5 12 12 16.5 7.5 12 12 7.5Z" opacity="0.55" />
                    </svg>
                  </div>
                  <div class="message-content ai-message">
                    <div class="message-meta">
                      <span class="model-badge">{{ currentModelName }}</span>
                    </div>
                    <div class="message-body">
                      <div class="markdown-body" v-html="renderMessage(message.content)"></div>
                      <!-- 流式输出指示器（内嵌于消息框内部） -->
                      <div v-if="message.isStreaming" class="streaming-indicator inline">
                        <div class="streaming-cursor"></div>
                        <span class="streaming-text">PROCESSING</span>
                        <span class="streaming-dots">
                          <span class="dot"></span>
                          <span class="dot"></span>
                          <span class="dot"></span>
                        </span>
                      </div>
                    </div>
                    <div class="message-actions">
                      <button class="action-btn" @click="handleCopyMessage(message.content)" title="复制">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <rect x="9" y="9" width="13" height="13" rx="2"/>
                          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                        </svg>
                      </button>
                      <button
                        v-if="index === displayMessages.length - 1"
                        class="action-btn"
                        @click="handleRegenerate"
                        title="重新生成"
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <path d="M23 4v6h-6M1 20v-6h6"/>
                          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
                        </svg>
                      </button>
                      <button
                        v-if="message.metadata?.trace?.length"
                        class="action-btn trace"
                        :class="{ active: isTraceActive(message) }"
                        @click="openTrace(message)"
                        title="查看调用 / 执行链路"
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
                        </svg>
                      </button>
                    </div>
                  </div>
                </template>
              </div>
            </div>
          </div>
        </div>

        <!-- 输入区域 -->
        <div class="input-console" :class="inputConsoleState">
          <!-- 统一胶囊容器 -->
          <div class="input-bar">
            <!-- 左侧工具栏 -->
            <button class="toolbar-btn" @click="openCommandPalette" title="Commands (/)">
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
            :commands="filteredCommands"
            :active-index="commandActiveIndex"
            @select="handleCommandSelect"
            @hover="commandActiveIndex = $event"
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

      <!-- 右侧：Agent Trace 抽屉 -->
      <Transition name="trace-scrim">
        <div v-if="agentStore.tracePanelOpen" class="trace-scrim" @click="closeTrace"></div>
      </Transition>
      <Transition name="trace-drawer">
        <AgentTrace v-if="agentStore.tracePanelOpen" class="trace-drawer" />
      </Transition>
    </div>

    <!-- Model Settings Dialog -->
    <ModelSettings v-model="showModelSettings" />

    <!-- Agent Pool Panel -->
    <AgentPoolPanel v-model:showPanel="showWorkersPanel" />

    <!-- Tool confirmation card (write/system-risk tools) -->
    <ToolConfirmCard />
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { formatCommandPrompt, SLASH_COMMANDS, type SlashCommand } from '@/agent/slashCommands'
import { setOnWorkerCompleteCallback, useAgentPool } from '@/agent/useAgentPool'
import AttachmentMenu from '@/components/AttachmentMenu.vue'
import AgentPoolPanel from '@/components/agent/AgentPoolPanel.vue'
import CommandPalette from '@/components/CommandPalette.vue'
import { useAgentStore } from '@/stores/agent'
import { useAiChatStore } from '@/stores/aiChat'
import { useModelsDevStore } from '@/stores/modelsDev'
import { useProviderConfigStore } from '@/stores/providerConfig'
import { useSkillsStore } from '@/stores/skills'
import { renderSafe, sharedMd } from '@/utils/markdown'
import AgentTrace from './AgentTrace.vue'
import ModelSettings from './ModelSettings.vue'
import ToolConfirmCard from './ToolConfirmCard.vue'

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
const commandActiveIndex = ref(0)
const showScrollButton = ref(false)

// Slash-command palette: open when the input is a single leading "/" token,
// filter by what's typed, and reset the keyboard highlight as the list changes.
const filteredCommands = computed<SlashCommand[]>(() => {
  const text = inputText.value.trimStart()
  if (!text.startsWith('/')) return []
  const token = text.slice(1).split(/\s/)[0].toLowerCase()
  return SLASH_COMMANDS.filter(cmd => cmd.name.slice(1).toLowerCase().startsWith(token))
})

watch(inputText, value => {
  const text = value.trimStart()
  showCommands.value = text.startsWith('/') && !/\s/.test(text) && filteredCommands.value.length > 0
  commandActiveIndex.value = 0
})
const showModelSettings = ref(false)
const showAttachmentMenu = ref(false)
const showModelDropdown = ref(false)
const isHistoryCollapsed = ref(true)
const showWorkersPanel = ref(false)

// Agent Pool
const { workers } = useAgentPool()

// Empty-state example prompts (developer-facing)
const examplePrompts = [
  '帮我排查 hosts 文件里重复的域名映射',
  '解释这段 Rust 代码并指出潜在 bug',
  '用一条 Tauri 命令读取当前系统内存占用',
]

function applyExample(text: string) {
  inputText.value = text
  autoResize()
}

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
  // Inject a language label + copy button into each highlighted code block.
  // Runs on the already-sanitized HTML with trusted markup; the only dynamic
  // value is the language id, which the highlighter already restricted to \w-.
  return renderSafe(sharedMd, content).replace(
    /<pre class="hljs"(?: data-lang="([^"]*)")?>/g,
    (_m, lang) => {
      const label = lang ? `<span class="code-lang">${lang}</span>` : ''
      return `<pre class="hljs">${label}<button class="code-copy-btn" type="button">Copy</button>`
    }
  )
}

// Delegated copy handler for the code blocks rendered via v-html.
function handleMessagesClick(e: MouseEvent) {
  const target = (e.target as HTMLElement | null)?.closest('.code-copy-btn') as HTMLElement | null
  if (!target) return
  const pre = target.closest('pre.hljs')
  const code = pre?.querySelector('code')
  if (!code) return
  navigator.clipboard
    .writeText(code.textContent || '')
    .then(() => {
      target.textContent = 'Copied'
      target.classList.add('copied')
      window.setTimeout(() => {
        target.textContent = 'Copy'
        target.classList.remove('copied')
      }, 1500)
    })
    .catch(() => {})
}

// Trace panel: one button per assistant turn opens the right rail focused on
// that message's run, instead of dumping the whole trace inline under the bubble.
type ChatMessage = NonNullable<typeof store.currentSession>['messages'][number]

function openTrace(message: ChatMessage) {
  if (isTraceActive(message)) {
    agentStore.tracePanelOpen = false
    agentStore.selectedTrace = null
    return
  }
  agentStore.selectedTrace = message.metadata ?? null
  agentStore.tracePanelOpen = true
}

function isTraceActive(message: ChatMessage): boolean {
  return agentStore.tracePanelOpen && agentStore.selectedTrace === message.metadata
}

function closeTrace() {
  agentStore.tracePanelOpen = false
  agentStore.selectedTrace = null
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
function openCommandPalette() {
  if (!inputText.value.trimStart().startsWith('/')) {
    inputText.value = '/'
  }
  nextTick(() => inputRef.value?.focus())
}

function handleCommandSelect(cmd: SlashCommand) {
  if (!store.currentSession) {
    store.createSession(
      '新对话',
      modelsDevStore.selectedProviderId || 'openai',
      modelsDevStore.selectedModelId || 'gpt-4o-mini'
    )
  }
  inputText.value = `${cmd.name} `
  showCommands.value = false
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
  // When the slash-command palette is open, keys drive the list (keyboard-first).
  if (showCommands.value && filteredCommands.value.length > 0) {
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      commandActiveIndex.value = (commandActiveIndex.value + 1) % filteredCommands.value.length
      return
    }
    if (event.key === 'ArrowUp') {
      event.preventDefault()
      commandActiveIndex.value =
        (commandActiveIndex.value - 1 + filteredCommands.value.length) %
        filteredCommands.value.length
      return
    }
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      const cmd = filteredCommands.value[commandActiveIndex.value]
      if (cmd) handleCommandSelect(cmd)
      return
    }
    if (event.key === 'Escape') {
      event.preventDefault()
      showCommands.value = false
      return
    }
  }

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

onMounted(async () => {
  skillsStore.loadFromStorage()
  await Promise.all([
    modelsDevStore.fetchProviders(),
    providerConfigStore.loadConfiguredProviders(),
    store.loadSessions(),
  ])
  scrollToBottom()

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
</script>

<style src="./chat.css" scoped></style>

<!--
  Non-scoped: assistant answers are rendered via v-html, so their DOM does NOT
  receive the scoped data-v attribute. Markdown/code styling must therefore live
  in a global block (contained under .markdown-body, which only appears in chat).
-->
<style>
.markdown-body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC',
    'Hiragino Sans GB', 'Microsoft YaHei', Roboto, sans-serif;
  font-size: 14.5px;
  line-height: 1.72;
  color: var(--text-muted);
  word-break: break-word;
}

.markdown-body > *:first-child {
  margin-top: 0;
}
.markdown-body > *:last-child {
  margin-bottom: 0;
}

.markdown-body p {
  margin: 0 0 12px;
}

.markdown-body h1,
.markdown-body h2,
.markdown-body h3,
.markdown-body h4 {
  margin: 22px 0 10px;
  line-height: 1.35;
  font-weight: 650;
  color: var(--bg-elevated);
  letter-spacing: 0.2px;
}
.markdown-body h1 {
  font-size: 20px;
  padding-bottom: 7px;
  border-bottom: 1px solid rgb(var(--text-rgb) / 0.08);
}
.markdown-body h2 {
  font-size: 17px;
}
.markdown-body h3 {
  font-size: 15px;
}
.markdown-body h4 {
  font-size: 14px;
  color: var(--text-muted);
}

.markdown-body strong {
  font-weight: 700;
  color: var(--bg-elevated);
}
.markdown-body em {
  font-style: italic;
}

.markdown-body a {
  color: var(--accent);
  text-decoration: none;
  border-bottom: 1px solid rgb(var(--accent-rgb) / 0.35);
}
.markdown-body a:hover {
  border-bottom-color: var(--accent);
}

/* Lists */
.markdown-body ul,
.markdown-body ol {
  margin: 10px 0 14px;
  padding-left: 22px;
}
.markdown-body li {
  margin: 5px 0;
  line-height: 1.65;
}
.markdown-body li::marker {
  color: var(--accent);
}
.markdown-body ul ul,
.markdown-body ol ol,
.markdown-body ul ol,
.markdown-body ol ul {
  margin: 5px 0;
}

/* Inline code */
.markdown-body :not(pre) > code {
  font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.86em;
  padding: 2px 6px;
  border-radius: 6px;
  background: rgb(var(--accent-rgb) / 0.12);
  border: 1px solid rgb(var(--accent-rgb) / 0.18);
  color: var(--accent-hover);
  white-space: break-spaces;
}

/* Code blocks: a real card with contained horizontal scroll */
.markdown-body pre.hljs {
  position: relative;
  margin: 12px 0;
  padding: 34px 14px 14px;
  background: var(--bg-terminal);
  border: 1px solid rgb(var(--text-rgb) / 0.08);
  border-radius: 10px;
  overflow-x: auto;
  max-width: 100%;
}
.markdown-body pre.hljs code {
  display: block;
  font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12.5px;
  line-height: 1.6;
  color: var(--text);
  background: transparent;
  border: 0;
  padding: 0;
  white-space: pre;
}

/* Code block top bar: language label (left) + copy button (right) */
.markdown-body pre.hljs > .code-lang {
  position: absolute;
  top: 9px;
  left: 13px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--text-subtle);
  pointer-events: none;
}
.markdown-body pre.hljs > .code-copy-btn {
  position: absolute;
  top: 6px;
  right: 8px;
  padding: 3px 10px;
  background:rgb(var(--ink-rgb) / 0.06);
  border: 1px solid rgb(var(--text-rgb) / 0.12);
  border-radius: 6px;
  color: var(--text-subtle);
  font-family: -apple-system, sans-serif;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
}
.markdown-body pre.hljs > .code-copy-btn:hover {
  background: rgb(var(--accent-rgb) / 0.14);
  border-color: rgb(var(--accent-rgb) / 0.5);
  color: var(--accent);
}
.markdown-body pre.hljs > .code-copy-btn.copied {
  color: var(--success);
  border-color: rgb(var(--success-rgb) / 0.4);
}

.markdown-body blockquote {
  margin: 12px 0;
  padding: 6px 14px;
  border-left: 3px solid rgb(var(--accent-rgb) / 0.45);
  background: rgb(var(--accent-rgb) / 0.05);
  color: var(--text-subtle);
  border-radius: 0 8px 8px 0;
}

.markdown-body hr {
  margin: 18px 0;
  border: 0;
  border-top: 1px solid rgb(var(--text-rgb) / 0.08);
}

.markdown-body table {
  width: 100%;
  margin: 14px 0;
  border-collapse: collapse;
  font-size: 13px;
}
.markdown-body th,
.markdown-body td {
  padding: 8px 11px;
  border: 1px solid rgb(var(--text-rgb) / 0.1);
  text-align: left;
}
.markdown-body th {
  background:rgb(var(--ink-rgb) / 0.04);
  font-weight: 600;
  color: var(--bg-elevated);
}

/* highlight.js token palette (GitHub-dark-ish), scoped to chat code */
.markdown-body .hljs-comment,
.markdown-body .hljs-quote {
  color: var(--text-subtle);
  font-style: italic;
}
.markdown-body .hljs-keyword,
.markdown-body .hljs-selector-tag,
.markdown-body .hljs-built_in,
.markdown-body .hljs-name,
.markdown-body .hljs-tag {
  color: var(--danger);
}
.markdown-body .hljs-string,
.markdown-body .hljs-title,
.markdown-body .hljs-section,
.markdown-body .hljs-attribute,
.markdown-body .hljs-literal,
.markdown-body .hljs-template-tag,
.markdown-body .hljs-template-variable,
.markdown-body .hljs-type,
.markdown-body .hljs-addition {
  color: var(--accent);
}
.markdown-body .hljs-number,
.markdown-body .hljs-symbol,
.markdown-body .hljs-bullet,
.markdown-body .hljs-link {
  color: var(--accent);
}
.markdown-body .hljs-title.function_,
.markdown-body .hljs-function .hljs-title {
  color: var(--accent-2);
}
.markdown-body .hljs-attr,
.markdown-body .hljs-variable,
.markdown-body .hljs-meta {
  color: var(--warning);
}
.markdown-body .hljs-emphasis {
  font-style: italic;
}
.markdown-body .hljs-strong {
  font-weight: 700;
}
</style>
