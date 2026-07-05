<template>
  <div 
    :class="[
      'message-wrapper',
      message.role,
      { 'streaming': message.isStreaming }
    ]"
  >
    <div class="message-container">
      <!-- Avatar -->
      <div class="message-avatar">
        <v-avatar 
          :size="36"
          :color="message.role === 'user' ? 'primary' : 'secondary'"
        >
          <v-icon 
            :icon="message.role === 'user' ? 'mdi-account' : 'mdi-robot'"
            size="20"
          />
        </v-avatar>
      </div>

      <!-- Content -->
      <div class="message-content">
        <!-- Role Label -->
        <div class="message-role">
          {{ message.role === 'user' ? 'You' : 'Assistant' }}
        </div>

        <!-- Message Body -->
        <div class="message-body">
          <div 
            v-if="message.error"
            class="message-error"
          >
            <v-icon icon="mdi-alert-circle" color="error" size="18" class="mr-2" />
            {{ message.error }}
          </div>
          
          <div 
            v-else
            class="message-text prose"
            v-html="renderedContent"
          />

          <!-- Streaming indicator -->
          <div 
            v-if="message.isStreaming && !message.content"
            class="streaming-indicator"
          >
            <span class="dot"></span>
            <span class="dot"></span>
            <span class="dot"></span>
          </div>
        </div>

        <!-- Actions -->
        <div 
          v-if="!message.isStreaming && message.content"
          class="message-actions"
        >
          <v-btn
            icon
            variant="text"
            size="small"
            @click="copyToClipboard"
            class="action-btn"
          >
            <v-icon icon="mdi-content-copy" size="18" />
            <v-tooltip activator="parent" location="top">
              {{ copied ? 'Copied!' : 'Copy' }}
            </v-tooltip>
          </v-btn>

          <v-btn
            v-if="message.role === 'assistant'"
            icon
            variant="text"
            size="small"
            @click="$emit('regenerate')"
            class="action-btn"
          >
            <v-icon icon="mdi-refresh" size="18" />
            <v-tooltip activator="parent" location="top">
              Regenerate
            </v-tooltip>
          </v-btn>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Message } from '@/stores/aiChat'
import { renderSafe, sharedMd } from '@/utils/markdown'

const props = defineProps<{
  message: Message
}>()

defineEmits<(e: 'regenerate') => void>()

const copied = ref(false)

const renderedContent = computed(() => {
  if (!props.message.content) return ''
  return renderSafe(sharedMd, props.message.content)
})

async function copyToClipboard() {
  try {
    await navigator.clipboard.writeText(props.message.content)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (e) {
    console.error('Failed to copy:', e)
  }
}
</script>

<style scoped>
.message-wrapper {
  display: flex;
  padding: 16px 24px;
  transition: background-color 0.2s ease;
}

.message-wrapper:hover {
  background-color: rgba(var(--v-theme-surface-variant), 0.3);
}

.message-wrapper.user {
  background-color: rgba(var(--v-theme-primary), 0.05);
}

.message-wrapper.streaming {
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.8; }
}

.message-container {
  display: flex;
  gap: 16px;
  max-width: 900px;
  width: 100%;
  margin: 0 auto;
}

.message-avatar {
  flex-shrink: 0;
}

.message-content {
  flex: 1;
  min-width: 0;
}

.message-role {
  font-size: 0.875rem;
  font-weight: 600;
  color: rgba(var(--v-theme-on-surface), 0.7);
  margin-bottom: 4px;
}

.message-body {
  position: relative;
}

.message-text {
  line-height: 1.6;
  word-wrap: break-word;
}

.message-text :deep(p) {
  margin: 0 0 12px 0;
}

.message-text :deep(p:last-child) {
  margin-bottom: 0;
}

.message-text :deep(pre) {
  background: rgba(var(--v-theme-surface-variant), 0.8);
  border-radius: 8px;
  padding: 16px;
  overflow-x: auto;
  margin: 12px 0;
  position: relative;
}

.message-text :deep(code) {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.875rem;
}

.message-text :deep(pre code) {
  background: transparent;
  padding: 0;
}

.message-text :deep(:not(pre) > code) {
  background: rgba(var(--v-theme-primary), 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.875em;
}

.message-text :deep(ul),
.message-text :deep(ol) {
  padding-left: 24px;
  margin: 8px 0;
}

.message-text :deep(li) {
  margin: 4px 0;
}

.message-text :deep(blockquote) {
  border-left: 4px solid rgb(var(--v-theme-primary));
  margin: 12px 0;
  padding-left: 16px;
  color: rgba(var(--v-theme-on-surface), 0.7);
}

.message-text :deep(a) {
  color: rgb(var(--v-theme-primary));
  text-decoration: none;
}

.message-text :deep(a:hover) {
  text-decoration: underline;
}

.message-text :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 12px 0;
}

.message-text :deep(th),
.message-text :deep(td) {
  border: 1px solid rgba(var(--v-theme-on-surface), 0.1);
  padding: 8px 12px;
  text-align: left;
}

.message-text :deep(th) {
  background: rgba(var(--v-theme-surface-variant), 0.5);
  font-weight: 600;
}

.message-error {
  display: flex;
  align-items: center;
  color: rgb(var(--v-theme-error));
  padding: 12px;
  background: rgba(var(--v-theme-error), 0.1);
  border-radius: 8px;
}

.streaming-indicator {
  display: flex;
  gap: 4px;
  padding: 8px 0;
}

.streaming-indicator .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: rgb(var(--v-theme-primary));
  animation: bounce 1.4s infinite ease-in-out;
}

.streaming-indicator .dot:nth-child(1) {
  animation-delay: -0.32s;
}

.streaming-indicator .dot:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes bounce {
  0%, 80%, 100% {
    transform: scale(0);
  }
  40% {
    transform: scale(1);
  }
}

.message-actions {
  display: flex;
  gap: 4px;
  margin-top: 8px;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.message-wrapper:hover .message-actions {
  opacity: 1;
}

.action-btn {
  opacity: 0.6;
}

.action-btn:hover {
  opacity: 1;
}

/* Highlight.js theme overrides for dark/light mode */
:global(.v-theme--dark) .message-text :deep(pre) {
  background:rgb(var(--ink-rgb) / 0.3);
}

:global(.v-theme--dark) .message-text :deep(:not(pre) > code) {
  background:rgb(var(--ink-rgb) / 0.1);
}
</style>
