<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="480"
  >
    <v-card class="model-settings-card">
      <!-- Header -->
      <div class="card-header">
        <div class="header-brand">
          <div class="brand-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
              <path d="M9 9a3 3 0 1 1 4 2.83V14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              <circle cx="12" cy="17" r="1" fill="currentColor"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>User Guide</h2>
            <p>Quick start instructions</p>
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
        <div class="help-section">
          <div class="help-item" v-for="item in helpItems" :key="item.icon">
            <div class="help-icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
                <line x1="12" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="2"/>
                <circle cx="12" cy="16" r="1" fill="currentColor"/>
              </svg>
            </div>
            <span class="help-text">{{ item.text }}</span>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="card-footer">
        <v-btn variant="tonal" color="primary" @click="$emit('update:modelValue', false)">
          Got it
        </v-btn>
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
defineProps<{ modelValue: boolean }>()
defineEmits<{ (e: 'update:modelValue', value: boolean): void }>()

const helpItems = [
  { icon: 'add-group', text: 'Add group to organize hosts entries' },
  { icon: 'add-entry', text: 'Add host entries with IP and domain' },
  { icon: 'toggle', text: 'Toggle entries on/off individually' },
  { icon: 'global', text: 'Use global switch to enable/disable all' },
]
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

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(20, 30, 40, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
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
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.15) 0%, rgba(0, 255, 136, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.25);
  border-radius: 10px;
  color: #00ffff;
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
  padding: 20px;
}

.help-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.help-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 14px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 10px;
  transition: all 0.15s ease;
}

.help-item:hover {
  background: rgba(0, 255, 255, 0.04);
  border-color: rgba(0, 255, 255, 0.15);
}

.help-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 255, 255, 0.08);
  border-radius: 8px;
  color: #00ffff;
  flex-shrink: 0;
}

.help-text {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.75);
  line-height: 1.4;
}

.card-footer {
  display: flex;
  justify-content: flex-end;
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
