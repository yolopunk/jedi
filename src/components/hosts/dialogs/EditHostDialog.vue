<template>
  <!-- 编辑条目对话框 -->
  <v-dialog v-model="dialogModel" max-width="480" persistent>
    <v-card class="scifi-card dialog-with-glow">
      <div class="dialog-decorator"></div>
      <v-card-title class="console-title-bar">
        <span class="dialog-icon">⬡</span>
        <span class="dialog-title">{{ $t('hosts.dialog.editTitle') }}</span>
        <v-spacer></v-spacer>
        <button class="close-btn" @click="closeDialog">✕</button>
      </v-card-title>
      <v-card-text class="console-card-text">
        <div class="form-section">
          <label class="input-label">
            <span class="label-icon">▸</span>
            {{ $t('hosts.dialog.ipLabel') }}
          </label>
          <div class="input-wrapper">
            <span class="input-prefix">IP</span>
            <input
              v-model="hostIp"
              type="text"
              class="console-input with-prefix"
              :placeholder="$t('hosts.dialog.ipPlaceholder')"
              @keyup.enter="confirmEdit"
            />
          </div>
        </div>

        <div class="form-section">
          <label class="input-label">
            <span class="label-icon">▸</span>
            {{ $t('hosts.dialog.domainLabel') }}
          </label>
          <div class="input-wrapper">
            <span class="input-prefix">🌐</span>
            <input
              v-model="hostDomain"
              type="text"
              class="console-input with-prefix"
              :placeholder="$t('hosts.dialog.domainPlaceholder')"
              @keyup.enter="confirmEdit"
            />
          </div>
        </div>
      </v-card-text>
      <v-card-actions class="console-card-actions">
        <button class="console-btn" @click="closeDialog">
          {{ $t('common.cancel') }}
        </button>
        <v-spacer></v-spacer>
        <button class="console-btn primary" @click="confirmEdit">
          {{ $t('common.confirm') }}
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { validateHostInput } from '@/utils/hostsUtils'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
  host: any | null;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'edit', data: {
    originalHost: any;
    ip: string;
    domain: string;
  }): void;
  (e: 'error', message: string): void;
}>()

// 对话框状态
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 表单数据
const hostIp = ref('')
const hostDomain = ref('')

// 监听主机数据变化
watch(() => props.host, (newHost) => {
  if (newHost) {
    hostIp.value = newHost.ip
    hostDomain.value = newHost.domain
  }
}, { immediate: true })

// 关闭对话框
function closeDialog() {
  dialogModel.value = false
  resetForm()
}

// 重置表单
function resetForm() {
  hostIp.value = ''
  hostDomain.value = ''
}

// 确认编辑
function confirmEdit() {
  // 验证输入
  if (!validateHostInput(hostIp.value, hostDomain.value)) {
    emit('error', 'IP和域名不能为空')
    return
  }

  // 验证编辑数据
  if (!props.host) {
    emit('error', '编辑数据丢失')
    return
  }

  // 提交编辑事件
  emit('edit', {
    originalHost: props.host,
    ip: hostIp.value.trim(),
    domain: hostDomain.value.trim()
  })

  // 关闭对话框
  closeDialog()
}
</script>

<style scoped>
.dialog-with-glow {
  position: relative;
  box-shadow: 0 0 40px rgba(0, 255, 255, 0.15);
}

.dialog-decorator {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, #00ffff, transparent);
  opacity: 0.6;
}

.dialog-icon {
  color: #00ffff;
  margin-right: 8px;
  font-size: 14px;
}

.dialog-title {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 1px;
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

.form-section {
  margin-bottom: 20px;
}

.input-label {
  display: flex;
  align-items: center;
  color: rgba(255, 255, 255, 0.7);
  font-size: 12px;
  margin-bottom: 8px;
  letter-spacing: 1px;
}

.label-icon {
  color: #00ff88;
  margin-right: 8px;
}

.input-prefix {
  display: flex;
  align-items: center;
  padding: 0 12px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
  background: rgba(0, 255, 255, 0.05);
  border-right: 1px solid rgba(0, 255, 255, 0.2);
  min-width: 50px;
  justify-content: center;
}

.console-input.with-prefix {
  padding-left: 12px;
}
</style>
