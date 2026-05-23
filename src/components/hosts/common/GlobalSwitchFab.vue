<template>
  <!-- 全局开关悬浮按钮 -->
  <v-btn
    :color="modelValue ? 'success' : 'grey'"
    class="global-switch-fab"
    size="large"
    icon
    elevation="4"
    @click="$emit('update:modelValue', !modelValue)"
  >
    <v-icon :icon="modelValue ? mdiPowerPlugOutline : mdiPowerPlugOffOutline" size="large"
           :class="{'active-icon': modelValue}"></v-icon>

    <!-- 状态指示器 -->
    <div class="status-indicator" :class="{ 'active': modelValue }">
      <span class="status-text">{{ modelValue ? $t('hosts.globalSwitch.enabled') : $t('hosts.globalSwitch.disabled') }}</span>
    </div>
  </v-btn>
</template>

<script setup lang="ts">
import { mdiPowerPlugOutline, mdiPowerPlugOffOutline } from '@mdi/js'

// 定义组件属性
defineProps<{
  modelValue: boolean
}>()

// 定义组件事件
defineEmits<(e: 'update:modelValue', value: boolean) => void>()
</script>

<style scoped>
.global-switch-fab {
  position: fixed;
  bottom: 60px;
  right: 32px;
  z-index: 100;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  border: 2px solid var(--jedi-border);
  overflow: visible;
}

.global-switch-fab:hover {
  transform: translateY(-4px) scale(1.05);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.15) !important;
}

.global-switch-fab:active {
  transform: translateY(0) scale(0.98);
}

/* 状态指示器 */
.status-indicator {
  position: absolute;
  top: -8px;
  right: -8px;
  background-color: var(--jedi-bg-surface);
  color: var(--jedi-text-primary);
  border-radius: 12px;
  padding: 2px 8px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.5px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  transition: all 0.3s ease;
  border: 1px solid var(--jedi-border);
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
</style>
