<template>
  <!-- 添加分组对话框 -->
  <v-dialog v-model="dialogModel" max-width="520" persistent>
    <v-card class="scifi-card dialog-with-glow">
      <div class="dialog-decorator"></div>
      <v-card-title class="console-title-bar">
        <span class="dialog-icon">⬡</span>
        <span class="dialog-title">{{ $t('hosts.dialog.addGroupTitle') }}</span>
        <v-spacer></v-spacer>
        <button class="close-btn" @click="closeDialog">✕</button>
      </v-card-title>
      <v-card-text class="console-card-text">
        <div class="form-section">
          <label class="input-label">
            <span class="label-icon">▸</span>
            {{ $t('hosts.dialog.groupNameLabel') }}
          </label>
          <div class="input-wrapper">
            <span class="input-prefix">NAME</span>
            <input
              v-model="groupName"
              type="text"
              class="console-input with-prefix"
              :placeholder="$t('hosts.dialog.groupNamePlaceholder')"
              @keyup.enter="confirmAdd"
            />
          </div>
        </div>

        <div class="setting-item mb-4">
          <div class="setting-info">
            <div class="setting-label">
              <span class="label-icon">▸</span>
              {{ $t('hosts.dialog.useRemote') }}
            </div>
          </div>
          <div class="setting-action">
            <div
              class="toggle-switch"
              :class="{ active: isRemote }"
              @click="isRemote = !isRemote"
            >
              <div class="toggle-handle"></div>
            </div>
          </div>
        </div>

        <v-expand-transition>
          <div class="form-section" v-if="isRemote">
            <label class="input-label">
              <span class="label-icon">▸</span>
              {{ $t('hosts.dialog.remoteUrlLabel') }}
            </label>
            <div class="input-wrapper">
              <span class="input-prefix">URL</span>
              <input
                v-model="remoteUrl"
                type="text"
                class="console-input with-prefix"
                :placeholder="$t('hosts.dialog.remoteUrlPlaceholder')"
                @keyup.enter="confirmAdd"
              />
            </div>
          </div>
        </v-expand-transition>

        <v-expand-transition>
          <div v-if="!isRemote">
            <div class="form-section">
              <label class="input-label">
                <span class="label-icon">▸</span>
                {{ $t('hosts.dialog.hostsListLabel') }}
              </label>
              <div class="input-wrapper">
                <textarea
                  v-model="hostsContent"
                  class="console-input"
                  rows="4"
                  :placeholder="$t('hosts.dialog.hostsListPlaceholder')"
                ></textarea>
              </div>
            </div>
            <v-alert
              type="info"
              variant="tonal"
              density="compact"
              class="mt-2 hosts-hint"
            >
              <div class="text-body-2" v-html="$t('hosts.dialog.hostsListHint')">
              </div>
            </v-alert>
          </div>
        </v-expand-transition>
      </v-card-text>
      <v-card-actions class="console-card-actions">
        <button class="console-btn" @click="closeDialog">
          {{ $t('common.cancel') }}
        </button>
        <v-spacer></v-spacer>
        <button class="console-btn primary" @click="confirmAdd">
          {{ $t('common.confirm') }}
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { fetchRemoteConfig } from '@/api/hosts'
import { HostEntry } from '@/types/hosts'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'add', data: {
    name: string;
    isRemote: boolean;
    url?: string;
    hosts?: HostEntry[]
  }): void;
  (e: 'error', message: string): void;
}>()

// 对话框状态
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 表单数据
const groupName = ref('')
const isRemote = ref(false)
const remoteUrl = ref('')
const hostsContent = ref('')

// 关闭对话框
function closeDialog() {
  dialogModel.value = false
  resetForm()
}

// 重置表单
function resetForm() {
  groupName.value = ''
  isRemote.value = false
  remoteUrl.value = ''
  hostsContent.value = ''
}

// 确认添加
async function confirmAdd() {
  // 验证分组名称
  if (!groupName.value) {
    emit('error', '分组名称不能为空')
    return
  }

  // 处理远程配置
  if (isRemote.value) {
    if (!remoteUrl.value) {
      emit('error', '远程配置URL不能为空')
      return
    }

    try {
      // 获取远程配置
      const result = await fetchRemoteConfig(remoteUrl.value)

      // 验证远程配置
      if (!Array.isArray(result)) {
        emit('error', '远程配置格式错误')
        return
      }

      // 查找匹配的分组
      const matchedGroup = result.find(g => g.name === groupName.value)

      if (matchedGroup) {
        // 提交添加事件
        emit('add', {
          name: groupName.value,
          isRemote: true,
          url: remoteUrl.value,
          hosts: matchedGroup.hosts
        })

        // 关闭对话框
        closeDialog()
      } else {
        emit('error', '远程配置中未找到该分组')
      }
    } catch (error) {
      emit('error', `远程加载失败: ${(error as Error).message}`)
    }
  } else {
    // 处理本地配置
    // 先解析所有行
    const parsedLines = hostsContent.value
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0)
      .map(line => {
        const parts = line.split(/\s+/)
        if (parts.length >= 2) {
          const ip = parts[0]
          const domain = parts[1]
          return { domain, ip }
        }
        return null
      })
      .filter(item => item !== null)

    // 检查重复域名，只保留第一个出现的域名
    const uniqueDomains = new Set()
    const hostsArray: HostEntry[] = parsedLines
      .filter(item => {
        if (item && !uniqueDomains.has(item.domain)) {
          uniqueDomains.add(item.domain)
          return true
        }
        return false
      })
      .map(item => ({ 
        domain: item!.domain, 
        ip: item!.ip,
        disabled: false
      }))

    // 提交添加事件
    emit('add', {
      name: groupName.value,
      isRemote: false,
      hosts: hostsArray
    })

    // 关闭对话框
    closeDialog()
  }
}
</script>

<style scoped>
.dialog-with-glow {
  position: relative;
  box-shadow: 0 0 40px rgba(0, 255, 255, 0.15);
}

.console-title-bar {
  display: flex !important;
  align-items: center !important;
  flex-wrap: nowrap !important;
  gap: 8px;
  padding: 12px 16px;
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

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  background: rgba(5, 5, 8, 0.9);
  border: 1px solid #1a1a3a;
  border-radius: 4px;
  padding: 8px 12px;
}

.input-prefix {
  display: flex;
  align-items: center;
  padding: 0 12px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 11px;
  background: rgba(0, 255, 255, 0.05);
  border-right: 1px solid rgba(0, 255, 255, 0.2);
  min-width: 60px;
  justify-content: center;
}

.console-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #00ffff;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
}

.console-input::placeholder {
  color: #52525b;
}

.console-input.with-prefix {
  padding-left: 12px;
}

.hosts-hint {
  background: rgba(0, 255, 255, 0.05) !important;
  border: 1px solid rgba(0, 255, 255, 0.1) !important;
}

/* =========================================
   Light Theme Styles (Tatooine Outpost)
   ========================================= */
.light-theme .dialog-with-glow {
  box-shadow: 0 0 30px rgba(184, 134, 11, 0.2);
}

.light-theme .dialog-decorator {
  background: linear-gradient(90deg, transparent, #cd7f32, transparent);
}

.light-theme .dialog-icon {
  color: #cd7f32;
}

.light-theme .close-btn {
  color: rgba(107, 68, 35, 0.6);
}

.light-theme .close-btn:hover {
  color: #b22222;
}

.light-theme .input-label {
  color: rgba(61, 41, 20, 0.8);
}

.light-theme .label-icon {
  color: #cd7f32;
}

.light-theme .input-wrapper {
  background: #faf3e8;
  border-color: #d4a574;
}

.light-theme .input-prefix {
  color: rgba(107, 68, 35, 0.7);
  background: rgba(205, 127, 50, 0.08);
  border-right: 1px solid rgba(205, 127, 50, 0.25);
}

.light-theme .console-input {
  color: #3d2914;
}

.light-theme .console-input::placeholder {
  color: #8b7355;
}

.light-theme .hosts-hint {
  background: rgba(205, 127, 50, 0.08) !important;
  border: 1px solid rgba(205, 127, 50, 0.2) !important;
}

.light-theme .console-btn.primary {
  border-color: #cd7f32;
  background: #cd7f32;
  color: #ffffff;
}

.light-theme .console-btn.primary:hover {
  background: #b8860b;
}

.light-theme .dialog-title {
  color: #3d2914;
}

.light-theme .scifi-card {
  background: linear-gradient(135deg, #efe0cc 0%, #e8d4bc 100%);
}

.light-theme .console-card-text {
  background: #faf3e8;
}

.light-theme .console-card-actions {
  background: linear-gradient(0deg, #e8d4bc 0%, #efe0cc 100%);
  border-top: 1px solid #b8860b;
}
</style>
