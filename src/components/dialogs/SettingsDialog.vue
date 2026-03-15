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
          <!-- General Settings -->
          <div v-if="settingsTab === 'general'" class="settings-section">
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

          <!-- Wallpaper Settings -->
          <div v-if="settingsTab === 'wallpaper'" class="settings-section">
            <div class="setting-item">
              <div class="setting-icon">🖼</div>
              <div class="setting-info">
                <div class="setting-label">{{ $t('settings.wpAutoUpdate') }}</div>
              </div>
              <div class="setting-action">
                <div
                  class="toggle-switch"
                  :class="{ active: wallpaperSettings.autoUpdate }"
                  @click="wallpaperSettings.autoUpdate = !wallpaperSettings.autoUpdate; saveWallpaperSettings(wallpaperSettings)"
                >
                  <div class="toggle-handle"></div>
                </div>
              </div>
            </div>

            <div class="setting-item" v-if="wallpaperSettings.autoUpdate">
              <div class="setting-icon"></div>
              <div class="setting-info">
                <div class="setting-label">{{ $t('settings.wpFrequency') }}</div>
              </div>
              <div class="setting-action">
                <div class="input-wrapper small">
                  <input
                    v-model.number="wallpaperSettings.frequencyDays"
                    type="number"
                    min="1"
                    class="console-input"
                    @input="saveWallpaperSettings(wallpaperSettings)"
                  />
                </div>
              </div>
            </div>

            <div class="setting-item">
              <div class="setting-icon"></div>
              <div class="setting-info">
                <div class="setting-label">{{ $t('settings.wpCategories') }}</div>
              </div>
              <div class="setting-action">
                <v-menu location="bottom end">
                  <template v-slot:activator="{ props }">
                    <button v-bind="props" class="console-btn small">
                      {{ t('settings.categories', { n: wallpaperSettings.selectedCategories?.length || 0 }) }}
                    </button>
                  </template>
                  <div class="console-menu">
                    <div
                      v-for="cat in allCategories"
                      :key="cat"
                      class="menu-item"
                      :class="{ active: wallpaperSettings.selectedCategories?.includes(cat) }"
                      @click="toggleCategory(cat)"
                    >
                      <span class="menu-check">{{ wallpaperSettings.selectedCategories?.includes(cat) ? '▣' : '▢' }}</span>
                      <span class="menu-text">{{ cat }}</span>
                    </div>
                  </div>
                </v-menu>
              </div>
            </div>

            <div class="setting-item no-hover">
              <div class="setting-icon"></div>
              <div class="setting-info">
                <div class="setting-subtitle text-right">
                  {{ $t('settings.wpLastUpdate', { time: wallpaperSettings.lastUpdate ? new Date(wallpaperSettings.lastUpdate).toLocaleString() : 'N/A' }) }}
                </div>
              </div>
            </div>
          </div>

          <!-- AI Chat Settings -->
          <div v-if="settingsTab === 'chat'" class="settings-section">
            <div class="section-header">
              <span class="section-title">{{ t('settings.aiProvider') }}</span>
              <span class="section-desc">{{ t('settings.providerConfig') }}</span>
            </div>

            <div class="provider-list">
              <div
                v-for="provider in providerList"
                :key="provider.id"
                class="provider-item"
                :class="{ 'configured': isProviderConfigured(provider.id) }"
              >
                <div class="provider-icon">{{ providerIcon(provider.id) }}</div>
                <div class="provider-name">{{ provider.name }}</div>
                <div class="provider-status">
                  <span
                    v-if="isProviderConfigured(provider.id)"
                    class="status-chip success"
                  >
                    {{ t('settings.configured') }}
                  </span>
                  <span v-else class="status-chip">
                    {{ t('settings.notConfigured') }}
                  </span>
                </div>
                <button class="console-btn small" @click="openProviderConfig(provider.id)">
                  {{ isProviderConfigured(provider.id) ? t('settings.edit') : t('settings.configure') }}
                </button>
              </div>
            </div>

            <div class="divider-line my-4"></div>

            <div class="section-header">
              <span class="section-title">{{ t('settings.chatSettings') }}</span>
              <span class="section-desc">{{ t('settings.chatSettingsDesc') }}</span>
            </div>

            <div class="setting-item">
              <div class="setting-icon">🌡</div>
              <div class="setting-info">
                <div class="setting-label">{{ t('settings.temperature') }}</div>
                <div class="setting-subtitle">{{ t('settings.temperatureDesc') }}</div>
              </div>
              <div class="setting-action slider-action">
                <span class="slider-value">{{ chatSettings.temperature.toFixed(2) }}</span>
                <v-slider
                  v-model="chatSettings.temperature"
                  :min="0"
                  :max="2"
                  :step="0.1"
                  density="compact"
                  color="primary"
                  style="width: 150px"
                  @update:model-value="saveChatSettings"
                />
              </div>
            </div>

            <div class="setting-item">
              <div class="setting-icon">📝</div>
              <div class="setting-info">
                <div class="setting-label">{{ t('settings.maxTokens') }}</div>
                <div class="setting-subtitle">{{ t('settings.maxTokensDesc') }}</div>
              </div>
              <div class="setting-action">
                <div class="input-wrapper small">
                  <input
                    v-model.number="chatSettings.maxTokens"
                    type="number"
                    min="256"
                    max="128000"
                    class="console-input"
                    @input="saveChatSettings"
                  />
                </div>
              </div>
            </div>

            <div class="setting-item">
              <div class="setting-icon">⚡</div>
              <div class="setting-info">
                <div class="setting-label">{{ t('settings.streamResponse') }}</div>
                <div class="setting-subtitle">{{ t('settings.streamResponseDesc') }}</div>
              </div>
              <div class="setting-action">
                <div
                  class="toggle-switch"
                  :class="{ active: chatSettings.streamEnabled }"
                  @click="chatSettings.streamEnabled = !chatSettings.streamEnabled; saveChatSettings()"
                >
                  <div class="toggle-handle"></div>
                </div>
              </div>
            </div>

            <div class="divider-line my-4"></div>

            <div class="section-header">
              <span class="section-title">{{ t('settings.mcpServers') }}</span>
              <span class="section-desc">{{ t('settings.mcpServersDesc') }}</span>
            </div>

            <div class="setting-item" v-for="server in mcpServers" :key="server.id">
              <div class="setting-icon">🧩</div>
              <div class="setting-info">
                <div class="setting-label">{{ server.name }}</div>
                <div class="setting-subtitle" v-if="server.description">{{ server.description }}</div>
              </div>
              <div class="setting-action">
                <div
                  class="toggle-switch"
                  :class="{ active: server.enabled }"
                  @click="toggleMcpServer(server.id)"
                >
                  <div class="toggle-handle"></div>
                </div>
              </div>
            </div>
          </div>

          <!-- Advanced Settings -->
          <div v-if="settingsTab === 'advanced'" class="settings-section">
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
        </div>
      </v-card-text>
      <v-card-actions class="console-card-actions">
        <v-spacer></v-spacer>
        <button class="console-btn" @click="dialogModel = false">
          <span class="btn-text">{{ t('settings.close') }}</span>
        </button>
      </v-card-actions>
    </v-card>

    <!-- Update Dialog -->
    <UpdateDialog
      v-if="updateInfo"
      v-model="showUpdateDialog"
      :update-info="updateInfo"
      :is-installing="isInstalling"
      @install="handleInstallUpdate"
    ></UpdateDialog>

    <!-- Provider Config Dialog -->
    <v-dialog v-model="showProviderDialog" max-width="500">
      <v-card class="scifi-card">
        <v-card-title class="console-title-bar">
          <span class="dialog-title">[ {{ currentProvider?.name }}_CONFIG ]</span>
        </v-card-title>
        <v-card-text class="console-card-text">
          <div class="input-wrapper mb-3">
            <span class="input-prompt">>></span>
            <input
              v-model="providerConfig.apiKey"
              type="password"
              class="console-input"
              :placeholder="t('settings.apiKey')"
            />
          </div>
          <div class="input-wrapper">
            <span class="input-prompt">>></span>
            <input
              v-model="providerConfig.endpoint"
              type="text"
              class="console-input"
              :placeholder="t('settings.apiEndpoint')"
            />
          </div>
        </v-card-text>
        <v-card-actions class="console-card-actions">
          <v-spacer />
          <button class="console-btn" @click="showProviderDialog = false">{{ t('common.cancel') }}</button>
          <button
            v-if="hasProviderKey(currentProvider?.id)"
            class="console-btn danger ml-2"
            @click="deleteProviderKey"
          >
            {{ t('common.delete') }}
          </button>
          <button class="console-btn primary ml-2" @click="saveProviderConfig">{{ t('common.save') }}</button>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useStorage } from '@/composables/useStorage'
import { enableAutostart, disableAutostart, isAutostartEnabled } from '@/api/app'
import { useWallpaper } from '@/composables/useWallpaper'
import { getWallpapers } from '@/api/wallpaper'
import { useUpdate } from '@/composables/useUpdate'
import { useTheme } from '@/composables/useTheme'
import { showInFolder } from '@/api/wallpaper'
import UpdateDialog from '@/components/dialogs/UpdateDialog.vue'
import { useAiChatStore, type McpServer } from '@/stores/aiChat'

const store = useAiChatStore()

const { locale, t } = useI18n()
const { setItem } = useStorage()
const { themeMode, setTheme } = useTheme()
const { settings: wallpaperSettings, saveSettings: saveWallpaperSettings } = useWallpaper()
const allCategories = ref<string[]>([])

const tabs = computed(() => [
  { value: 'general', label: t('settings.general') },
  { value: 'wallpaper', label: t('settings.wallpaper') },
  { value: 'chat', label: t('settings.chat') },
  { value: 'advanced', label: t('settings.advanced') }
])

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

const hostsPath = '/etc/hosts'

async function openHostsFile() {
  try {
    await showInFolder(hostsPath)
  } catch (error) {
    console.error('Failed to open hosts file:', error)
  }
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

function toggleCategory(cat: string) {
  if (!wallpaperSettings.value.selectedCategories) {
    wallpaperSettings.value.selectedCategories = []
  }
  const index = wallpaperSettings.value.selectedCategories.indexOf(cat)
  if (index > -1) {
    wallpaperSettings.value.selectedCategories.splice(index, 1)
  } else {
    wallpaperSettings.value.selectedCategories.push(cat)
  }
  saveWallpaperSettings(wallpaperSettings.value)
}

const providerList = [
  { id: 'openai', name: 'OpenAI', defaultEndpoint: 'https://api.openai.com/v1' },
  { id: 'anthropic', name: 'Anthropic', defaultEndpoint: 'https://api.anthropic.com' },
  { id: 'google', name: 'Google (Gemini)', defaultEndpoint: 'https://generativelanguage.googleapis.com' },
  { id: 'deepseek', name: 'DeepSeek', defaultEndpoint: 'https://api.deepseek.com' },
]

function providerIcon(id: string) {
  const icons: Record<string, string> = {
    openai: '○',
    anthropic: '◎',
    google: '⊕',
    deepseek: '▶'
  }
  return icons[id] || '○'
}

const chatSettings = ref({
  temperature: 0.7,
  maxTokens: 4096,
  streamEnabled: true,
})

const mcpServers = ref<McpServer[]>([...store.mcpServers])

const showProviderDialog = ref(false)
const currentProvider = ref<typeof providerList[0] | null>(null)
const providerConfig = ref({
  apiKey: '',
  endpoint: '',
})

function isProviderConfigured(providerId: string) {
  return store.providers.some(p => p.provider === providerId && p.has_key)
}

function hasProviderKey(providerId: string | undefined) {
  if (!providerId) return false
  return isProviderConfigured(providerId)
}

function openProviderConfig(providerId: string) {
  currentProvider.value = providerList.find(p => p.id === providerId) || null
  providerConfig.value = { apiKey: '', endpoint: '' }
  showProviderDialog.value = true
}

async function saveProviderConfig() {
  if (!currentProvider.value) return

  try {
    await store.saveApiKey(
      currentProvider.value.id,
      providerConfig.value.apiKey,
      providerConfig.value.endpoint || undefined
    )
    showProviderDialog.value = false
  } catch (e) {
    console.error('Failed to save provider config:', e)
  }
}

async function deleteProviderKey() {
  if (!currentProvider.value) return

  try {
    await store.deleteApiKey(currentProvider.value.id)
    showProviderDialog.value = false
  } catch (e) {
    console.error('Failed to delete provider key:', e)
  }
}

function saveChatSettings() {
  store.temperature = chatSettings.value.temperature
  store.maxTokens = chatSettings.value.maxTokens
  store.streamEnabled = chatSettings.value.streamEnabled
  store.saveSettings()
}

function toggleMcpServer(serverId: string) {
  store.toggleMcpServer(serverId)
  const server = mcpServers.value.find(s => s.id === serverId)
  if (server) {
    server.enabled = !server.enabled
  }
}

function loadAiChatSettings() {
  chatSettings.value.temperature = store.temperature
  chatSettings.value.maxTokens = store.maxTokens
  chatSettings.value.streamEnabled = store.streamEnabled
  mcpServers.value = [...store.mcpServers]
}

watch(dialogModel, (newVal) => {
  if (newVal) {
    loadAiChatSettings()
    store.loadProviders()
  }
})

onMounted(async () => {
  await checkAutostartStatus()

  const { loadSettings } = useWallpaper()
  await loadSettings()

  try {
    const wallpapers = await getWallpapers()
    const categories = new Set(wallpapers.map(w => w.category))
    allCategories.value = Array.from(categories).sort()
  } catch (e) {
    console.error('Failed to load wallpaper categories', e)
  }

  store.loadSettings()
})
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

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

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

.divider-line {
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(0, 255, 255, 0.2), transparent);
  margin: 16px 0;
}

.my-4 {
  margin-top: 16px;
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 0 8px;
}

.section-title {
  font-size: 12px;
  font-weight: 700;
  color: #00ff88;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  letter-spacing: 1px;
}

.section-desc {
  font-size: 10px;
  color: #52525b;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.provider-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.provider-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: rgba(0, 255, 255, 0.03);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 4px;
  transition: all 0.15s ease;
}

.provider-item:hover {
  background: rgba(0, 255, 255, 0.08);
  border-color: rgba(0, 255, 255, 0.2);
}

.provider-item.configured {
  background: rgba(0, 255, 136, 0.05);
  border-color: rgba(0, 255, 136, 0.2);
}

.provider-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 255, 255, 0.05);
  border-radius: 6px;
  font-size: 14px;
}

.provider-name {
  flex: 1;
  font-size: 12px;
  font-weight: 600;
  color: #e4e4e7;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.provider-status {
  margin-right: 8px;
}

.status-chip {
  font-size: 10px;
  padding: 4px 10px;
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 4px;
  color: #52525b;
  font-family: 'JetBrains Mono', monospace;
  letter-spacing: 1px;
}

.status-chip.success {
  border-color: rgba(0, 255, 136, 0.4);
  color: #00ff88;
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

/* Small input wrapper */
.input-wrapper.small {
  padding: 4px 8px;
}

.input-wrapper.small .console-input {
  font-size: 11px;
  padding: 4px 8px;
}

.text-right {
  text-align: right;
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
   Light Theme Styles (Blue Tech)
   ========================================= */
.light-theme .scifi-card {
  background: linear-gradient(135deg, #E8EEF5 0%, #DDE5EF 100%);
  border-color: rgba(30, 136, 229, 0.25);
}

.light-theme .scifi-card::before {
  background: linear-gradient(90deg, transparent, #1E88E5, transparent);
}

.light-theme .console-title-bar {
  background: linear-gradient(180deg, #E8EEF5 0%, #DDE5EF 100%);
  border-bottom-color: rgba(30, 136, 229, 0.25);
}

.light-theme .dialog-title {
  color: #1E88E5;
  text-shadow: 0 0 10px rgba(30, 136, 229, 0.4);
}

.light-theme .console-card-text {
  background: #E8EEF5;
}

.light-theme .settings-tabs {
  background: #DDE5EF;
  border-bottom-color: rgba(30, 136, 229, 0.2);
}

.light-theme .tab-button {
  color: #546E7A;
}

.light-theme .tab-button:hover {
  color: #1E88E5;
}

.light-theme .tab-button.active {
  color: #1E88E5;
  border-bottom-color: #1E88E5;
}

.light-theme .tab-text {
  color: inherit;
}

.light-theme .setting-item {
  border-bottom-color: #D0DAE5;
}

.light-theme .setting-label {
  color: #1A237E;
}

.light-theme .setting-subtitle {
  color: #546E7A;
}

.light-theme .setting-icon {
  color: #1E88E5;
}

.light-theme .console-input {
  background: #FFFFFF;
  border-color: #B3E5FC;
  color: #1A237E;
}

.light-theme .console-input:focus {
  border-color: #1E88E5;
  box-shadow: 0 0 15px rgba(30, 136, 229, 0.25);
}

.light-theme .console-input::placeholder {
  color: #78909C;
}

.light-theme .input-prompt {
  color: #1E88E5;
}

.light-theme .console-btn {
  border-color: #1E88E5;
  color: #1E88E5;
  background: transparent;
}

.light-theme .console-btn:hover {
  background: rgba(30, 136, 229, 0.12);
}

.light-theme .console-btn.primary {
  border-color: #1E88E5;
  color: #FFFFFF;
  background: #1E88E5;
}

.light-theme .console-btn.primary:hover {
  background: #1565C0;
}

.light-theme .console-btn.danger {
  border-color: #E53935;
  color: #E53935;
}

.light-theme .console-btn.danger:hover {
  background: rgba(229, 57, 53, 0.1);
}

.light-theme .toggle-switch {
  background: #B3E5FC;
}

.light-theme .toggle-switch.active {
  background: #1E88E5;
}

.light-theme .toggle-switch .toggle-handle {
  background: #FFFFFF;
}

.light-theme .console-menu {
  background: #E8EEF5 !important;
  border-color: rgba(30, 136, 229, 0.25) !important;
}

.light-theme .menu-item {
  color: #546E7A;
}

.light-theme .menu-item:hover {
  background: rgba(30, 136, 229, 0.12);
  color: #1E88E5;
}

.light-theme .menu-check {
  color: #43A047;
}

.light-theme .section-header {
  border-bottom-color: rgba(30, 136, 229, 0.2);
}

.light-theme .section-title {
  color: #1E88E5;
}

.light-theme .section-desc {
  color: #546E7A;
}

.light-theme .provider-item {
  border-color: #C5D5E3;
  background: #E8EEF5;
}

.light-theme .provider-item.configured {
  border-color: rgba(67, 160, 71, 0.4);
  background: rgba(67, 160, 71, 0.08);
}

.light-theme .provider-name {
  color: #1A237E;
}

.light-theme .status-chip {
  background: #C5D5E3;
  color: #546E7A;
}

.light-theme .status-chip.success {
  background: rgba(67, 160, 71, 0.2);
  color: #00aa66;
}

.light-theme .divider-line {
  background: #e4e4e7;
}

.light-theme .tab-content::-webkit-scrollbar-thumb {
  background: rgba(8, 145, 178, 0.2);
}

.light-theme .tab-content::-webkit-scrollbar-thumb:hover {
  background: rgba(8, 145, 178, 0.3);
}
</style>
