<template>
  <v-dialog v-model="dialogModel" max-width="800">
    <v-card class="scifi-card">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">[ SYSTEM_CONFIG ]</span>
      </v-card-title>
      <v-card-text class="console-card-text settings-content">
        <!-- Tabs -->
        <div class="settings-tabs">
          <button
            v-for="tab in tabs"
            :key="tab.value"
            class="tab-button"
            :class="{ active: settingsTab === tab.value }"
            @click="settingsTab = tab.value"
          >
            <span class="tab-text">{{ tab.label }}</span>
          </button>
        </div>

        <div class="tab-content">
          <GeneralSettingsTab v-if="settingsTab === 'general'" />
          <WallpaperSettingsTab v-if="settingsTab === 'wallpaper'" />
          <AdvancedSettingsTab v-if="settingsTab === 'advanced'" />
        </div>
      </v-card-text>
      <v-card-actions class="console-card-actions">
        <v-spacer></v-spacer>
        <button class="console-btn" @click="dialogModel = false">
          <span class="btn-text">{{ t('settings.close') }}</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GeneralSettingsTab from './settings/GeneralSettingsTab.vue'
import WallpaperSettingsTab from './settings/WallpaperSettingsTab.vue'
import AdvancedSettingsTab from './settings/AdvancedSettingsTab.vue'

const { t } = useI18n()

const tabs = computed(() => [
  { value: 'general', label: t('settings.general') },
  { value: 'wallpaper', label: t('settings.wallpaper') },
  { value: 'advanced', label: t('settings.advanced') }
])

const props = defineProps<{
  modelValue: boolean;
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>()

const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const settingsTab = ref('general')
</script>

<style scoped>
.settings-content {
  padding: 0 !important;
}

.settings-tabs {
  display: flex;
  gap: 4px;
  padding: 12px 16px 8px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid rgba(0, 255, 255, 0.1);
}

.tab-button {
  padding: 8px 16px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: #52525b;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 1px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab-button:hover {
  background: rgba(0, 255, 255, 0.05);
  color: #a1a1aa;
}

.tab-button.active {
  background: rgba(0, 255, 255, 0.1);
  color: #00ffff;
}

.tab-content {
  padding: 16px;
  max-height: 400px;
  overflow-y: auto;
}

/* Custom scrollbar */
.tab-content::-webkit-scrollbar {
  width: 6px;
}

.tab-content::-webkit-scrollbar-track {
  background: transparent;
}

.tab-content::-webkit-scrollbar-thumb {
  background: rgba(0, 255, 255, 0.2);
  border-radius: 3px;
}

.tab-content::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 255, 255, 0.3);
}

/* =========================================
   Light Theme Styles
   ========================================= */
.light-theme .scifi-card {
  background: linear-gradient(135deg, #efe0cc 0%, #e8d4bc 100%);
  border-color: rgba(184, 134, 11, 0.3);
}

.light-theme .scifi-card::before {
  background: linear-gradient(90deg, transparent, #cd7f32, transparent);
}

.light-theme .console-title-bar {
  background: linear-gradient(180deg, #efe0cc 0%, #e8d4bc 100%);
  border-bottom-color: rgba(184, 134, 11, 0.3);
}

.light-theme .dialog-title {
  color: #cd7f32;
  text-shadow: 0 0 8px rgba(205, 127, 50, 0.3);
}

.light-theme .console-card-text {
  background: #f5e6d3;
}

.light-theme .settings-tabs {
  background: #e8d4bc;
  border-bottom-color: rgba(184, 134, 11, 0.25);
}

.light-theme .tab-button {
  color: #6b4423;
}

.light-theme .tab-button:hover {
  color: #cd7f32;
}

.light-theme .tab-button.active {
  color: #cd7f32;
}

.light-theme .tab-text {
  color: inherit;
}

.light-theme .tab-content::-webkit-scrollbar-thumb {
  background: rgba(184, 134, 11, 0.3);
}

.light-theme .tab-content::-webkit-scrollbar-thumb:hover {
  background: rgba(184, 134, 11, 0.5);
}
</style>
