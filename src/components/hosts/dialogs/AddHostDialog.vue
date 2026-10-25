<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="onModelChange"
    max-width="480"
  >
    <v-card class="model-settings-card">
      <!-- Header -->
      <div class="card-header">
        <div class="header-brand">
          <div class="brand-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <rect x="3" y="3" width="18" height="18" rx="2" stroke="currentColor" stroke-width="1.5"/>
              <line x1="12" y1="8" x2="12" y2="16" stroke="currentColor" stroke-width="2"/>
              <line x1="8" y1="12" x2="16" y2="12" stroke="currentColor" stroke-width="2"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>{{ $t('hosts.dialog.addHostTitle') }}</h2>
            <p>{{ $t('hosts.dialog.addToGroup', { group: groupName }) }}</p>
          </div>
        </div>
        <button class="close-btn" type="button" :aria-label="$t('common.close')" @click="requestClose">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="card-body">
        <div class="form-group">
          <label class="form-label">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="8" stroke="currentColor" stroke-width="2"/>
              <line x1="12" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="2"/>
              <line x1="12" y1="12" x2="14" y2="14" stroke="currentColor" stroke-width="2"/>
            </svg>
            {{ $t('hosts.dialog.ipLabel') }}
          </label>
          <input
            id="add-host-ip"
            ref="ipInputRef"
            v-model="hostIp"
            type="text"
            class="form-input"
            placeholder="192.168.1.1"
            @keyup.enter="confirmAdd"
          />
        </div>

        <div class="form-group">
          <label class="form-label">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
              <path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10" stroke="currentColor" stroke-width="1.5"/>
            </svg>
            {{ $t('hosts.dialog.domainLabel') }}
          </label>
          <input
            id="add-host-domain"
            v-model="hostDomain"
            type="text"
            class="form-input"
            placeholder="example.com"
            @keyup.enter="confirmAdd"
          />
        </div>

        <div v-if="showDirtyConfirm" class="dirty-bar" role="alert">
          <span class="dirty-title">{{ $t('mcp.dirtyTitle') }}</span>
          <div class="dirty-actions">
            <button ref="dirtyKeepRef" type="button" class="dirty-btn" @click="keepEditing">
              {{ $t('mcp.dirtyKeep') }}
            </button>
            <button type="button" class="dirty-btn danger" @click="discardAndClose">
              {{ $t('mcp.dirtyDiscard') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="card-footer">
        <div class="footer-hint">
          <span class="hint-dot"></span>
          {{ $t('hosts.dialog.addToGroup', { group: groupName }) }}
        </div>
        <v-spacer />
        <v-btn variant="text" @click="closeDialog">{{ $t('common.cancel') }}</v-btn>
        <v-btn variant="tonal" color="primary" @click="confirmAdd">{{ $t('common.add') }}</v-btn>
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { validateHostInput } from '@/utils/hostsUtils'

const { t } = useI18n()

const props = defineProps<{
  modelValue: boolean
  groupName: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'add', data: { groupName: string; ip: string; domain: string }): void
  (e: 'error', message: string): void
}>()

const hostIp = ref('')
const hostDomain = ref('')
const showDirtyConfirm = ref(false)
const dirtyKeepRef = ref<HTMLButtonElement | null>(null)
const ipInputRef = ref<HTMLInputElement | null>(null)

// 任一输入字段有内容即视为脏数据
const isDirty = computed(() => hostIp.value.trim() !== '' || hostDomain.value.trim() !== '')

watch(
  () => props.modelValue,
  open => {
    if (open) {
      hostIp.value = ''
      hostDomain.value = ''
      showDirtyConfirm.value = false
    }
  }
)

function onModelChange(value: boolean) {
  if (value) {
    emit('update:modelValue', true)
    return
  }
  requestClose()
}

function closeDialog() {
  emit('update:modelValue', false)
}

// 空表单时 Esc / 遮罩 / 关闭按钮直接关闭；有改动时先显示内联确认条
function requestClose() {
  if (!isDirty.value) {
    closeDialog()
    return
  }
  showDirtyConfirm.value = true
  nextTick(() => dirtyKeepRef.value?.focus())
}

function keepEditing() {
  showDirtyConfirm.value = false
  nextTick(() => ipInputRef.value?.focus())
}

function discardAndClose() {
  hostIp.value = ''
  hostDomain.value = ''
  showDirtyConfirm.value = false
  closeDialog()
}

function confirmAdd() {
  if (!validateHostInput(hostIp.value, hostDomain.value)) {
    emit('error', t('hosts.dialog.ipDomainRequired'))
    return
  }
  emit('add', {
    groupName: props.groupName,
    ip: hostIp.value.trim(),
    domain: hostDomain.value.trim(),
  })
  closeDialog()
}
</script>

<style scoped>
.model-settings-card {
  background: var(--bg-terminal) !important;
  border-radius: 16px !important;
  overflow: hidden;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgb(var(--bg-rgb) / 0.6);
  border-bottom: 1px solid rgb(var(--text-rgb) / 0.06);
  flex-shrink: 0;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.brand-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgb(var(--accent-rgb) / 0.15) 0%, rgb(var(--success-rgb) / 0.05) 100%);
  border: 1px solid rgb(var(--accent-rgb) / 0.25);
  border-radius: 10px;
  color: var(--accent);
}

.brand-text h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
}

.brand-text p {
  margin: 2px 0 0;
  font-size: 12px;
  color: rgb(var(--text-rgb) / 0.4);
}

.close-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid rgb(var(--text-rgb) / 0.08);
  border-radius: 8px;
  color: rgb(var(--text-rgb) / 0.5);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: rgb(var(--danger-rgb) / 0.1);
  border-color: rgb(var(--danger-rgb) / 0.3);
  color: var(--danger);
}

.card-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1px;
  color: rgb(var(--text-rgb) / 0.5);
  text-transform: uppercase;
}

.form-label svg {
  color: var(--success);
}

.form-input {
  width: 100%;
  padding: 10px 12px;
  background: rgb(var(--accent-rgb) / 0.03);
  border: 1px solid rgb(var(--accent-rgb) / 0.12);
  border-radius: 10px;
  outline: none;
  color: var(--text);
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.form-input:focus {
  border-color: rgb(var(--accent-rgb) / 0.4);
  box-shadow: 0 0 0 2px rgb(var(--accent-rgb) / 0.1);
}

.form-input::placeholder {
  color: rgb(var(--text-rgb) / 0.45);
}

.dirty-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 8px;
  background: rgb(var(--warning-rgb) / 0.14);
  border: 1px solid rgb(var(--warning-rgb) / 0.32);
}

.dirty-title {
  font-size: 12px;
  color: var(--warning);
}

.dirty-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.dirty-btn {
  cursor: pointer;
  border: none;
  border-radius: 7px;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  background: rgb(var(--text-rgb) / 0.08);
  color: var(--text);
  transition: filter 0.15s ease;
}

.dirty-btn:hover {
  filter: brightness(1.15);
}

.dirty-btn.danger {
  background: rgb(var(--danger-rgb) / 0.16);
  color: var(--danger);
}

.card-footer {
  display: flex;
  align-items: center;
  padding: 16px 20px;
  border-top: 1px solid rgb(var(--text-rgb) / 0.06);
  flex-shrink: 0;
}

.footer-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: rgb(var(--text-rgb) / 0.4);
}

.hint-dot {
  width: 6px;
  height: 6px;
  background: var(--success);
  border-radius: 50%;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background:rgb(var(--ink-rgb) / 0.1);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background:rgb(var(--ink-rgb) / 0.15);
}
</style>
