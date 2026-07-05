<template>
  <!-- 通知消息组件 - Sci-Fi Console Style -->
  <v-snackbar
    v-model="show"
    :scrim="false"
    :timeout="timeout"
    location="top"
    class="console-snackbar-root"
    :elevation="0"
    transition="snackbar-slide-transition"
  >
    <div class="console-snackbar" :class="`type-${type}`">
      <!-- Left accent border -->
      <div class="snackbar-accent"></div>

      <!-- Content -->
      <div class="snackbar-main">
        <!-- Icon with glow -->
        <div class="snackbar-icon-wrapper">
          <span class="snackbar-icon">{{ getIcon }}</span>
        </div>

        <!-- Message -->
        <div class="snackbar-message">
          <div class="snackbar-label">{{ getLabel }}</div>
          <div class="snackbar-text">{{ text }}</div>
        </div>
      </div>

      <!-- Close button -->
      <button class="snackbar-close" @click="show = false">
        <span class="close-icon">✕</span>
      </button>
    </div>
  </v-snackbar>
</template>

<script setup lang="ts">
import { computed } from 'vue'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean
  text: string
  color: 'success' | 'error' | 'info' | 'warning'
  timeout?: number
}>()

// 定义组件事件
const emit = defineEmits<(e: 'update:modelValue', value: boolean) => void>()

// 本地状态
const show = computed({
  get: () => props.modelValue,
  set: value => emit('update:modelValue', value),
})

const type = computed(() => props.color)

// 计算图标 - 使用 sci-fi 风格字符
const getIcon = computed(() => {
  switch (props.color) {
    case 'success':
      return '◆'
    case 'error':
      return '✕'
    case 'warning':
      return '!'
    default:
      return '›'
  }
})

const getLabel = computed(() => {
  switch (props.color) {
    case 'success':
      return 'SUCCESS'
    case 'error':
      return 'ERROR'
    case 'warning':
      return 'WARNING'
    default:
      return 'INFO'
  }
})
</script>

<style scoped>
.console-snackbar-root {
  padding: 0 !important;
  background: transparent !important;
  box-shadow: none !important;
  border-radius: 0 !important;
}

.console-snackbar-root :deep(.v-snackbar__wrapper) {
  padding: 0 !important;
  background: transparent !important;
  box-shadow: none !important;
  border-radius: 0 !important;
}

.console-snackbar-root :deep(.v-snackbar__scrim) {
  display: none !important;
  opacity: 0 !important;
  pointer-events: none !important;
}

.console-snackbar {
  position: relative;
  display: flex;
  align-items: stretch;
  min-width: 320px;
  max-width: 480px;
  background: rgb(var(--bg-rgb) / 0.98);
  border: 1px solid rgb(var(--accent-rgb) / 0.15);
  border-radius: 2px;
  overflow: hidden;
  box-shadow:
    0 0 0 1px rgba(0, 0, 0, 0.8),
    0 8px 32px rgba(0, 0, 0, 0.6),
    0 0 40px rgb(var(--accent-rgb) / 0.05);
  backdrop-filter: blur(12px);
  animation: snackbar-enter 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes snackbar-enter {
  0% {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }
  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.snackbar-accent {
  width: 3px;
  background: linear-gradient(180deg, var(--accent), var(--success));
  box-shadow: 0 0 12px rgb(var(--accent-rgb) / 0.4);
}

/* Type-specific styles */
.console-snackbar.type-success .snackbar-accent {
  background: linear-gradient(180deg, var(--success), var(--success));
  box-shadow: 0 0 12px rgb(var(--success-rgb) / 0.4);
}

.console-snackbar.type-error .snackbar-accent {
  background: linear-gradient(180deg, var(--danger), var(--danger));
  box-shadow: 0 0 12px rgb(var(--danger-rgb) / 0.4);
}

.console-snackbar.type-warning .snackbar-accent {
  background: linear-gradient(180deg, var(--warning), var(--warning));
  box-shadow: 0 0 12px rgb(var(--warning-rgb) / 0.4);
}

.console-snackbar.type-info .snackbar-accent {
  background: linear-gradient(180deg, var(--accent), var(--accent));
  box-shadow: 0 0 12px rgb(var(--accent-rgb) / 0.4);
}

.snackbar-main {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: linear-gradient(
    135deg,
    rgb(var(--bg-rgb) / 0.98) 0%,
    rgb(var(--bg-rgb) / 0.98) 100%
  );
}

.snackbar-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: rgb(var(--accent-rgb) / 0.05);
  border: 1px solid rgb(var(--accent-rgb) / 0.2);
  border-radius: 2px;
}

.snackbar-icon {
  font-size: 14px;
  font-weight: bold;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  color: var(--accent);
  text-shadow: 0 0 10px rgb(var(--accent-rgb) / 0.6);
}

/* Type-specific icon colors */
.console-snackbar.type-success .snackbar-icon-wrapper {
  border-color: rgb(var(--success-rgb) / 0.3);
  background: rgb(var(--success-rgb) / 0.05);
}

.console-snackbar.type-success .snackbar-icon {
  color: var(--success);
  text-shadow: 0 0 10px rgb(var(--success-rgb) / 0.6);
}

.console-snackbar.type-error .snackbar-icon-wrapper {
  border-color: rgb(var(--danger-rgb) / 0.3);
  background: rgb(var(--danger-rgb) / 0.05);
}

.console-snackbar.type-error .snackbar-icon {
  color: var(--danger);
  text-shadow: 0 0 10px rgb(var(--danger-rgb) / 0.6);
}

.console-snackbar.type-warning .snackbar-icon-wrapper {
  border-color: rgb(var(--warning-rgb) / 0.3);
  background: rgb(var(--warning-rgb) / 0.05);
}

.console-snackbar.type-warning .snackbar-icon {
  color: var(--warning);
  text-shadow: 0 0 10px rgb(var(--warning-rgb) / 0.6);
}

.console-snackbar.type-info .snackbar-icon-wrapper {
  border-color: rgb(var(--accent-rgb) / 0.3);
  background: rgb(var(--accent-rgb) / 0.05);
}

.console-snackbar.type-info .snackbar-icon {
  color: var(--accent);
  text-shadow: 0 0 10px rgb(var(--accent-rgb) / 0.6);
}

.snackbar-message {
  flex: 1;
  min-width: 0;
}

.snackbar-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 2px;
  text-transform: uppercase;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  color: var(--accent);
  margin-bottom: 2px;
  opacity: 0.9;
}

.console-snackbar.type-success .snackbar-label {
  color: var(--success);
}

.console-snackbar.type-error .snackbar-label {
  color: var(--danger);
}

.console-snackbar.type-warning .snackbar-label {
  color: var(--warning);
}

.console-snackbar.type-info .snackbar-label {
  color: var(--accent);
}

.snackbar-text {
  font-size: 12px;
  line-height: 1.4;
  font-family: var(--jedi-font-ui);
  color: rgb(var(--text-rgb) / 0.9);
}

.snackbar-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  background: transparent;
  border: none;
  border-left: 1px solid rgb(var(--text-rgb) / 0.08);
  color: rgba(161, 161, 170, 0.6);
  cursor: pointer;
  transition: all 0.15s ease;
}

.snackbar-close:hover {
  background:rgb(var(--ink-rgb) / 0.05);
  color: rgb(var(--text-rgb) / 0.9);
}

.close-icon {
  font-size: 12px;
  font-weight: bold;
  font-family: 'JetBrains Mono', monospace;
}
</style>
