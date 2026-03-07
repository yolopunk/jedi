<template>
  <div class="ai-chat-page">
    <!-- Sidebar -->
    <v-navigation-drawer
      v-model="showSidebar"
      :rail="railMode"
      :width="300"
      :mobile-breakpoint="960"
      class="chat-sidebar"
      @update:rail="railMode = $event"
    >
      <SessionList
        :sessions="store.sessions"
        :current-session-id="store.currentSessionId"
        @new-session="handleNewSession"
        @select-session="handleSelectSession"
        @rename-session="handleRenameSession"
        @export-session="handleExportSession"
        @delete-session="handleDeleteSession"
        @open-settings="showSettings = true"
      />
    </v-navigation-drawer>

    <!-- Main Content -->
    <v-main class="chat-main">
      <div class="chat-container">
        <!-- Header -->
        <div class="chat-header">
          <v-btn
            v-if="!showSidebar || $vuetify.display.mobile"
            icon
            variant="text"
            @click="showSidebar = !showSidebar"
          >
            <v-icon icon="mdi-menu" />
          </v-btn>

          <div class="header-title">
            <h1 v-if="store.currentSession">
              {{ store.currentSession.title }}
            </h1>
            <h1 v-else>AI Chat</h1>
          </div>

          <div class="header-actions">
            <ModelSelector
              v-model="selectedModelId"
              :models="store.allModels"
              :disabled="store.isLoading"
              class="model-selector"
            />

            <v-btn
              icon
              variant="text"
              @click="showSettings = true"
            >
              <v-icon icon="mdi-cog" />
            </v-btn>
          </div>
        </div>

        <!-- Messages Area -->
        <div ref="messagesContainer" class="messages-area">
          <!-- Welcome Screen -->
          <div v-if="!store.currentSession || store.currentMessages.length === 0" class="welcome-screen">
            <div class="welcome-content">
              <div class="welcome-icon">
                <v-icon icon="mdi-robot-happy" size="64" color="primary" />
              </div>
              <h2>Welcome to AI Chat</h2>
              <p>Start a conversation or choose a quick action below</p>
              
              <div class="quick-actions">
                <v-btn
                  v-for="action in quickActions"
                  :key="action.title"
                  variant="outlined"
                  class="quick-action-btn"
                  @click="handleQuickAction(action)"
                >
                  <v-icon :icon="action.icon" start />
                  {{ action.title }}
                </v-btn>
              </div>
            </div>
          </div>

          <!-- Messages List -->
          <TransitionGroup v-else name="message" tag="div" class="messages-list">
            <ChatMessage
              v-for="message in store.currentMessages"
              :key="message.id"
              :message="message"
              @regenerate="handleRegenerate(message)"
            />
          </TransitionGroup>

          <!-- Scroll to Bottom Button -->
          <v-btn
            v-show="showScrollButton"
            icon
            color="primary"
            class="scroll-to-bottom"
            @click="scrollToBottom"
          >
            <v-icon icon="mdi-chevron-down" />
          </v-btn>
        </div>

        <!-- Input Area -->
        <ChatInput
          ref="chatInputRef"
          :is-loading="store.isLoading"
          :selected-model="selectedModel"
          :stream-enabled="store.settings.streamEnabled"
          @send="handleSendMessage"
          @stop="handleStopGeneration"
        />
      </div>
    </v-main>

    <!-- Settings Dialog -->
    <ModelSettings
      v-model="showSettings"
      :providers="store.providers"
      :settings="store.settings"
      @save-provider="handleSaveProvider"
      @delete-provider="handleDeleteProvider"
      @update-settings="handleUpdateSettings"
    />

    <!-- Rename Dialog -->
    <v-dialog v-model="showRenameDialog" max-width="400">
      <v-card>
        <v-card-title>Rename Conversation</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="renameTitle"
            label="Title"
            variant="outlined"
            @keydown.enter="confirmRename"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showRenameDialog = false">Cancel</v-btn>
          <v-btn color="primary" @click="confirmRename">Rename</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useAiChatStore, type Provider } from '@/stores/aiChat'
import SessionList from './SessionList.vue'
import ChatMessage from './ChatMessage.vue'
import ChatInput from './ChatInput.vue'
import ModelSettings from './ModelSettings.vue'
import ModelSelector from './ModelSelector.vue'

const store = useAiChatStore()

// UI State
const showSidebar = ref(true)
const railMode = ref(false)
const showSettings = ref(false)
const showRenameDialog = ref(false)
const renameTitle = ref('')
const renameSessionId = ref<string | null>(null)
const messagesContainer = ref<HTMLElement | null>(null)
const chatInputRef = ref<InstanceType<typeof ChatInput> | null>(null)
const showScrollButton = ref(false)

// Model selection
const selectedModelId = ref<string | null>(null)

const selectedModel = computed(() => {
  if (!selectedModelId.value) return null
  return store.allModels.find(m => m.id === selectedModelId.value)
})

// Quick actions
const quickActions = [
  { title: 'Explain a concept', icon: 'mdi-lightbulb', prompt: 'Explain the concept of ' },
  { title: 'Write code', icon: 'mdi-code-braces', prompt: 'Write code to ' },
  { title: 'Summarize text', icon: 'mdi-text-box', prompt: 'Summarize the following text:\n\n' },
  { title: 'Brainstorm ideas', icon: 'mdi-brain', prompt: 'Help me brainstorm ideas for ' },
]

// Initialize store
onMounted(() => {
  store.initialize()
})

// Watch for new messages to scroll
watch(
  () => store.currentMessages.length,
  () => {
    nextTick(() => {
      scrollToBottom()
    })
  }
)

// Scroll handling
function handleScroll() {
  if (!messagesContainer.value) return
  const { scrollTop, scrollHeight, clientHeight } = messagesContainer.value
  showScrollButton.value = scrollHeight - scrollTop - clientHeight > 100
}

function scrollToBottom() {
  if (!messagesContainer.value) return
  messagesContainer.value.scrollTo({
    top: messagesContainer.value.scrollHeight,
    behavior: 'smooth',
  })
}

// Session handlers
function handleNewSession() {
  store.createSession()
  chatInputRef.value?.focus()
}

function handleSelectSession(sessionId: string) {
  store.selectSession(sessionId)
}

function handleRenameSession(sessionId: string) {
  const session = store.sessions.find(s => s.id === sessionId)
  if (session) {
    renameSessionId.value = sessionId
    renameTitle.value = session.title
    showRenameDialog.value = true
  }
}

function confirmRename() {
  if (renameSessionId.value && renameTitle.value.trim()) {
    store.updateSessionTitle(renameSessionId.value, renameTitle.value.trim())
    showRenameDialog.value = false
  }
}

function handleExportSession(sessionId: string) {
  const session = store.sessions.find(s => s.id === sessionId)
  if (!session) return

  const content = session.messages
    .map(m => `## ${m.role === 'user' ? 'You' : 'Assistant'}\n\n${m.content}`)
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
  if (confirm('Are you sure you want to delete this conversation?')) {
    store.deleteSession(sessionId)
  }
}

// Message handlers
async function handleSendMessage(content: string) {
  if (!content.trim()) return
  
  // Create session if needed
  if (!store.currentSessionId) {
    store.createSession()
  }
  
  await store.sendMessage(content)
}

function handleStopGeneration() {
  // TODO: Implement stop generation
  console.log('Stop generation')
}

function handleRegenerate(message: any) {
  // TODO: Implement regenerate
  console.log('Regenerate:', message.id)
}

// Quick action handler
function handleQuickAction(action: typeof quickActions[0]) {
  if (!store.currentSessionId) {
    store.createSession()
  }
  chatInputRef.value?.setText(action.prompt)
}

// Settings handlers
async function handleSaveProvider(provider: Partial<Provider>) {
  try {
    await store.saveApiKey(provider.name, provider.apiKey, provider.baseUrl)
  } catch (e) {
    console.error('Failed to save provider:', e)
  }
}

async function handleDeleteProvider(providerId: string) {
  try {
    await store.deleteApiKey(providerId)
  } catch (e) {
    console.error('Failed to delete provider:', e)
  }
}

function handleUpdateSettings(settings: any) {
  store.updateSettings(settings)
}

// Add scroll listener
watch(messagesContainer, (el) => {
  if (el) {
    el.addEventListener('scroll', handleScroll)
  }
})
</script>

<style scoped>
.ai-chat-page {
  display: flex;
  height: 100vh;
  background: rgb(var(--v-theme-background));
}

.chat-sidebar {
  border-right: 1px solid rgba(var(--v-theme-on-surface), 0.1);
}

.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.chat-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.chat-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 24px;
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.1);
  background: rgba(var(--v-theme-background), 0.8);
  backdrop-filter: blur(10px);
}

.header-title {
  flex: 1;
  min-width: 0;
}

.header-title h1 {
  font-size: 1.25rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.model-selector {
  min-width: 200px;
}

.messages-area {
  flex: 1;
  overflow-y: auto;
  position: relative;
  display: flex;
  flex-direction: column;
}

.welcome-screen {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 48px;
}

.welcome-content {
  text-align: center;
  max-width: 600px;
}

.welcome-icon {
  margin-bottom: 24px;
}

.welcome-content h2 {
  font-size: 2rem;
  font-weight: 600;
  margin-bottom: 8px;
}

.welcome-content p {
  color: rgba(var(--v-theme-on-surface), 0.6);
  margin-bottom: 32px;
}

.quick-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  justify-content: center;
}

.quick-action-btn {
  border-radius: 12px;
  text-transform: none;
  letter-spacing: 0;
}

.messages-list {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.scroll-to-bottom {
  position: absolute;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10;
}

/* Message animations */
.message-enter-active {
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Scrollbar styling */
.messages-area::-webkit-scrollbar {
  width: 8px;
}

.messages-area::-webkit-scrollbar-track {
  background: transparent;
}

.messages-area::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-on-surface), 0.2);
  border-radius: 4px;
}

.messages-area::-webkit-scrollbar-thumb:hover {
  background: rgba(var(--v-theme-on-surface), 0.3);
}

/* Mobile responsive */
@media (max-width: 960px) {
  .chat-header {
    padding: 12px 16px;
  }

  .model-selector {
    display: none;
  }

  .welcome-content h2 {
    font-size: 1.5rem;
  }

  .quick-actions {
    flex-direction: column;
  }

  .quick-action-btn {
    width: 100%;
  }
}

/* Dark mode */
:global(.v-theme--dark) .chat-header {
  background: rgba(0, 0, 0, 0.3);
}
</style>