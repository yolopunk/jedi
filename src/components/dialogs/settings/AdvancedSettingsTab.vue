<template>
  <div class="settings-section">
    <div class="setting-item">
      <div class="setting-icon"><v-icon :icon="mdiFileDocumentOutline" size="18" /></div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.hostsPath') }}</div>
      </div>
      <div class="setting-action hosts-path-action">
        <div class="input-wrapper small">
          <input type="text" readonly :value="hostsPath" class="console-input" />
        </div>
        <button
          class="console-btn small ml-2"
          :aria-label="t('wallpapers.openFolder')"
          :title="t('wallpapers.openFolder')"
          @click="openHostsFile"
        >
          <v-icon :icon="mdiFolderOpenOutline" size="16" />
        </button>
      </div>
    </div>
    <div v-if="pathLoadFailed" class="path-error" role="alert">
      {{ t('settings.pathLoadFailed') }}
    </div>

    <div class="setting-item">
      <div class="setting-icon"><v-icon :icon="mdiBackupRestore" size="18" /></div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.backup') }}</div>
        <div class="setting-hint">{{ t('settings.unavailableHint') }}</div>
      </div>
      <div class="setting-action">
        <button class="console-btn small" disabled :title="t('settings.unavailableHint')">{{ t('settings.backupBtn') }}</button>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-icon"><v-icon :icon="mdiAutorenew" size="18" /></div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.reset') }}</div>
        <div class="setting-hint">{{ t('settings.unavailableHint') }}</div>
      </div>
      <div class="setting-action">
        <button class="console-btn danger small" disabled :title="t('settings.unavailableHint')">{{ t('settings.resetBtn') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  mdiAutorenew,
  mdiBackupRestore,
  mdiFileDocumentOutline,
  mdiFolderOpenOutline,
} from '@mdi/js'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getHostsPath } from '@/api/hosts'
import { showInFolder } from '@/api/wallpaper'

const { t } = useI18n()

const hostsPath = ref('')
const pathLoadFailed = ref(false)

onMounted(async () => {
  try {
    hostsPath.value = await getHostsPath()
  } catch (error) {
    console.error('Failed to get hosts file path:', error)
    pathLoadFailed.value = true
  }
})

async function openHostsFile() {
  if (pathLoadFailed.value) return
  try {
    await showInFolder(hostsPath.value)
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
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.setting-hint {
  font-size: 12px;
  color: var(--text-subtle);
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.path-error {
  padding: 0 12px 4px;
  font-size: 12px;
  color: var(--danger);
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
