<template>
  <div class="chat-input-wrapper">
    <div class="input-container">
      <!-- Unified Input Bar -->
      <div class="input-bar" :class="{ 'is-loading': isLoading }">
        <!-- Attach Button -->
        <v-btn
          icon
          variant="text"
          size="small"
          class="attach-btn"
          :disabled="disabled || isLoading"
          @click="showAttachMenu = !showAttachMenu"
        >
          <v-icon icon="mdi-plus" size="20" />
          <v-menu
            v-model="showAttachMenu"
            activator="parent"
            location="top start"
          >
            <v-list density="compact" min-width="160">
              <v-list-item @click="$emit('attach-image')">
                <template v-slot:prepend>
                  <v-icon icon="mdi-image" size="18" />
                </template>
                <v-list-item-title>Attach Image</v-list-item-title>
              </v-list-item>
              <v-list-item @click="$emit('attach-file')">
                <template v-slot:prepend>
                  <v-icon icon="mdi-file-document" size="18" />
                </template>
                <v-list-item-title>Attach File</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-menu>
        </v-btn>

        <!-- Text Input -->
        <v-textarea
          ref="inputRef"
          v-model="inputText"
          :placeholder="placeholder"
          :disabled="disabled || isLoading"
          :rows="1"
          :max-rows="6"
          auto-grow
          hide-details
          variant="plain"
          class="chat-textarea"
          @keydown="handleKeydown"
          @paste="handlePaste"
        />

        <!-- Send / Stop Button -->
        <v-btn
          v-if="isLoading"
          icon
          variant="flat"
          size="small"
          color="error"
          class="action-btn stop-btn"
          @click="$emit('stop')"
        >
          <v-icon icon="mdi-stop" size="20" />
          <v-tooltip activator="parent" location="top">Stop</v-tooltip>
        </v-btn>
        <v-btn
          v-else
          icon
          variant="flat"
          size="small"
          :color="canSend ? 'primary' : 'default'"
          :disabled="!canSend"
          class="action-btn send-btn"
          @click="handleSend"
        >
          <v-icon icon="mdi-send" size="20" />
          <v-tooltip activator="parent" location="top">Send (Enter)</v-tooltip>
        </v-btn>
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
            <v-icon icon="mdi-brain" start size="14" />
            {{ selectedModel.name }}
          </v-chip>

          <v-chip
            v-if="streamEnabled"
            size="small"
            variant="text"
            class="stream-chip"
          >
            <v-icon icon="mdi-lightning-bolt" start size="14" />
            Stream
          </v-chip>
        </div>

        <div class="footer-right">
          <span class="char-count" :class="{ 'over-limit': charCount > maxChars }">
            {{ charCount }}/{{ maxChars }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import type { ModelsDevModel } from '@/types/modelsDev'

const props = defineProps<{
  disabled?: boolean
  isLoading?: boolean
  selectedModel?: ModelsDevModel | null
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

const inputRef = ref<{ focus: () => void } | null>(null)
const inputText = ref('')
const _showAttachMenu = ref(false)

const maxChars = computed(() => props.maxChars || 4000)
const _placeholder = computed(() => props.placeholder || 'Type your message...')

const charCount = computed(() => inputText.value.length)

const canSend = computed(() => {
  return (
    inputText.value.trim() &&
    !props.disabled &&
    !props.isLoading &&
    charCount.value <= maxChars.value
  )
})

function _handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    if (canSend.value) {
      handleSend()
    }
  }
}

function _handlePaste(event: ClipboardEvent) {
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
  padding: 12px 20px 20px;
  background: linear-gradient(to top, rgba(var(--v-theme-background), 1) 80%, transparent));
}

.input-container {
  max-width: 900px;
  margin: 0 auto;
}

/* Unified Input Bar - Telegram style */
.input-bar {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  padding: 8px 8px 8px 12px;
  background: rgba(var(--v-theme-surface-variant), 0.6);
  border: 1px solid rgba(var(--v-theme-on-surface), 0.12);
  border-radius: 24px;
  transition: all 0.2s ease;
}

.input-bar:focus-within {
  border-color: rgba(var(--v-theme-primary), 0.5);
  box-shadow: 0 0 0 3px rgba(var(--v-theme-primary), 0.1);
}

.input-bar.is-loading {
  background: rgba(var(--v-theme-surface-variant), 0.4);
}

/* Attach Button */
.attach-btn {
  flex-shrink: 0;
  margin-right: 4px;
  color: rgba(var(--v-theme-on-surface), 0.6);
}

.attach-btn:hover {
  color: rgb(var(--v-theme-primary));
}

/* Text Input */
.chat-textarea {
  flex: 1;
  min-width: 0;
}

.chat-textarea :deep(textarea) {
  font-size: 0.95rem;
  line-height: 1.5;
  max-height: 150px;
  overflow-y: auto;
  padding: 0 !important;
  margin: 0 !important;
}

.chat-textarea :deep(textarea::placeholder) {
  color: rgba(var(--v-theme-on-surface), 0.4);
}

/* Remove default textarea styling */
.chat-textarea :deep(.v-field) {
  background: transparent !important;
  padding: 0;
}

.chat-textarea :deep(.v-field__input) {
  padding: 0;
  min-height: unset;
}

/* Action Buttons */
.action-btn {
  flex-shrink: 0;
  margin-left: 4px;
}

.send-btn {
  transition: all 0.2s ease;
}

.send-btn:not(:disabled):hover {
  transform: scale(1.08);
}

.stop-btn {
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

/* Bottom Bar */
.input-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
  padding: 0 16px;
}

.footer-left {
  display: flex;
  gap: 8px;
}

.model-chip {
  font-size: 0.7rem;
  height: 24px;
}

.stream-chip {
  font-size: 0.7rem;
  opacity: 0.6;
  height: 24px;
}

.footer-right {
  display: flex;
  align-items: center;
}

.char-count {
  font-size: 0.7rem;
  color: rgba(var(--v-theme-on-surface), 0.4);
}

.char-count.over-limit {
  color: rgb(var(--v-theme-error));
}

/* Dark mode adjustments */
:global(.v-theme--dark) .input-bar {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.1);
}

:global(.v-theme--dark) .input-bar:focus-within {
  border-color: rgba(var(--v-theme-primary), 0.6);
  box-shadow: 0 0 0 3px rgba(var(--v-theme-primary), 0.15);
}

/* Mobile responsive */
@media (max-width: 600px) {
  .chat-input-wrapper {
    padding: 10px 14px 14px;
  }

  .input-bar {
    padding: 6px 6px 6px 10px;
    border-radius: 20px;
  }

  .input-footer {
    padding: 0 10px;
  }

  .footer-left {
    gap: 6px;
  }

  .model-chip,
  .stream-chip {
    font-size: 0.65rem;
    height: 22px;
  }
}
</style>
