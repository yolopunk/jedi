<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="480"
    persistent
    attach="body"
  >
    <v-card class="model-settings-card">
      <!-- Header -->
      <div class="card-header">
        <div class="header-brand">
          <div class="brand-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" stroke="currentColor" stroke-width="1.5"/>
              <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>{{ $t('hosts.dialog.editSourceTitle') }}</h2>
            <p>{{ $t('hosts.dialog.editSourceSubtitle') }}</p>
          </div>
        </div>
        <button
          class="close-btn"
          type="button"
          :aria-label="$t('common.close')"
          @click="closeDialog"
        >
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
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" stroke="currentColor" stroke-width="2"/>
              <circle cx="12" cy="7" r="4" stroke="currentColor" stroke-width="2"/>
            </svg>
            {{ $t('hosts.dialog.groupNameLabel') }}
          </label>
          <div class="static-value">{{ source?.name || '-' }}</div>
        </div>

        <div class="form-group">
          <label class="form-label">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
              <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" stroke="currentColor" stroke-width="2"/>
              <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" stroke="currentColor" stroke-width="2"/>
            </svg>
            {{ $t('hosts.dialog.remoteUrlLabel') }}
          </label>
          <div class="input-wrapper">
            <span class="input-prefix">URL</span>
            <input
              v-model="url"
              type="text"
              class="form-input with-prefix"
              :placeholder="$t('hosts.dialog.remoteUrlPlaceholder')"
              @keyup.enter="confirmSave"
            />
          </div>
          <div class="input-hint">{{ $t('hosts.dialog.remoteToggleHint') }}</div>
        </div>
      </div>

      <!-- Footer -->
      <div class="card-footer">
        <button type="button" class="console-btn" @click="closeDialog">
          <span class="btn-text">{{ $t('common.cancel') }}</span>
        </button>
        <button type="button" class="console-btn primary" @click="confirmSave">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          <span class="btn-text">{{ $t('hosts.dialog.saveAndSync') }}</span>
        </button>
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { HostsSource } from '@/types/hosts'

const props = defineProps<{
  modelValue: boolean
  source: HostsSource | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', payload: { name: string; url: string }): void
  (e: 'error', message: string): void
}>()

const { t } = useI18n()
const url = ref('')

// 每次打开时用当前数据源的 URL 回填
watch(
  () => props.modelValue,
  open => {
    if (open && props.source) {
      url.value = props.source.url
    }
  },
  { immediate: true }
)

function closeDialog() {
  emit('update:modelValue', false)
}

function confirmSave() {
  if (!props.source) return

  const trimmed = url.value.trim()
  if (!trimmed) {
    emit('error', t('hosts.dialog.remoteUrlRequired'))
    return
  }
  if (!/^https?:\/\//i.test(trimmed)) {
    emit('error', t('hosts.dialog.remoteUrlInvalid'))
    return
  }

  emit('save', { name: props.source.name, url: trimmed })
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

.static-value {
  padding: 12px 14px;
  background: rgb(var(--ink-rgb) / 0.03);
  border: 1px solid rgb(var(--text-rgb) / 0.06);
  border-radius: 10px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  color: var(--accent);
}

.input-wrapper {
  display: flex;
  align-items: center;
  background: rgb(var(--accent-rgb) / 0.03);
  border: 1px solid rgb(var(--accent-rgb) / 0.12);
  border-radius: 10px;
  overflow: hidden;
  transition: border-color 0.15s;
}

.input-wrapper:focus-within {
  border-color: rgb(var(--accent-rgb) / 0.4);
  box-shadow: 0 0 0 2px rgb(var(--accent-rgb) / 0.1);
}

.input-prefix {
  padding: 12px 14px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  color: rgb(var(--accent-rgb) / 0.6);
  background: rgb(var(--accent-rgb) / 0.05);
  border-right: 1px solid rgb(var(--accent-rgb) / 0.12);
  min-width: 64px;
  text-align: center;
}

.form-input {
  flex: 1;
  padding: 12px 14px;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text);
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
}

.form-input::placeholder {
  color: rgb(var(--text-rgb) / 0.2);
}

.form-input.with-prefix {
  padding-left: 12px;
}

.input-hint {
  font-size: 11px;
  color: rgb(var(--text-rgb) / 0.3);
  margin-top: 4px;
}

.card-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid rgb(var(--text-rgb) / 0.06);
  flex-shrink: 0;
}
</style>
