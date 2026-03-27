<template>
  <!-- 删除确认对话框 -->
  <v-dialog v-model="dialogModel" max-width="420" persistent>
    <v-card class="scifi-card dialog-danger">
      <div class="dialog-decorator danger"></div>
      <v-card-title class="console-title-bar danger">
        <span class="dialog-icon danger">⬡</span>
        <span class="dialog-title">{{ $t('hosts.dialog.deleteTitle') }}</span>
        <v-spacer></v-spacer>
        <button class="close-btn" @click="closeDialog">✕</button>
      </v-card-title>
      <v-card-text class="console-card-text">
        <div class="delete-warning-icon">⚠</div>
        <p class="delete-confirm-text">{{ $t('hosts.dialog.deleteConfirm') }}</p>
        <div v-if="host" class="mt-3 pa-3 host-info-box">
          <div class="d-flex align-center mb-1">
            <span class="term-cyan mr-2">▸</span>
            <span class="font-weight-medium">{{ $t('hosts.dialog.ipLabel') }}：</span>
            <span class="ml-2">{{ host.ip }}</span>
          </div>
          <div class="d-flex align-center">
            <span class="term-cyan mr-2">▸</span>
            <span class="font-weight-medium">{{ $t('hosts.dialog.domainLabel') }}：</span>
            <span class="ml-2">{{ host.domain }}</span>
          </div>
        </div>
        <p class="text-body-2 mt-4 term-red">{{ $t('hosts.dialog.deleteWarning') }}</p>
      </v-card-text>
      <v-card-actions class="console-card-actions">
        <button class="console-btn" @click="closeDialog">
          {{ $t('common.cancel') }}
        </button>
        <v-spacer></v-spacer>
        <button class="console-btn danger" @click="confirmDelete">
          {{ $t('hosts.dialog.deleteBtn') }}
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { HostEntry } from '@/types/hosts'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
  host: HostEntry | null;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'delete', host: HostEntry): void;
}>()

// 对话框状态
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 关闭对话框
function closeDialog() {
  dialogModel.value = false
}

// 确认删除
function confirmDelete() {
  if (props.host) {
    emit('delete', props.host)
  }
  closeDialog()
}
</script>

<style scoped>
.dialog-danger {
  box-shadow: 0 0 40px rgba(255, 68, 68, 0.15);
}

.dialog-decorator.danger {
  background: linear-gradient(90deg, transparent, #ff4444, transparent);
}

.console-title-bar.danger {
  border-bottom-color: rgba(255, 68, 68, 0.3);
}

.dialog-icon.danger {
  color: #ff4444;
}

.delete-warning-icon {
  font-size: 48px;
  text-align: center;
  margin-bottom: 16px;
  animation: pulse-warning 2s infinite;
}

@keyframes pulse-warning {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.7; transform: scale(1.1); }
}

.delete-confirm-text {
  text-align: center;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
}

.host-info-box {
  background-color: var(--jedi-bg-surface-hover);
  border-radius: 8px;
  border: 1px solid var(--jedi-border);
}

.dialog-title {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #ff4444;
}

.close-btn {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  font-size: 14px;
  padding: 4px 8px;
  transition: color 0.2s;
}

.close-btn:hover {
  color: #ff4444;
}

.term-red {
  color: #ff4444;
}

/* =========================================
   Light Theme Styles (Tatooine Outpost)
   ========================================= */
.light-theme .dialog-danger {
  box-shadow: 0 0 30px rgba(178, 34, 34, 0.2);
}

.light-theme .dialog-decorator.danger {
  background: linear-gradient(90deg, transparent, #b22222, transparent);
}

.light-theme .console-title-bar.danger {
  border-bottom-color: rgba(178, 34, 34, 0.4);
}

.light-theme .dialog-icon.danger {
  color: #b22222;
}

.light-theme .delete-confirm-text {
  color: rgba(61, 41, 20, 0.9);
}

.light-theme .dialog-title {
  color: #b22222;
}

.light-theme .close-btn {
  color: rgba(107, 68, 35, 0.6);
}

.light-theme .close-btn:hover {
  color: #b22222;
}

.light-theme .term-red {
  color: #b22222;
}
</style>
