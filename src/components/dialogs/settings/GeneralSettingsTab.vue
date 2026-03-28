<template>
  <div class="settings-section">
    <div class="setting-item">
      <div class="setting-icon">🌐</div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.language') }}</div>
      </div>
      <div class="setting-action">
        <v-menu location="bottom end">
          <template v-slot:activator="{ props }">
            <button v-bind="props" class="console-btn small">
              {{ currentLangLabel }}
            </button>
          </template>
          <div class="console-menu">
            <div
              v-for="lang in languages"
              :key="lang.value"
              class="menu-item"
              :class="{ active: locale === lang.value }"
              @click="changeLanguage(lang.value)"
            >
              <span class="menu-check">{{ locale === lang.value ? '▣' : '▢' }}</span>
              <span class="menu-text">{{ lang.label }}</span>
            </div>
          </div>
        </v-menu>
      </div>
    </div>

    <!-- Theme Setting -->
    <div class="setting-item">
      <div class="setting-icon">🎨</div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.theme') }}</div>
      </div>
      <div class="setting-action">
        <v-menu location="bottom end">
          <template v-slot:activator="{ props }">
            <button v-bind="props" class="console-btn small">
              {{ currentThemeLabel }}
            </button>
          </template>
          <div class="console-menu">
            <div
              v-for="theme in themeModes"
              :key="theme.value"
              class="menu-item"
              :class="{ active: themeMode === theme.value }"
              @click="setTheme(theme.value as 'light' | 'dark' | 'system')"
            >
              <span class="menu-icon">{{ theme.icon }}</span>
              <span class="menu-text">{{ theme.label }}</span>
            </div>
          </div>
        </v-menu>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-icon">🚀</div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.autostart') }}</div>
      </div>
      <div class="setting-action">
        <div
          class="toggle-switch"
          :class="{ active: autostartEnabled }"
          @click="!autostartLoading && toggleAutostart(!autostartEnabled)"
        >
          <div class="toggle-handle"></div>
        </div>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-icon">📥</div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.minimizeToTray') }}</div>
      </div>
      <div class="setting-action">
        <div class="toggle-switch">
          <div class="toggle-handle"></div>
        </div>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-icon">🔄</div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.autoUpdate') }}</div>
      </div>
      <div class="setting-action">
        <div
          class="toggle-switch"
          :class="{ active: autoUpdateEnabled }"
          @click="handleAutoUpdateChange(!autoUpdateEnabled)"
        >
          <div class="toggle-handle"></div>
        </div>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-icon">↻</div>
      <div class="setting-info">
        <div class="setting-label">{{ $t('settings.checkUpdate') }}</div>
        <div class="setting-subtitle" v-if="hasUpdate">
          {{ $t('settings.updateAvailable', { version: updateInfo?.version }) }}
        </div>
        <div class="setting-subtitle" v-else-if="updateLoading">
          {{ $t('settings.updateChecking') }}
        </div>
        <div class="setting-subtitle" v-else>
          {{ formatLastCheckTime() }}
        </div>
      </div>
      <div class="setting-action">
        <button class="console-btn small" :disabled="updateLoading" @click="handleManualCheck">
          <span v-if="!updateLoading">{{ $t('settings.checkUpdate') }}</span>
          <span v-else>...</span>
        </button>
      </div>
    </div>
  </div>

  <!-- Update Dialog -->
  <UpdateDialog
    v-if="updateInfo"
    v-model="showUpdateDialog"
    :update-info="updateInfo"
    :is-installing="isInstalling"
    @install="handleInstallUpdate"
  ></UpdateDialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useStorage } from '@/composables/useStorage'
import { enableAutostart, disableAutostart, isAutostartEnabled } from '@/api/app'
import { useUpdate } from '@/composables/useUpdate'
import { useTheme } from '@/composables/useTheme'
import UpdateDialog from '@/components/dialogs/UpdateDialog.vue'

const { locale, t } = useI18n()
const { setItem } = useStorage()
const { themeMode, setTheme } = useTheme()

const languages = computed(() => [
  { label: '简体中文', value: 'zh' },
  { label: 'English', value: 'en' }
])

const themeModes = computed(() => [
  { label: t('settings.themeDark'), value: 'dark', icon: '🌙' },
  { label: t('settings.themeLight'), value: 'light', icon: '☀️' },
  { label: t('settings.themeSystem'), value: 'system', icon: '💻' }
])

const currentThemeLabel = computed(() => {
  return themeModes.value.find(m => m.value === themeMode.value)?.label || t('settings.themeSystem')
})

const currentLangLabel = computed(() => {
  return languages.value.find((l: { value: string }) => l.value === locale.value)?.label || '简体中文'
})

const changeLanguage = async (lang: string) => {
  locale.value = lang
  await setItem('language', lang)
}

const autostartEnabled = ref(false)
const autostartLoading = ref(false)

const {
  hasUpdate,
  updateInfo,
  isChecking: updateLoading,
  isInstalling,
  autoUpdateEnabled,
  checkForUpdate,
  installUpdate,
  formatLastCheckTime
} = useUpdate()

const showUpdateDialog = ref(false)

const handleAutoUpdateChange = (value: boolean | null) => {
  if (value !== null) {
    autoUpdateEnabled.value = value
  }
}

const handleManualCheck = async () => {
  await checkForUpdate()
  if (hasUpdate.value) {
    showUpdateDialog.value = true
  }
}

const handleInstallUpdate = async () => {
  try {
    await installUpdate()
    showUpdateDialog.value = false
  } catch (error) {
    console.error('Failed to install update:', error)
  }
}

async function toggleAutostart(value: boolean | null) {
  if (value === null) return
  try {
    autostartLoading.value = true
    if (value) {
      await enableAutostart()
      autostartEnabled.value = true
    } else {
      await disableAutostart()
      autostartEnabled.value = false
    }
  } catch (error) {
    console.error('切换自启动状态失败:', error)
  } finally {
    autostartLoading.value = false
  }
}

async function checkAutostartStatus() {
  try {
    autostartLoading.value = true
    const enabled = await isAutostartEnabled()
    autostartEnabled.value = enabled
  } catch (error) {
    console.error('检查自启动状态失败:', error)
  } finally {
    autostartLoading.value = false
  }
}

onMounted(async () => {
  await checkAutostartStatus()
})
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
  background: rgba(0, 255, 255, 0.03);
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
  color: #e4e4e7;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.setting-subtitle {
  font-size: 10px;
  color: #52525b;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.setting-action {
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-action.slider-action {
  gap: 12px;
}

.slider-value {
  font-size: 12px;
  font-weight: 600;
  color: #00ffff;
  font-family: 'JetBrains Mono', monospace;
  min-width: 40px;
  text-align: right;
}

/* Toggle Switch */
.toggle-switch {
  width: 40px;
  height: 22px;
  background: rgba(82, 82, 91, 0.5);
  border-radius: 12px;
  position: relative;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid rgba(82, 82, 91, 0.5);
}

.toggle-switch.active {
  background: rgba(0, 255, 136, 0.15);
  border-color: rgba(0, 255, 136, 0.5);
}

.toggle-handle {
  position: absolute;
  width: 16px;
  height: 16px;
  background: #52525b;
  border-radius: 50%;
  top: 2px;
  left: 2px;
  transition: all 0.2s ease;
}

.toggle-switch.active .toggle-handle {
  background: #00ff88;
  box-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
  left: 20px;
}

/* =========================================
   Light Theme Styles
   ========================================= */
.light-theme .setting-item {
  border-bottom-color: #d4a574;
}

.light-theme .setting-label {
  color: #3d2914;
}

.light-theme .setting-subtitle {
  color: #6b4423;
}

.light-theme .setting-icon {
  color: #cd7f32;
}

.light-theme .toggle-switch {
  background: #d4a574;
}

.light-theme .toggle-switch.active {
  background: #cd7f32;
}

.light-theme .toggle-switch .toggle-handle {
  background: #ffffff;
}

.light-theme .console-menu {
  background: #efe0cc !important;
  border-color: rgba(184, 134, 11, 0.3) !important;
}

.light-theme .menu-item {
  color: #6b4423;
}

.light-theme .menu-item:hover {
  background: rgba(205, 127, 50, 0.12);
  color: #cd7f32;
}

.light-theme .menu-check {
  color: #daa520;
}
</style>
