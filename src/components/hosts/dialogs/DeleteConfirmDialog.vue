<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="420"
  >
    <v-card class="model-settings-card danger-card">
      <!-- Header -->
      <div class="card-header danger">
        <div class="header-brand">
          <div class="brand-icon danger">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" stroke="currentColor" stroke-width="1.5"/>
              <line x1="12" y1="9" x2="12" y2="13" stroke="currentColor" stroke-width="2"/>
              <circle cx="12" cy="17" r="1" fill="currentColor"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>Confirm Delete</h2>
            <p>This action cannot be undone</p>
          </div>
        </div>
        <button class="close-btn" @click="$emit('update:modelValue', false)">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="card-body">
        <div class="warning-icon">
          <svg width="36" height="36" viewBox="0 0 24 24" fill="none">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" stroke="currentColor" stroke-width="1.5"/>
            <line x1="12" y1="9" x2="12" y2="13" stroke="currentColor" stroke-width="2"/>
            <circle cx="12" cy="17" r="1" fill="currentColor"/>
          </svg>
        </div>

        <p class="confirm-text">Are you sure you want to delete this host entry?</p>

        <div v-if="host" class="host-info">
          <div class="info-row">
            <span class="info-label">IP</span>
            <span class="info-value">{{ host.ip }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">Domain</span>
            <span class="info-value">{{ host.domain }}</span>
          </div>
        </div>

        <p class="danger-hint">This operation is irreversible.</p>
      </div>

      <!-- Footer -->
      <div class="card-footer">
        <v-btn variant="text" @click="$emit('update:modelValue', false)">
          Cancel
        </v-btn>
        <v-btn variant="tonal" color="error" @click="confirmDelete">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          Delete
        </v-btn>
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import type { HostEntry } from '@/types/hosts'

const props = defineProps<{
  modelValue: boolean
  host: HostEntry | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'delete', host: HostEntry): void
}>()

function _confirmDelete() {
  if (props.host) {
    emit('delete', props.host)
  }
  emit('update:modelValue', false)
}
</script>

<style scoped>
.model-settings-card {
  background: #0a0e14 !important;
  border-radius: 16px !important;
  overflow: hidden;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
}

.danger-card {
  border: 1px solid rgba(255, 107, 107, 0.2);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(20, 30, 40, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.card-header.danger {
  background: rgba(255, 50, 50, 0.05);
  border-bottom-color: rgba(255, 107, 107, 0.15);
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
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.15) 0%, rgba(0, 255, 136, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.25);
  border-radius: 10px;
  color: #00ffff;
}

.brand-icon.danger {
  background: linear-gradient(135deg, rgba(255, 50, 50, 0.15) 0%, rgba(255, 107, 107, 0.05) 100%);
  border-color: rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

.brand-text h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.brand-text p {
  margin: 2px 0 0;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.card-header.danger .brand-text h2 {
  color: #ff6b6b;
}

.close-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: rgba(255, 107, 107, 0.1);
  border-color: rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

.card-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.warning-icon {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 107, 107, 0.08);
  border-radius: 50%;
  color: #ff6b6b;
  margin-bottom: 16px;
}

.confirm-text {
  margin: 0;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.85);
  line-height: 1.5;
}

.host-info {
  margin-top: 16px;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  width: 100%;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
}

.info-label {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1px;
  color: rgba(255, 255, 255, 0.35);
  text-transform: uppercase;
}

.info-value {
  font-size: 13px;
  font-family: 'JetBrains Mono', monospace;
  color: #00ffff;
}

.danger-hint {
  margin: 16px 0 0;
  font-size: 11px;
  color: rgba(255, 107, 107, 0.7);
}

.card-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.15);
}
</style>
