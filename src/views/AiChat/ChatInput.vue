<template>
  <div class="chat-input-wrapper">
    <div class="input-container">
      <!-- Main Input Area -->
      <div class="input-main">
        <v-textarea
          ref="inputRef"
          v-model="inputText"
          :placeholder="placeholder"
          :disabled="disabled || isLoading"
          :rows="1"
          :max-rows="8"
          auto-grow
          hide-details
          variant="solo"
          class="chat-textarea"
          @keydown="handleKeydown"
          @paste="handlePaste"
        >
          <template v-slot:prepend-inner>
            <v-btn
              icon
              variant="text"
              size="small"
              class="attach-btn"
              @click="showAttachMenu = !showAttachMenu"
            >
              <v-icon icon="mdi-plus" />
              <v-menu
                v-model="showAttachMenu"
                activator="parent"
                location="top"
              >
                <v-list density="compact">
                  <v-list-item @click="$emit('attach-image')">
                    <template v-slot:prepend>
                      <v-icon icon="mdi-image" />
                    </template>
                    <v-list-item-title>Attach Image</v-list-item-title>
                  </v-list-item>
                  <v-list-item @click="$emit('attach-file')">
                    <template v-slot:prepend>
                      <v-icon icon="mdi-file-document" />
                    </template>
                    <v-list-item-title>Attach File</v-list-item-title>
                  </v-list-item>
                </v-list>
              </v-menu>
            </v-btn>
          </template>

          <template v-slot:append-inner>
            <div class="input-actions">
              <v-btn
                v-if="inputText.trim()"
                icon
                :color="canSend ? 'primary' : 'default'"
                :disabled="!canSend"
                size="small"
                class="send-btn"
                @click="handleSend"
              >
                <v-icon icon="mdi-send" size="20" />
                <v-tooltip activator="parent" location="top">
                  Send (Enter)
                </v-tooltip>
              </v-btn>
              
              <v-btn
                v-else-if="isLoading"
                icon
                color="error"
                size="small"
                class="stop-btn"
                @click="$emit('stop')"
              >
                <v-icon icon="mdi-stop" size="20" />
                <v-tooltip activator="parent" location="top">
                  Stop Generation
                </v-tooltip>
              </v-btn>

              <v-btn
                v-else
                icon
                disabled
                size="small"
                class="send-btn disabled"
              >
                <v-icon icon="mdi-send" size="20" />
              </v-btn>
            </div>
          </template>
        </v-textarea>
      </div>

      <!-- Bottom Bar -->
      <div class="input-footer">
        <div class="footer-left">
          <v-chip
            v-if="selectedModel"
            size="small"
            variant="outlined"
            class="model-chip"
          >
            <v-icon icon="mdi-brain" start size="16" />
            {{ selectedModel.name }}
          </v-chip>

          <v-chip
            v-if="streamEnabled"
            size="small"
            variant="text"
            class="stream-chip"
          >
            <v-icon icon="mdi-lightning-bolt" start size="16" />
            Stream
          </v-chip>
        </div>

        <div class="footer-right">
          <span class="char-count" :class="{ 'over-limit': charCount > maxChars }">
            {{ charCount }}/{{ maxChars }}
          </span>
          <span class="shortcut-hint">
            <kbd>Enter</kbd> to send, <kbd>Shift+Enter</kbd> for new line
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted } from 'vue'
import type { Model } from '@/stores/aiChat'

const props = defineProps<{
  disabled?: boolean
  isLoading?: boolean
  selectedModel?: Model | null
  streamEnabled?: boolean
  maxChars?: number
  placeholder?: string
}>()

const emit = defineEmits<{
  (e: 'send', content: string): void
  (e: 'stop'): void
  (e: 'attach-image'): void
  (e: 'attach-file'): void
}>()

const inputRef = ref<any>(null)
const inputText = ref('')
const showAttachMenu = ref(false)

const maxChars = computed(() => props.maxChars || 4000)
const placeholder = computed(() => props.placeholder || 'Type your message...')

const charCount = computed(() => inputText.value.length)

const canSend = computed(() => {
  return inputText.value.trim() && 
         !props.disabled && 
         !props.isLoading &&
         charCount.value <= maxChars.value
})

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    if (canSend.value) {
      handleSend()
    }
  }
}

function handlePaste(event: ClipboardEvent) {
  // Handle image paste
  const items = event.clipboardData?.items
  if (items) {
    for (const item of items) {
      if (item.type.startsWith('image/')) {
        event.preventDefault()
        emit('attach-image')
        return
      }
    }
  }
}

function handleSend() {
  if (!canSend.value) return
  
  const content = inputText.value.trim()
  inputText.value = ''
  emit('send', content)
  
  // Refocus input
  nextTick(() => {
    inputRef.value?.focus()
  })
}

function focus() {
  inputRef.value?.focus()
}

function setText(text: string) {
  inputText.value = text
  nextTick(() => {
    focus()
  })
}

onMounted(() => {
  focus()
})

defineExpose({ focus, setText })
</script>

<style scoped>
.chat-input-wrapper {
  padding: 16px 24px 24px;
  background: linear-gradient(to top, rgba(var(--v-theme-background), 1) 80%, transparent);
}

.input-container {
  max-width: 900px;
  margin: 0 auto;
}

.input-main {
  position: relative;
}

.chat-textarea {
  border-radius: 24px !important;
  overflow: hidden;
}

.chat-textarea :deep(.v-field) {
  border-radius: 24px;
  padding: 12px 16px;
  background: rgba(var(--v-theme-surface-variant), 0.5);
  border: 1px solid rgba(var(--v-theme-on-surface), 0.1);
  transition: all 0.2s ease;
}

.chat-textarea :deep(.v-field:hover) {
  border-color: rgba(var(--v-theme-on-surface), 0.2);
}

.chat-textarea :deep(.v-field--focused) {
  border-color: rgb(var(--v-theme-primary));
  box-shadow: 0 0 0 2px rgba(var(--v-theme-primary), 0.2);
}

.chat-textarea :deep(textarea) {
  font-size: 1rem;
  line-height: 1.5;
  max-height: 200px;
  overflow-y: auto;
}

.chat-textarea :deep(textarea::placeholder) {
  color: rgba(var(--v-theme-on-surface), 0.5);
}

.attach-btn {
  margin-left: -8px;
}

.input-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-right: -8px;
}

.send-btn {
  transition: all 0.2s ease;
}

.send-btn:not(.disabled):hover {
  transform: scale(1.1);
}

.stop-btn {
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.1); }
}

.input-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
  padding: 0 8px;
}

.footer-left {
  display: flex;
  gap: 8px;
}

.model-chip {
  font-size: 0.75rem;
}

.stream-chip {
  font-size: 0.75rem;
  opacity: 0.6;
}

.footer-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.char-count {
  font-size: 0.75rem;
  color: rgba(var(--v-theme-on-surface), 0.5);
}

.char-count.over-limit {
  color: rgb(var(--v-theme-error));
}

.shortcut-hint {
  font-size: 0.75rem;
  color: rgba(var(--v-theme-on-surface), 0.4);
}

.shortcut-hint kbd {
  display: inline-block;
  padding: 2px 6px;
  font-size: 0.7rem;
  font-family: inherit;
  background: rgba(var(--v-theme-surface-variant), 0.5);
  border-radius: 4px;
  border: 1px solid rgba(var(--v-theme-on-surface), 0.1);
}

/* Dark mode adjustments */
:global(.v-theme--dark) .chat-textarea :deep(.v-field) {
  background: rgba(255, 255, 255, 0.05);
}

:global(.v-theme--dark) .shortcut-hint kbd {
  background: rgba(255, 255, 255, 0.1);
}

/* Mobile responsive */
@media (max-width: 600px) {
  .chat-input-wrapper {
    padding: 12px 16px 16px;
  }

  .input-footer {
    flex-direction: column;
    gap: 8px;
  }

  .footer-right {
    width: 100%;
    justify-content: space-between;
  }

  .shortcut-hint {
    display: none;
  }
}
</style>
