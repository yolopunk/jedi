<template>
  <v-dialog v-model="dialogModel" max-width="600">
    <v-card class="scifi-card">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">[ USER_GUIDE ]</span>
      </v-card-title>
      <v-card-text class="console-card-text">
        <h3 class="help-section-title mb-4">&gt; {{ $t('help.basicGuide') }}</h3>
        <div class="help-list">
          <div class="help-item">
            <span class="help-icon">＋</span>
            <span class="help-text">{{ $t('help.addGroup') }}</span>
          </div>
          <div class="help-item">
            <span class="help-icon">＋</span>
            <span class="help-text">{{ $t('help.addEntry') }}</span>
          </div>
          <div class="help-item">
            <span class="help-icon">⊡</span>
            <span class="help-text">{{ $t('help.toggleEntry') }}</span>
          </div>
          <div class="help-item">
            <span class="help-icon">⚡</span>
            <span class="help-text">{{ $t('help.globalSwitch') }}</span>
          </div>
        </div>
      </v-card-text>
      <v-card-actions class="console-card-actions">
        <v-spacer></v-spacer>
        <button class="console-btn primary" @click="dialogModel = false">
          <span class="btn-text">{{ $t('common.close') }}</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'

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
</script>

<style scoped>
.help-section-title {
  font-size: 12px;
  font-weight: 700;
  color: #00ffff;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  letter-spacing: 1px;
  margin: 0;
}

.help-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.help-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: rgba(0, 255, 255, 0.03);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 4px;
  transition: all 0.15s ease;
}

.help-item:hover {
  background: rgba(0, 255, 255, 0.08);
  border-color: rgba(0, 255, 255, 0.2);
}

.help-icon {
  width: 24px;
  text-align: center;
  font-size: 14px;
  color: #00ff88;
  text-shadow: 0 0 8px rgba(0, 255, 136, 0.4);
}

.help-text {
  font-size: 11px;
  color: #a1a1aa;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}
</style>
