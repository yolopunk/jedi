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
  background: rgba(10, 10, 15, 0.98);
  border: 1px solid rgba(0, 255, 255, 0.15);
  border-radius: 2px;
  overflow: hidden;
  box-shadow:
    0 0 0 1px rgba(0, 0, 0, 0.8),
    0 8px 32px rgba(0, 0, 0, 0.6),
    0 0 40px rgba(0, 255, 255, 0.05);
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
  background: linear-gradient(180deg, #00ffff, #00ff88);
  box-shadow: 0 0 12px rgba(0, 255, 255, 0.4);
}

/* Type-specific styles */
.console-snackbar.type-success .snackbar-accent {
  background: linear-gradient(180deg, #00ff88, #00cc6a);
  box-shadow: 0 0 12px rgba(0, 255, 136, 0.4);
}

.console-snackbar.type-error .snackbar-accent {
  background: linear-gradient(180deg, #ff4444, #cc3333);
  box-shadow: 0 0 12px rgba(255, 68, 68, 0.4);
}

.console-snackbar.type-warning .snackbar-accent {
  background: linear-gradient(180deg, #ffaa00, #cc8800);
  box-shadow: 0 0 12px rgba(255, 170, 0, 0.4);
}

.console-snackbar.type-info .snackbar-accent {
  background: linear-gradient(180deg, #00aaff, #0088cc);
  box-shadow: 0 0 12px rgba(0, 170, 255, 0.4);
}

.snackbar-main {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: linear-gradient(
    135deg,
    rgba(15, 15, 26, 0.98) 0%,
    rgba(10, 10, 15, 0.98) 100%
  );
}

.snackbar-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: rgba(0, 255, 255, 0.05);
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 2px;
}

.snackbar-icon {
  font-size: 14px;
  font-weight: bold;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  color: #00ffff;
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.6);
}

/* Type-specific icon colors */
.console-snackbar.type-success .snackbar-icon-wrapper {
  border-color: rgba(0, 255, 136, 0.3);
  background: rgba(0, 255, 136, 0.05);
}

.console-snackbar.type-success .snackbar-icon {
  color: #00ff88;
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.6);
}

.console-snackbar.type-error .snackbar-icon-wrapper {
  border-color: rgba(255, 68, 68, 0.3);
  background: rgba(255, 68, 68, 0.05);
}

.console-snackbar.type-error .snackbar-icon {
  color: #ff4444;
  text-shadow: 0 0 10px rgba(255, 68, 68, 0.6);
}

.console-snackbar.type-warning .snackbar-icon-wrapper {
  border-color: rgba(255, 170, 0, 0.3);
  background: rgba(255, 170, 0, 0.05);
}

.console-snackbar.type-warning .snackbar-icon {
  color: #ffaa00;
  text-shadow: 0 0 10px rgba(255, 170, 0, 0.6);
}

.console-snackbar.type-info .snackbar-icon-wrapper {
  border-color: rgba(0, 170, 255, 0.3);
  background: rgba(0, 170, 255, 0.05);
}

.console-snackbar.type-info .snackbar-icon {
  color: #00aaff;
  text-shadow: 0 0 10px rgba(0, 170, 255, 0.6);
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
  color: #00ffff;
  margin-bottom: 2px;
  opacity: 0.9;
}

.console-snackbar.type-success .snackbar-label {
  color: #00ff88;
}

.console-snackbar.type-error .snackbar-label {
  color: #ff4444;
}

.console-snackbar.type-warning .snackbar-label {
  color: #ffaa00;
}

.console-snackbar.type-info .snackbar-label {
  color: #00aaff;
}

.snackbar-text {
  font-size: 12px;
  line-height: 1.4;
  font-family: var(--jedi-font-ui);
  color: rgba(244, 244, 245, 0.9);
}

.snackbar-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  background: transparent;
  border: none;
  border-left: 1px solid rgba(255, 255, 255, 0.08);
  color: rgba(161, 161, 170, 0.6);
  cursor: pointer;
  transition: all 0.15s ease;
}

.snackbar-close:hover {
  background: rgba(255, 255, 255, 0.05);
  color: rgba(244, 244, 245, 0.9);
}

.close-icon {
  font-size: 12px;
  font-weight: bold;
  font-family: 'JetBrains Mono', monospace;
}

/* =========================================
   Light Theme Styles (Tatooine Outpost)
   ========================================= */
.light-theme .console-snackbar {
  background: rgba(245, 230, 211, 0.98);
  border-color: rgba(184, 134, 11, 0.3);
  box-shadow:
    0 0 0 1px rgba(184, 134, 11, 0.2),
    0 8px 32px rgba(107, 68, 35, 0.2),
    0 0 40px rgba(184, 134, 11, 0.1);
}

.light-theme .snackbar-accent {
  background: linear-gradient(180deg, #cd7f32, #b8860b);
  box-shadow: 0 0 12px rgba(205, 127, 50, 0.4);
}

.light-theme .console-snackbar.type-success .snackbar-accent {
  background: linear-gradient(180deg, #daa520, #b8860b);
  box-shadow: 0 0 12px rgba(218, 165, 32, 0.4);
}

.light-theme .console-snackbar.type-error .snackbar-accent {
  background: linear-gradient(180deg, #b22222, #8b0000);
  box-shadow: 0 0 12px rgba(178, 34, 34, 0.4);
}

.light-theme .console-snackbar.type-warning .snackbar-accent {
  background: linear-gradient(180deg, #cd853f, #b87333);
  box-shadow: 0 0 12px rgba(205, 133, 63, 0.4);
}

.light-theme .console-snackbar.type-info .snackbar-accent {
  background: linear-gradient(180deg, #cd7f32, #b8860b);
  box-shadow: 0 0 12px rgba(205, 127, 50, 0.4);
}

.light-theme .snackbar-main {
  background: linear-gradient(
    135deg,
    rgba(239, 224, 204, 0.98) 0%,
    rgba(232, 212, 188, 0.98) 100%
  );
}

.light-theme .snackbar-icon-wrapper {
  background: rgba(205, 127, 50, 0.1);
  border-color: rgba(205, 127, 50, 0.3);
}

.light-theme .snackbar-icon {
  color: #cd7f32;
  text-shadow: 0 0 10px rgba(205, 127, 50, 0.4);
}

.light-theme .console-snackbar.type-success .snackbar-icon-wrapper {
  border-color: rgba(218, 165, 32, 0.3);
  background: rgba(218, 165, 32, 0.1);
}

.light-theme .console-snackbar.type-success .snackbar-icon {
  color: #daa520;
  text-shadow: 0 0 10px rgba(218, 165, 32, 0.4);
}

.light-theme .console-snackbar.type-error .snackbar-icon-wrapper {
  border-color: rgba(178, 34, 34, 0.3);
  background: rgba(178, 34, 34, 0.1);
}

.light-theme .console-snackbar.type-error .snackbar-icon {
  color: #b22222;
  text-shadow: 0 0 10px rgba(178, 34, 34, 0.4);
}

.light-theme .console-snackbar.type-warning .snackbar-icon-wrapper {
  border-color: rgba(205, 133, 63, 0.3);
  background: rgba(205, 133, 63, 0.1);
}

.light-theme .console-snackbar.type-warning .snackbar-icon {
  color: #cd853f;
  text-shadow: 0 0 10px rgba(205, 133, 63, 0.4);
}

.light-theme .console-snackbar.type-info .snackbar-icon-wrapper {
  border-color: rgba(205, 127, 50, 0.3);
  background: rgba(205, 127, 50, 0.1);
}

.light-theme .console-snackbar.type-info .snackbar-icon {
  color: #cd7f32;
  text-shadow: 0 0 10px rgba(205, 127, 50, 0.4);
}

.light-theme .snackbar-label {
  color: #cd7f32;
}

.light-theme .console-snackbar.type-success .snackbar-label {
  color: #daa520;
}

.light-theme .console-snackbar.type-error .snackbar-label {
  color: #b22222;
}

.light-theme .console-snackbar.type-warning .snackbar-label {
  color: #cd853f;
}

.light-theme .console-snackbar.type-info .snackbar-label {
  color: #cd7f32;
}

.light-theme .snackbar-text {
  color: #3d2914;
}

.light-theme .snackbar-close {
  border-left-color: rgba(184, 134, 11, 0.2);
  color: rgba(107, 68, 35, 0.6);
}

.light-theme .snackbar-close:hover {
  background: rgba(205, 127, 50, 0.1);
  color: #3d2914;
}
</style>
