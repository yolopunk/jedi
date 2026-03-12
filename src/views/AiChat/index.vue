<template>
  <div class="chat-page">
    <div class="chat-layout">
      <!-- Main Chat Area -->
      <div class="chat-area">
        <!-- Chat Header -->
        <div class="chat-header">
          <div class="d-flex align-center">
            <v-btn
              icon
              size="small"
              variant="text"
              class="mr-2 history-btn"
              @click="showHistoryDialog = true"
            >
              <v-icon :icon="mdiHistory" />
              <v-tooltip activator="parent" location="bottom">{{ t('chat.history') }}</v-tooltip>
            </v-btn>

            <div class="model-selector-wrapper">
              <v-select
                v-model="store.selectedModelId"
                :items="modelOptions"
                item-title="name"
                item-value="id"
                density="compact"
                variant="outlined"
                hide-details
                class="model-selector"
                :menu-props="{ contentClass: 'model-dropdown rounded-lg' }"
                :disabled="store.isLoading"
                @update:model-value="onModelChange"
              >
                <template v-slot:prepend-inner>
                  <v-icon :icon="mdiStar" size="16" class="mr-2" color="primary" />
                </template>
                <template v-slot:item="{ item, props }">
                  <v-list-item v-bind="props" density="compact">
                    <template v-slot:append>
                      <span class="text-caption text-medium-emphasis">{{ (item as any).raw.providerName }}</span>
                    </template>
                  </v-list-item>
                </template>
              </v-select>
            </div>
          </div>

          <div class="header-actions">
            <v-btn
              v-if="store.currentSession && store.currentSession.messages.length > 0"
              icon
              size="small"
              variant="text"
              class="ml-1"
              @click="handleNewSession"
            >
              <v-icon :icon="mdiPlus" />
              <v-tooltip activator="parent" location="bottom">{{ t('chat.newChat') }}</v-tooltip>
            </v-btn>
            <v-btn
              v-if="store.currentSession && store.currentSession.messages.length > 0"
              icon
              size="small"
              variant="text"
              class="ml-1"
              @click="handleClearChat"
            >
              <v-icon :icon="mdiDeleteOutline" />
              <v-tooltip activator="parent" location="bottom">{{ t('chat.clearContext') }}</v-tooltip>
            </v-btn>
          </div>
        </div>

        <!-- Messages Container -->
        <div ref="messagesContainer" class="messages-container" @scroll="handleScroll">
          <!-- Welcome Screen -->
          <div v-if="!store.currentSession || store.currentSession.messages.length === 0" class="welcome-screen">
            <div class="welcome-content">
              <div class="welcome-logo text-center">
                <v-avatar size="56" color="primary" variant="tonal" class="mb-4">
                  <v-icon :icon="mdiRobot" size="28" />
                </v-avatar>
                <h1 class="text-h5 font-weight-bold mb-2">{{ t('chat.welcome.title') }}</h1>
                <p class="text-body-2 text-medium-emphasis">{{ t('chat.welcome.subtitle') }}</p>
              </div>

              <!-- Quick Prompts -->
              <div class="quick-prompts-grid">
                <div
                  v-for="prompt in quickPrompts"
                  :key="prompt.title"
                  class="prompt-card"
                  @click="handleQuickPrompt(prompt)"
                >
                  <v-icon :icon="prompt.icon" class="prompt-icon mb-2" size="20" color="primary" />
                  <div class="text-left">
                    <div class="font-weight-medium prompt-title">{{ prompt.title }}</div>
                    <div class="text-caption text-medium-emphasis prompt-hint">{{ prompt.hint }}</div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Messages List -->
          <div v-else class="messages-list">
            <div
              v-for="(message, index) in displayMessages"
              :key="index"
              class="message-wrapper"
              :class="message.role"
            >
              <div class="message-container">
                <div class="message-avatar">
                  <v-avatar
                    :size="message.role === 'user' ? 28 : 32"
                    :color="message.role === 'user' ? 'secondary' : 'primary'"
                    variant="tonal"
                  >
                    <v-icon :icon="message.role === 'user' ? mdiAccount : mdiRobot" :size="message.role === 'user' ? 16 : 18" />
                  </v-avatar>
                </div>

                <div class="message-content-wrapper">
                  <div class="message-meta" v-if="message.role === 'assistant'">
                    <span class="font-weight-medium text-caption">{{ t('chat.roles.assistant') }}</span>
                  </div>

                  <div class="message-bubble">
                    <div class="markdown-body" v-html="renderMessage(message.content)"></div>
                  </div>

                  <div v-if="message.role === 'assistant'" class="message-actions">
                    <v-btn
                      icon
                      size="x-small"
                      variant="text"
                      @click="handleCopyMessage(message.content)"
                    >
                      <v-icon :icon="mdiContentCopy" size="14" />
                      <v-tooltip activator="parent" location="top">{{ t('chat.copy') }}</v-tooltip>
                    </v-btn>
                    <v-btn
                      v-if="index === displayMessages.length - 1"
                      icon
                      size="x-small"
                      variant="text"
                      @click="handleRegenerate"
                    >
                      <v-icon :icon="mdiRefresh" size="14" />
                      <v-tooltip activator="parent" location="top">{{ t('chat.regenerate') }}</v-tooltip>
                    </v-btn>
                  </div>
                </div>
              </div>
            </div>

            <!-- Streaming indicator -->
            <div v-if="store.isLoading" class="message-wrapper assistant">
              <div class="message-container">
                <div class="message-avatar">
                  <v-avatar size="32" color="primary" variant="tonal">
                    <v-icon :icon="mdiRobot" size="18" />
                  </v-avatar>
                </div>
                <div class="message-content-wrapper">
                  <div class="message-meta">
                    <span class="font-weight-medium text-caption">{{ t('chat.roles.assistant') }}</span>
                  </div>
                  <div class="message-bubble">
                    <div class="streaming-content">
                      <span class="markdown-body" v-html="renderMessage(store.streamingContent)"></span>
                      <span class="cursor"></span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div style="height: 24px;"></div>
          </div>
        </div>

        <!-- Scroll to bottom button -->
        <v-scale-transition>
          <v-btn
            v-show="showScrollButton"
            icon
            size="small"
            color="surface"
            elevation="2"
            class="scroll-bottom-btn"
            @click="scrollToBottom"
          >
            <v-icon :icon="mdiChevronDown" />
          </v-btn>
        </v-scale-transition>

        <!-- Input Area -->
        <div class="input-area-wrapper">
          <div class="input-container">
            <v-textarea
              ref="inputRef"
              v-model="inputText"
              :placeholder="placeholder"
              variant="outlined"
              auto-grow
              rows="1"
              max-rows="8"
              hide-details
              class="chat-input"
              rounded="lg"
              @keydown="handleKeydown"
            >
              <template v-slot:prepend-inner>
                <v-btn
                  icon
                  size="small"
                  variant="text"
                  class="attachment-btn"
                  @click="store.toggleMcpServer('default')"
                >
                  <v-icon :icon="mdiPuzzle" size="18" />
                </v-btn>
              </template>
              <template v-slot:append-inner>
                <div class="input-actions d-flex align-center">
                  <v-btn
                    v-if="!store.isLoading"
                    icon
                    size="small"
                    color="primary"
                    :disabled="!inputText.trim()"
                    class="send-btn"
                    @click="handleSend"
                  >
                    <v-icon :icon="mdiSend" size="18" />
                  </v-btn>
                  <v-btn
                    v-else
                    icon
                    size="small"
                    variant="tonal"
                    color="error"
                    class="stop-btn"
                    @click="handleStop"
                  >
                    <v-icon :icon="mdiStop" size="18" />
                  </v-btn>
                </div>
              </template>
            </v-textarea>

            <div class="input-footer">
              <div class="text-caption text-disabled input-hint">
                {{ t('chat.inputHint') }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- History Drawer -->
    <v-navigation-drawer
      v-model="showHistoryDialog"
      location="left"
      temporary
      width="280"
      class="history-drawer"
    >
      <div class="drawer-header pa-4 d-flex align-center justify-space-between border-bottom">
        <div class="text-h6 font-weight-bold">{{ t('chat.history') }}</div>
        <v-btn
          variant="flat"
          color="primary"
          size="small"
          class="new-chat-btn"
          @click="handleNewSessionAndClose"
          icon
        >
          <v-icon :icon="mdiPlus" />
          <v-tooltip activator="parent" location="bottom">{{ t('chat.newChat') }}</v-tooltip>
        </v-btn>
      </div>

      <div class="session-list-container h-100 overflow-y-auto">
        <div v-if="store.sessions.length === 0" class="empty-sessions text-center mt-8 text-medium-emphasis">
          <v-icon :icon="mdiChatOutline" size="48" class="mb-2 opacity-50" />
          <div class="text-body-2">{{ t('chat.noHistory') }}</div>
        </div>

        <div v-else class="session-list pa-2">
          <div
            v-for="session in store.sessions"
            :key="session.id"
            class="session-item rounded-lg mb-1"
            :class="{ active: session.id === store.currentSessionId }"
            @click="handleSelectSessionAndClose(session.id)"
          >
            <div class="session-item-content py-2 px-3">
              <div class="session-title text-truncate font-weight-medium">{{ session.title }}</div>
              <div class="session-meta text-caption text-medium-emphasis">
                <span class="session-date">{{ formatDate(session.updated_at) }}</span>
              </div>
            </div>

            <div class="session-actions px-2">
              <v-menu location="bottom end">
                <template v-slot:activator="{ props }">
                  <v-btn
                    icon
                    size="x-small"
                    variant="text"
                    v-bind="props"
                    class="more-btn"
                    @click.stop
                  >
                    <v-icon :icon="mdiDotsVertical" size="16" />
                  </v-btn>
                </template>
                <v-list density="compact" width="150" class="rounded-lg">
                  <v-list-item @click="handleRenameSession(session)" density="compact">
                    <template v-slot:prepend>
                      <v-icon :icon="mdiPencil" size="16" class="mr-2" />
                    </template>
                    <v-list-item-title>{{ t('chat.rename') }}</v-list-item-title>
                  </v-list-item>
                  <v-list-item @click="handleExportSession(session)" density="compact">
                    <template v-slot:prepend>
                      <v-icon :icon="mdiDownload" size="16" class="mr-2" />
                    </template>
                    <v-list-item-title>{{ t('chat.export') }}</v-list-item-title>
                  </v-list-item>
                  <v-divider class="my-1" />
                  <v-list-item class="text-error" @click="handleDeleteSession(session.id)" density="compact">
                    <template v-slot:prepend>
                      <v-icon :icon="mdiDelete" size="16" class="mr-2" />
                    </template>
                    <v-list-item-title>{{ t('chat.delete') }}</v-list-item-title>
                  </v-list-item>
                </v-list>
              </v-menu>
            </div>
          </div>
        </div>
      </div>
    </v-navigation-drawer>

    <!-- Rename Dialog -->
    <v-dialog v-model="showRenameDialog" max-width="400">
      <v-card class="rounded-lg">
        <v-card-title class="text-body-1 font-weight-bold pa-4">{{ t('chat.dialog.renameTitle') }}</v-card-title>
        <v-card-text class="px-4 pb-0">
          <v-text-field
            v-model="renameTitle"
            :placeholder="t('chat.dialog.renamePlaceholder')"
            variant="outlined"
            density="compact"
            autofocus
            @keydown.enter="confirmRename"
          />
        </v-card-text>
        <v-card-actions class="px-4 pb-4">
          <v-spacer />
          <v-btn variant="text" size="small" @click="showRenameDialog = false">{{ t('chat.dialog.cancel') }}</v-btn>
          <v-btn color="primary" variant="flat" size="small" @click="confirmRename">{{ t('chat.dialog.save') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAiChatStore, type Session } from '@/stores/aiChat'
import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'
import DOMPurify from 'dompurify'
import {
  mdiPlus, mdiChatOutline, mdiDotsVertical, mdiPencil, mdiDownload, mdiDelete,
  mdiMenu, mdiRobot, mdiDeleteOutline, mdiAccount, mdiContentCopy, mdiRefresh,
  mdiChevronDown, mdiSend, mdiStop, mdiPuzzle, mdiLightbulb, mdiCodeBraces,
  mdiTextBoxMultiple, mdiLightningBolt, mdiHistory, mdiStar
} from '@mdi/js'

// Component Props/Emits
const emit = defineEmits<{
  (e: 'open-settings'): void
}>()

const { t } = useI18n()
const store = useAiChatStore()

// UI State
const showHistoryDialog = ref(false)
const showRenameDialog = ref(false)
const renameTitle = ref('')
const renamingSession = ref<Session | null>(null)
const inputText = ref('')
const messagesContainer = ref<HTMLElement | null>(null)
const inputRef = ref<any>(null)
const showScrollButton = ref(false)

// Markdown renderer
const md = MarkdownIt({
  highlight: (str, lang) => {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(str, { language: lang }).value
      } catch (e) {
        // ignore
      }
    }
    return ''
  },
  linkify: true,
  breaks: true,
})

// Quick prompts
const quickPrompts = computed(() => [
  { title: t('chat.prompts.explain.title'), hint: t('chat.prompts.explain.hint'), icon: mdiLightbulb, prompt: 'Please explain this concept in detail: ' },
  { title: t('chat.prompts.code.title'), hint: t('chat.prompts.code.hint'), icon: mdiCodeBraces, prompt: 'Please help me write code for: ' },
  { title: t('chat.prompts.summarize.title'), hint: t('chat.prompts.summarize.hint'), icon: mdiTextBoxMultiple, prompt: 'Please summarize the following content:\n\n' },
  { title: t('chat.prompts.brainstorm.title'), hint: t('chat.prompts.brainstorm.hint'), icon: mdiLightningBolt, prompt: 'Help me brainstorm ideas for: ' },
])

// Computed
const modelOptions = computed(() => store.availableModels)

const placeholder = computed(() => {
  if (store.availableModels.length === 0) {
    return t('chat.error.providerConfig')
  }
  return t('chat.inputPlaceholder')
})

const displayMessages = computed(() => {
  return store.currentSession?.messages || []
})

// Methods
function renderMessage(content: string) {
  if (!content) return ''
  const html = md.render(content)
  return DOMPurify.sanitize(html)
}

function formatDate(dateStr: string) {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  if (diff < 60000) return t('chat.time.justNow')
  if (diff < 3600000) return t('chat.time.minutesAgo', { n: Math.floor(diff / 60000) })
  if (diff < 86400000) return t('chat.time.hoursAgo', { n: Math.floor(diff / 3600000) })
  if (diff < 604800000) return t('chat.time.daysAgo', { n: Math.floor(diff / 86400000) })

  return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
}

function handleNewSessionAndClose() {
  handleNewSession()
  showHistoryDialog.value = false
}

function handleSelectSessionAndClose(sessionId: string) {
  handleSelectSession(sessionId)
  showHistoryDialog.value = false
}

function handleNewSession() {
  store.createSession()
  nextTick(() => {
    inputRef.value?.focus()
  })
}

function handleSelectSession(sessionId: string) {
  store.currentSessionId = sessionId
  nextTick(() => {
    scrollToBottom()
  })
}

function handleRenameSession(session: Session) {
  renamingSession.value = session
  renameTitle.value = session.title
  showRenameDialog.value = true
}

function confirmRename() {
  if (renamingSession.value && renameTitle.value.trim()) {
    renamingSession.value.title = renameTitle.value.trim()
  }
  showRenameDialog.value = false
}

function handleExportSession(session: Session) {
  const content = session.messages
    .map(m => `## ${m.role === 'user' ? 'User' : 'AI'}\n\n${m.content}`)
    .join('\n\n---\n\n')

  const blob = new Blob([content], { type: 'text/markdown' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${session.title}.md`
  a.click()
  URL.revokeObjectURL(url)
}

function handleDeleteSession(sessionId: string) {
  if (confirm(t('chat.confirm.delete'))) {
    store.deleteSession(sessionId)
  }
}

function handleClearChat() {
  if (confirm(t('chat.confirm.clear'))) {
    if (store.currentSession) {
      store.currentSession.messages = []
    }
  }
}

function handleQuickPrompt(prompt: typeof quickPrompts[0]) {
  if (!store.currentSession) {
    store.createSession()
  }
  inputText.value = prompt.prompt
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

  try {
    await store.sendMessage(content)
    scrollToBottom()
  } catch (e) {
    console.error('Failed to send message:', e)
  }
}

function handleStop() {
  // Implement stop logic if store supports it
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

function onModelChange() {
  if (store.selectedModelId) {
    store.setSelectedModel(store.selectedModelId)
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
.chat-page {
  height: 100%;
  width: 100%;
  display: flex;
  overflow: hidden;
  background-color: rgb(var(--v-theme-background));
}

.chat-layout {
  display: flex;
  width: 100%;
  height: 100%;
}

/* Chat Area */
.chat-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  position: relative;
  min-width: 0;
  height: 100%;
}

.chat-header {
  height: 56px;
  padding: 0 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  flex-shrink: 0;
  z-index: 10;
}

.history-btn {
  border-radius: 8px;
}

.model-selector-wrapper {
  min-width: 180px;
}

.model-selector {
  min-width: 180px;
}

.model-selector :deep(.v-field) {
  border-radius: 12px;
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
  scroll-behavior: smooth;
}

.messages-list {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px 20px;
}

/* Welcome Screen */
.welcome-screen {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.welcome-content {
  max-width: 700px;
  width: 100%;
}

.welcome-logo {
  margin-bottom: 32px;
}

.quick-prompts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-top: 24px;
}

.prompt-card {
  border: 1px solid rgba(var(--v-theme-on-surface), 0.1);
  border-radius: 12px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  background-color: rgba(var(--v-theme-surface), 0.5);
  display: flex;
  flex-direction: column;
}

.prompt-card:hover {
  background-color: rgba(var(--v-theme-surface), 0.8);
  border-color: rgba(var(--v-theme-primary), 0.3);
  transform: translateY(-2px);
}

.prompt-title {
  font-size: 0.9rem;
  margin-bottom: 2px;
}

.prompt-hint {
  line-height: 1.4;
}

/* Message Styles */
.message-wrapper {
  margin-bottom: 24px;
}

.message-wrapper.user {
  margin-bottom: 32px;
}

.message-container {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.message-wrapper.user .message-container {
  flex-direction: row-reverse;
  gap: 12px;
}

.message-avatar {
  flex-shrink: 0;
  margin-top: 2px;
}

.message-content-wrapper {
  flex: 1;
  min-width: 0;
}

.message-wrapper.user .message-content-wrapper {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.message-meta {
  margin-bottom: 6px;
}

.message-wrapper.user .message-meta {
  display: none;
}

.message-bubble {
  position: relative;
  line-height: 1.6;
  font-size: 0.95rem;
}

.message-wrapper.user .message-bubble {
  background-color: rgba(var(--v-theme-surface), 0.8);
  border-radius: 16px;
  padding: 10px 16px;
  max-width: 80%;
  width: fit-content;
}

.message-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 8px;
  opacity: 0;
  transition: opacity 0.2s;
}

.message-wrapper:hover .message-actions {
  opacity: 1;
}

/* Markdown Styles */
.markdown-body {
  color: rgb(var(--v-theme-on-surface));
}

.markdown-body :deep(pre) {
  background-color: rgba(var(--v-theme-surface), 0.8) !important;
  border-radius: 12px;
  padding: 16px;
  margin: 12px 0;
  overflow-x: auto;
}

.markdown-body :deep(code) {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.85em;
  background-color: rgba(var(--v-theme-on-surface), 0.08);
  padding: 2px 6px;
  border-radius: 6px;
}

.markdown-body :deep(pre code) {
  background-color: transparent;
  padding: 0;
  color: inherit;
}

.markdown-body :deep(p) {
  margin-bottom: 12px;
}

.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}

/* Input Area */
.input-area-wrapper {
  padding: 16px 20px 20px;
  background: linear-gradient(to top, rgb(var(--v-theme-background)) 60%, transparent);
  flex-shrink: 0;
}

.input-container {
  max-width: 800px;
  margin: 0 auto;
  width: 100%;
}

.chat-input :deep(.v-field) {
  border-radius: 16px !important;
  background-color: rgba(var(--v-theme-surface), 0.6) !important;
}

.chat-input :deep(.v-field__input) {
  padding: 12px 12px !important;
  font-size: 0.95rem;
  line-height: 1.5;
  max-height: 200px;
}

.attachment-btn {
  border-radius: 8px;
}

.send-btn {
  border-radius: 50%;
  width: 36px;
  height: 36px;
}

.stop-btn {
  border-radius: 50%;
  width: 36px;
  height: 36px;
}

.input-footer {
  margin-top: 8px;
  text-align: center;
}

.input-hint {
  font-size: 11px;
  opacity: 0.6;
}

/* Scroll Button */
.scroll-bottom-btn {
  position: absolute;
  bottom: 140px;
  right: 24px;
  z-index: 10;
  border-radius: 50%;
}

/* Animations */
.cursor {
  display: inline-block;
  width: 2px;
  height: 1em;
  background-color: currentColor;
  margin-left: 2px;
  animation: blink 1s step-end infinite;
  vertical-align: text-bottom;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

/* History Drawer */
.history-drawer :deep(.v-navigation-drawer__content) {
  display: flex;
  flex-direction: column;
}

.drawer-header {
  flex-shrink: 0;
}

.session-list-container {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 16px;
}

.session-list-container::-webkit-scrollbar {
  width: 4px;
}

.session-list-container::-webkit-scrollbar-thumb {
  background: transparent;
}

.session-list-container:hover::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-on-surface), 0.1);
  border-radius: 2px;
}

.session-item {
  padding: 8px 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  transition: all 0.2s;
  border-radius: 8px;
}

.session-item:hover {
  background-color: rgba(var(--v-theme-on-surface), 0.04);
}

.session-item.active {
  background-color: rgba(var(--v-theme-primary), 0.1);
}

.session-item-content {
  flex: 1;
  min-width: 0;
  margin-right: 8px;
}

.session-title {
  font-size: 0.9rem;
  font-weight: 500;
  margin-bottom: 2px;
  color: rgb(var(--v-theme-on-surface));
}

.session-meta {
  font-size: 0.75rem;
  color: rgba(var(--v-theme-on-surface), 0.6);
}

.session-actions {
  opacity: 0;
  transition: opacity 0.2s;
}

.session-item:hover .session-actions,
.session-item.active .session-actions {
  opacity: 1;
}

/* Responsive */
@media (max-width: 640px) {
  .quick-prompts-grid {
    grid-template-columns: 1fr;
  }

  .chat-header {
    padding: 0 12px;
  }

  .messages-list {
    padding: 16px 12px;
  }

  .input-area-wrapper {
    padding: 12px;
  }
}
</style>
