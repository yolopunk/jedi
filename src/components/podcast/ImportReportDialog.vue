<template>
  <v-dialog
    :model-value="modelValue"
    max-width="580"
    persistent
    @update:model-value="v => emit('update:modelValue', v)"
  >
    <v-card class="import-report-dialog">
      <div class="dialog-decorator"></div>

      <v-card-title class="console-title-bar">
        <span class="dialog-icon">◈</span>
        <span class="dialog-title">{{ $t('podcast.importResult') }}</span>
        <v-spacer />
        <button
          type="button"
          class="close-btn"
          :aria-label="$t('common.close')"
          @click="emit('update:modelValue', false)"
        >
          <v-icon :icon="mdiClose" size="16" aria-hidden="true" />
        </button>
      </v-card-title>

      <v-card-text class="report-body">
        <!-- 成功摘要 -->
        <div class="summary">
          <div class="summary-icon">
            <v-icon :icon="mdiCheck" size="20" aria-hidden="true" />
          </div>
          <div class="summary-text">
            <div class="summary-title">{{ $t('podcast.importAdded', { n: addedCount }) }}</div>
            <div v-if="failures.length" class="summary-sub">
              {{ $t('podcast.importFailedCount', { n: failures.length }) }}
            </div>
          </div>
        </div>

        <!-- 失败列表 -->
        <div v-if="failures.length" class="failure-list">
          <div v-for="f in failures" :key="f.xml_url" class="failure-item">
            <v-icon :icon="mdiAlertCircleOutline" size="14" class="fail-icon" aria-hidden="true" />
            <div class="failure-info">
              <div class="failure-title">{{ f.title || f.xml_url }}</div>
              <div class="failure-reason mono">{{ shorten(f.error, 72) }}</div>
            </div>
            <button
              v-if="isXiaoyuzhou(f.xml_url)"
              type="button"
              class="console-btn small"
              @click="emit('retry', f)"
            >
              {{ $t('podcast.addManually') }}
            </button>
          </div>
        </div>

        <!-- 提示条 -->
        <div v-if="failures.length" class="hint">
          <v-icon :icon="mdiInformationOutline" size="14" aria-hidden="true" />
          <span>{{ $t('podcast.importFailureHint') }}</span>
        </div>
      </v-card-text>

      <v-card-actions class="console-card-actions">
        <v-spacer />
        <button type="button" class="console-btn" @click="emit('update:modelValue', false)">
          {{ $t('common.close') }}
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { mdiAlertCircleOutline, mdiCheck, mdiClose, mdiInformationOutline } from '@mdi/js'
import { useI18n } from 'vue-i18n'
import type { OpmlImportFailure } from '@/api/podcast'

defineProps<{
  modelValue: boolean
  addedCount: number
  failures: OpmlImportFailure[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'retry', failure: OpmlImportFailure): void
}>()

useI18n()

const isXiaoyuzhou = (u: string) => u.includes('xiaoyuzhoufm.com')

function shorten(s: string, max: number) {
  return s.length > max ? `${s.slice(0, max - 1)}…` : s
}
</script>

<style scoped>
.import-report-dialog {
  background: var(--bg-elevated) !important;
  border: 1px solid var(--border);
  border-radius: var(--radius-dialog) !important;
  overflow: hidden;
  box-shadow: var(--shadow-dialog);
  position: relative;
}

.dialog-decorator {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, var(--accent), transparent);
  opacity: 0.6;
  z-index: 1;
}

.report-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px 20px 8px !important;
}

.summary {
  display: flex;
  align-items: center;
  gap: 14px;
}

.summary-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: rgb(var(--success-rgb) / 0.12);
  border: 1px solid rgb(var(--success-rgb) / 0.35);
  display: grid;
  place-items: center;
  color: var(--success);
  flex-shrink: 0;
}

.summary-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.summary-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
}

.summary-sub {
  font-size: 12px;
  color: var(--text-muted);
}

.failure-list {
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-panel);
  overflow: hidden;
}

.failure-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
}

.failure-item + .failure-item {
  border-top: 1px solid var(--border);
}

.fail-icon {
  color: var(--danger);
  flex-shrink: 0;
}

.failure-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.failure-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.failure-reason {
  font-family: var(--jedi-font-mono);
  font-size: 10px;
  color: var(--text-subtle);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mono {
  font-family: var(--jedi-font-mono);
}

.console-btn.small {
  min-height: 30px;
  padding: 0 12px;
  font-size: 12px;
  flex-shrink: 0;
}

.hint {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  padding: 12px 14px;
  background: rgb(var(--warning-rgb) / 0.08);
  border: 1px solid rgb(var(--warning-rgb) / 0.3);
  border-radius: 10px;
}

.hint .v-icon {
  color: var(--warning);
  flex-shrink: 0;
  margin-top: 2px;
}

.hint span {
  font-size: 12px;
  line-height: 1.8;
  color: var(--text-muted);
}

.close-btn {
  background: none;
  border: none;
  color: rgb(var(--text-rgb) / 0.5);
  cursor: pointer;
  padding: 4px 8px;
  display: grid;
  place-items: center;
}

.close-btn:hover {
  color: var(--danger);
}
</style>
