<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="640"
  >
    <v-card class="model-settings-card">
      <!-- Header -->
      <div class="card-header">
        <div class="header-brand">
          <div class="brand-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.5"/>
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>Settings</h2>
            <p>Configure application preferences</p>
          </div>
        </div>
        <button class="close-btn" @click="$emit('update:modelValue', false)">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <!-- Tabs -->
      <div class="settings-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.value"
          class="tab-btn"
          :class="{ active: settingsTab === tab.value }"
          @click="settingsTab = tab.value"
        >
          {{ tab.label }}
        </button>
      </div>

      <!-- Content -->
      <div class="card-body">
        <GeneralSettingsTab v-if="settingsTab === 'general'" />
        <WallpaperSettingsTab v-if="settingsTab === 'wallpaper'" />
        <AdvancedSettingsTab v-if="settingsTab === 'advanced'" />
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const tabs = computed(() => [
  { value: 'general', label: t('settings.general') },
  { value: 'wallpaper', label: t('settings.wallpaper') },
  { value: 'advanced', label: t('settings.advanced') },
])

defineProps<{ modelValue: boolean }>()
defineEmits<(e: 'update:modelValue', value: boolean) => void>()

const settingsTab = ref('general')
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

.settings-tabs {
  display: flex;
  gap: 4px;
  padding: 12px 16px;
  background:rgb(var(--ink-rgb) / 0.2);
  border-bottom: 1px solid rgb(var(--text-rgb) / 0.04);
  flex-shrink: 0;
}

.tab-btn {
  padding: 8px 16px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  color: rgb(var(--text-rgb) / 0.4);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  background:rgb(var(--ink-rgb) / 0.03);
  color: rgb(var(--text-rgb) / 0.6);
}

.tab-btn.active {
  background: rgb(var(--accent-rgb) / 0.1);
  border-color: rgb(var(--accent-rgb) / 0.25);
  color: var(--accent);
}

.card-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
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
