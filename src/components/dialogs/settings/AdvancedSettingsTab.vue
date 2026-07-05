<template>
  <div class="settings-section">
    <div class="setting-item">
      <div class="setting-icon">📄</div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.hostsPath') }}</div>
      </div>
      <div class="setting-action hosts-path-action">
        <div class="input-wrapper small">
          <input type="text" readonly :value="hostsPath" class="console-input" />
        </div>
        <button class="console-btn small ml-2" @click="openHostsFile" :title="t('wallpapers.openFolder')">
          <span>📂</span>
        </button>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-icon">↩</div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.backup') }}</div>
      </div>
      <div class="setting-action">
        <button class="console-btn small">{{ t('settings.backupBtn') }}</button>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-icon">🔄</div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.reset') }}</div>
      </div>
      <div class="setting-action">
        <button class="console-btn danger small">{{ t('settings.resetBtn') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { showInFolder } from '@/api/wallpaper'

const { t } = useI18n()

const hostsPath = '/etc/hosts'

async function openHostsFile() {
  try {
    await showInFolder(hostsPath)
  } catch (error) {
    console.error('Failed to open hosts file:', error)
  }
}
</script>

<style scoped>
.setting-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 4px;
  transition: background-color 0.15s ease;
}

.setting-item:not(.no-hover):hover {
  background: rgb(var(--accent-rgb) / 0.03);
}

.setting-icon {
  width: 24px;
  text-align: center;
  font-size: 16px;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--border);
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.setting-action {
  display: flex;
  align-items: center;
  gap: 8px;
}

.input-wrapper.small {
  padding: 4px 8px;
}

.input-wrapper.small .console-input {
  font-size: 11px;
  padding: 4px 8px;
}
</style>
