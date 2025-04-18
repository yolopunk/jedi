<template>
  <!-- 全局开关悬浮按钮 -->
  <v-btn
    :color="modelValue ? 'var(--jedi-success)' : 'var(--jedi-grey-medium)'"
    class="global-switch-fab"
    size="large"
    icon
    elevation="4"
    @click="$emit('update:modelValue', !modelValue)"
  >
    <v-tooltip
      activator="parent"
      location="top"
      :text="modelValue ? '已启用 - 点击禁用' : '已禁用 - 点击启用'"
    ></v-tooltip>

    <v-icon :icon="modelValue ? mdiPowerPlugOutline : mdiPowerPlugOffOutline" size="large"
           :class="{'active-icon': modelValue}"></v-icon>

    <!-- 状态指示器 -->
    <div class="status-indicator" :class="{ 'active': modelValue }">
      <span class="status-text">{{ modelValue ? '已启用' : '已禁用' }}</span>
    </div>
  </v-btn>
</template>

<script setup lang="ts">
import { mdiPowerPlugOutline, mdiPowerPlugOffOutline } from '@mdi/js'

// 定义组件属性
defineProps<{
  modelValue: boolean;
}>()

// 定义组件事件
defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>()
</script>

<style scoped>
.global-switch-fab {
  position: fixed;
  bottom: 40px;
  right: 40px;
  z-index: 100;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  border: 2px solid rgba(255, 255, 255, 0.2);
  overflow: visible;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(76, 175, 80, 0.4);
  }
  70% {
    box-shadow: 0 0 0 10px rgba(76, 175, 80, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(76, 175, 80, 0);
  }
}

.global-switch-fab:hover {
  transform: translateY(-4px) scale(1.05);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.15) !important;
  animation: none;
}

.global-switch-fab:active {
  transform: translateY(0) scale(0.98);
}

/* 状态指示器 */
.status-indicator {
  position: absolute;
  top: -8px;
  right: -8px;
  background-color: var(--jedi-grey-medium);
  color: white;
  border-radius: 12px;
  padding: 2px 8px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.5px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  transition: all 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.2);
  opacity: 0;
  transform: translateY(10px);
  pointer-events: none;
  min-width: 50px;
  text-align: center;
}

.global-switch-fab:hover .status-indicator {
  opacity: 1;
  transform: translateY(0);
}

.status-indicator.active {
  background-color: var(--jedi-success);
  animation: glow 1.5s infinite alternate;
}

@keyframes glow {
  from {
    box-shadow: 0 0 5px rgba(76, 175, 80, 0.5);
  }
  to {
    box-shadow: 0 0 10px rgba(76, 175, 80, 0.8);
  }
}

.status-text {
  white-space: nowrap;
}

.active-icon {
  animation: rotate 1s ease-in-out;
}

@keyframes rotate {
  0% {
    transform: rotate(0deg);
  }
  25% {
    transform: rotate(-30deg);
  }
  75% {
    transform: rotate(30deg);
  }
  100% {
    transform: rotate(0deg);
  }
}
</style>
