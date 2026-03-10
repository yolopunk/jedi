<template>
  <v-dialog v-model="dialogModel" max-width="800" class="jedi-dialog-card">
    <v-card class="jedi-dialog-card">
      <v-toolbar color="surface" class="px-4 jedi-dialog-header border-b">
        <v-icon :icon="mdiCog" color="primary" class="mr-2"></v-icon>
        <v-toolbar-title class="font-weight-medium">应用设置</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn :icon="mdiClose" variant="text" color="medium-emphasis" @click="dialogModel = false"></v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-tabs v-model="settingsTab" color="var(--jedi-accent)">
          <v-tab value="general">常规设置</v-tab>
          <v-tab value="wallpaper">{{ $t('settings.wallpaper') }}</v-tab>
          <v-tab value="chat">AI Chat</v-tab>
          <v-tab value="advanced">{{ $t('settings.advanced') }}</v-tab>
        </v-tabs>

        <v-window v-model="settingsTab" class="mt-4">
          <!-- General Settings -->
          <v-window-item value="general">
            <v-list>
              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiTranslate" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>语言 / Language</v-list-item-title>
                <template v-slot:append>
                  <v-menu>
                    <template v-slot:activator="{ props }">
                      <v-btn
                        color="var(--jedi-accent)"
                        variant="tonal"
                        size="small"
                        v-bind="props"
                        rounded="sm"
                      >
                        {{ currentLangLabel }}
                      </v-btn>
                    </template>
                    <v-list density="compact">
                      <v-list-item
                        v-for="lang in languages"
                        :key="lang.value"
                        :value="lang.value"
                        @click="changeLanguage(lang.value)"
                        :active="locale === lang.value"
                        color="primary"
                      >
                        <v-list-item-title>{{ lang.label }}</v-list-item-title>
                      </v-list-item>
                    </v-list>
                  </v-menu>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiLaunch" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>开机自启动</v-list-item-title>
                <template v-slot:append>
                  <v-switch
                    v-model="autostartEnabled"
                    color="var(--jedi-accent)"
                    hide-details
                    :loading="autostartLoading"
                    @update:model-value="toggleAutostart"
                  ></v-switch>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiTrayArrowDown" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>最小化到托盘</v-list-item-title>
                <template v-slot:append>
                  <v-switch color="var(--jedi-accent)" hide-details></v-switch>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiUpdate" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>自动检查更新</v-list-item-title>
                <template v-slot:append>
                  <v-switch
                    v-model="autoUpdateEnabled"
                    color="var(--jedi-accent)"
                    hide-details
                    @update:model-value="handleAutoUpdateChange"
                  ></v-switch>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiRefresh" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>{{ $t('settings.checkUpdate') }}</v-list-item-title>
                <v-list-item-subtitle v-if="hasUpdate">
                  {{ $t('settings.updateAvailable', { version: updateInfo?.version }) }}
                </v-list-item-subtitle>
                <v-list-item-subtitle v-else-if="updateLoading">
                  {{ $t('settings.updateChecking') }}
                </v-list-item-subtitle>
                <v-list-item-subtitle v-else>
                  {{ formatLastCheckTime() }}
                </v-list-item-subtitle>
                <template v-slot:append>
                  <v-btn
                    color="var(--jedi-accent)"
                    variant="tonal"
                    size="small"
                    rounded="sm"
                    :loading="updateLoading"
                    :disabled="updateLoading"
                    @click="handleManualCheck"
                  >
                    {{ $t('settings.checkUpdate') }}
                  </v-btn>
                </template>
              </v-list-item>
            </v-list>

            <!-- Update Dialog -->
            <UpdateDialog
              v-if="updateInfo"
              v-model="showUpdateDialog"
              :update-info="updateInfo"
              :is-installing="isInstalling"
              @install="handleInstallUpdate"
            ></UpdateDialog>
          </v-window-item>

          <!-- Wallpaper Settings -->
          <v-window-item value="wallpaper">
            <v-list>
              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiWallpaper" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>{{ $t('settings.wpAutoUpdate') }}</v-list-item-title>
                <template v-slot:append>
                  <v-switch
                    v-model="wallpaperSettings.autoUpdate"
                    color="var(--jedi-accent)"
                    hide-details
                    @update:model-value="saveWallpaperSettings(wallpaperSettings)"
                  ></v-switch>
                </template>
              </v-list-item>

              <v-list-item v-if="wallpaperSettings.autoUpdate">
                 <template v-slot:prepend>
                  <v-icon icon="" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>{{ $t('settings.wpFrequency') }}</v-list-item-title>
                <template v-slot:append>
                  <v-text-field
                    v-model.number="wallpaperSettings.frequencyDays"
                    type="number"
                    min="1"
                    variant="outlined"
                    density="compact"
                    hide-details
                    style="width: 100px"
                    @update:model-value="saveWallpaperSettings(wallpaperSettings)"
                  ></v-text-field>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon icon="" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>{{ $t('settings.wpCategories') }}</v-list-item-title>
                <template v-slot:append>
                  <v-select
                    v-model="wallpaperSettings.selectedCategories"
                    :items="allCategories"
                    multiple
                    chips
                    variant="outlined"
                    density="compact"
                    hide-details
                    style="width: 250px"
                    @update:model-value="saveWallpaperSettings(wallpaperSettings)"
                  ></v-select>
                </template>
              </v-list-item>

               <v-list-item>
                <v-list-item-subtitle class="text-caption text-right">
                  {{ $t('settings.wpLastUpdate', { time: wallpaperSettings.lastUpdate ? new Date(wallpaperSettings.lastUpdate).toLocaleString() : 'N/A' }) }}
                </v-list-item-subtitle>
              </v-list-item>
            </v-list>
          </v-window-item>

          <!-- AI Chat Settings -->
          <v-window-item value="chat">
            <div class="chat-settings">
              <!-- API Providers -->
              <div class="settings-section">
                <h3 class="section-title">AI 提供商</h3>
                <p class="section-desc">配置您的 AI 模型提供商 API 密钥</p>

                <v-list class="provider-list" density="compact">
                  <v-list-item
                    v-for="provider in providerList"
                    :key="provider.id"
                    class="provider-item"
                    :class="{ 'configured': isProviderConfigured(provider.id) }"
                  >
                    <template v-slot:prepend>
                      <div class="provider-icon">
                        <v-icon :icon="provider.icon" size="20" />
                      </div>
                    </template>
                    <v-list-item-title>{{ provider.name }}</v-list-item-title>
                    <template v-slot:append>
                      <div class="provider-status">
                        <v-chip
                          v-if="isProviderConfigured(provider.id)"
                          size="small"
                          color="success"
                          variant="flat"
                        >
                          已配置
                        </v-chip>
                        <v-chip
                          v-else
                          size="small"
                          variant="outlined"
                        >
                          未配置
                        </v-chip>
                      </div>
                      <v-btn
                        size="small"
                        variant="text"
                        @click="openProviderConfig(provider.id)"
                      >
                        {{ isProviderConfigured(provider.id) ? '编辑' : '配置' }}
                      </v-btn>
                    </template>
                  </v-list-item>
                </v-list>
              </div>

              <!-- Chat Settings -->
              <div class="settings-section mt-6">
                <h3 class="section-title">对话设置</h3>
                <p class="section-desc">配置 AI 对话的默认参数</p>

                <v-list>
                  <v-list-item>
                    <template v-slot:prepend>
                      <v-icon :icon="mdiThermometer" color="var(--jedi-primary)" class="mr-3"></v-icon>
                    </template>
                    <v-list-item-title>温度 (Temperature)</v-list-item-title>
                    <v-list-item-subtitle>控制输出的随机性，值越高越有创意</v-list-item-subtitle>
                    <template v-slot:append>
                      <div class="slider-container">
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
                    </template>
                  </v-list-item>

                  <v-list-item>
                    <template v-slot:prepend>
                      <v-icon :icon="mdiText" color="var(--jedi-primary)" class="mr-3"></v-icon>
                    </template>
                    <v-list-item-title>最大令牌数</v-list-item-title>
                    <v-list-item-subtitle>单次响应的最大长度</v-list-item-subtitle>
                    <template v-slot:append>
                      <v-text-field
                        v-model.number="chatSettings.maxTokens"
                        type="number"
                        :min="256"
                        :max="128000"
                        variant="outlined"
                        density="compact"
                        hide-details
                        style="width: 120px"
                        @update:model-value="saveChatSettings"
                      />
                    </template>
                  </v-list-item>

                  <v-list-item>
                    <template v-slot:prepend>
                      <v-icon :icon="mdiLightningBolt" color="var(--jedi-primary)" class="mr-3"></v-icon>
                    </template>
                    <v-list-item-title>流式响应</v-list-item-title>
                    <v-list-item-subtitle>实时显示 AI 响应内容</v-list-item-subtitle>
                    <template v-slot:append>
                      <v-switch
                        v-model="chatSettings.streamEnabled"
                        color="var(--jedi-accent)"
                        hide-details
                        @update:model-value="saveChatSettings"
                      />
                    </template>
                  </v-list-item>
                </v-list>
              </div>

              <!-- MCP Servers -->
              <div class="settings-section mt-6">
                <h3 class="section-title">MCP 服务器</h3>
                <p class="section-desc">配置 MCP (Model Context Protocol) 服务器</p>

                <v-list>
                  <v-list-item
                    v-for="server in mcpServers"
                    :key="server.id"
                  >
                    <template v-slot:prepend>
                      <v-icon :icon="server.icon || mdiPuzzle" size="20" />
                    </template>
                    <v-list-item-title>{{ server.name }}</v-list-item-title>
                    <v-list-item-subtitle v-if="server.description">{{ server.description }}</v-list-item-subtitle>
                    <template v-slot:append>
                      <v-switch
                        v-model="server.enabled"
                        color="primary"
                        hide-details
                        @update:model-value="toggleMcpServer(server.id)"
                      />
                    </template>
                  </v-list-item>
                </v-list>
              </div>
            </div>
          </v-window-item>

          <!-- Advanced Settings -->
          <v-window-item value="advanced">
            <v-list>
              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiFileDocument" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>Hosts 文件路径</v-list-item-title>
                <template v-slot:append>
                  <v-text-field
                    variant="outlined"
                    density="compact"
                    hide-details
                    readonly
                    value="/etc/hosts"
                    style="width: 250px"
                  ></v-text-field>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiBackupRestore" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>备份设置</v-list-item-title>
                <template v-slot:append>
                  <v-btn color="var(--jedi-accent)" variant="tonal" size="small" rounded="sm">备份</v-btn>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiRefresh" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>重置应用</v-list-item-title>
                <template v-slot:append>
                  <v-btn color="var(--jedi-danger)" variant="tonal" size="small" rounded="sm">重置</v-btn>
                </template>
              </v-list-item>
            </v-list>
          </v-window-item>
        </v-window>
      </v-card-text>
      <v-card-actions class="pa-4 pt-0">
        <v-spacer></v-spacer>
        <v-btn variant="text" @click="dialogModel = false" rounded="sm" class="mr-2">
          关闭
        </v-btn>
      </v-card-actions>
    </v-card>

    <!-- Provider Config Dialog -->
    <v-dialog v-model="showProviderDialog" max-width="500">
      <v-card>
        <v-card-title>
          {{ currentProvider?.name }} 配置
        </v-card-title>
        <v-card-text>
          <v-text-field
            v-model="providerConfig.apiKey"
            :label="`${currentProvider?.name} API Key`"
            variant="outlined"
            type="password"
            :prepend-inner-icon="mdiKey"
            class="mb-3"
          />
          <v-text-field
            v-model="providerConfig.endpoint"
            label="API Endpoint (可选)"
            variant="outlined"
            :prepend-inner-icon="mdiWeb"
            :placeholder="currentProvider?.defaultEndpoint"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showProviderDialog = false">取消</v-btn>
          <v-btn
            v-if="hasProviderKey(currentProvider?.id)"
            color="error"
            variant="text"
            @click="deleteProviderKey"
          >
            删除
          </v-btn>
          <v-btn color="primary" @click="saveProviderConfig">保存</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useStorage } from '@/composables/useStorage'
import {
  mdiCog,
  mdiClose,
  mdiLaunch,
  mdiTrayArrowDown,
  mdiUpdate,
  mdiFileDocument,
  mdiBackupRestore,
  mdiRefresh,
  mdiTranslate,
  mdiWallpaper,
  mdiKey,
  mdiWeb,
  mdiThermometer,
  mdiText,
  mdiLightningBolt,
  mdiPuzzle,
  mdiOpenid,
  mdiGoogle,
  mdiBrain,
  mdiChevronRight
} from '@mdi/js'
import { enableAutostart, disableAutostart, isAutostartEnabled } from '@/api/app'
import { useWallpaper } from '@/composables/useWallpaper'
import { getWallpapers } from '@/api/wallpaper'
import { useUpdate } from '@/composables/useUpdate'
import UpdateDialog from '@/components/dialogs/UpdateDialog.vue'
import { useAiChatStore, type McpServer } from '@/stores/aiChat'

const store = useAiChatStore()

const { locale } = useI18n()
const { setItem } = useStorage()
const { settings: wallpaperSettings, saveSettings: saveWallpaperSettings } = useWallpaper()
const allCategories = ref<string[]>([])

// Language options
const languages = [
  { label: '简体中文', value: 'zh' },
  { label: 'English', value: 'en' }
]

const currentLangLabel = computed(() => {
  return languages.find(l => l.value === locale.value)?.label || '简体中文'
})

const changeLanguage = async (lang: string) => {
  locale.value = lang
  await setItem('language', lang)
}

// Props and emits
const props = defineProps<{
  modelValue: boolean;
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>()

// Dialog state
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// Settings tab
const settingsTab = ref('general')

// Autostart related state
const autostartEnabled = ref(false)
const autostartLoading = ref(false)

// Update composable
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

// Toggle autostart
async function toggleAutostart(value: boolean | null) {
  if (value === null) return
  try {
    autostartLoading.value = true
    if (value) {
      await enableAutostart()
    } else {
      await disableAutostart()
    }
  } catch (error) {
    console.error('切换自启动状态失败:', error)
    autostartEnabled.value = !value
  } finally {
    autostartLoading.value = false
  }
}

// Check autostart status
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

// AI Chat Settings
const providerList = [
  { id: 'openai', name: 'OpenAI', icon: mdiOpenid, defaultEndpoint: 'https://api.openai.com/v1' },
  { id: 'anthropic', name: 'Anthropic', icon: mdiBrain, defaultEndpoint: 'https://api.anthropic.com' },
  { id: 'google', name: 'Google (Gemini)', icon: mdiGoogle, defaultEndpoint: 'https://generativelanguage.googleapis.com' },
  { id: 'deepseek', name: 'DeepSeek', icon: mdiChevronRight, defaultEndpoint: 'https://api.deepseek.com' },
]

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
  // Sync local state
  const server = mcpServers.value.find(s => s.id === serverId)
  if (server) {
    server.enabled = !server.enabled
  }
}

// Load AI Chat settings
function loadAiChatSettings() {
  chatSettings.value.temperature = store.temperature
  chatSettings.value.maxTokens = store.maxTokens
  chatSettings.value.streamEnabled = store.streamEnabled
  mcpServers.value = [...store.mcpServers]
}

// Watch dialog open/close
watch(dialogModel, (newVal) => {
  if (newVal) {
    loadAiChatSettings()
    store.loadProviders()
  }
})

onMounted(async () => {
  await checkAutostartStatus()

  // Load wallpaper settings and categories
  const { loadSettings } = useWallpaper()
  await loadSettings()

  try {
    const wallpapers = await getWallpapers()
    const categories = new Set(wallpapers.map(w => w.category))
    allCategories.value = Array.from(categories).sort()
  } catch (e) {
    console.error('Failed to load wallpaper categories', e)
  }

  // Load AI Chat store
  store.loadSettings()
})
</script>

<style scoped>
.save-btn {
  transition: all 0.2s;
}

.save-btn:hover {
  background: rgba(59, 130, 246, 0.1);
  transform: scale(1.05);
}

/* AI Chat Settings */
.chat-settings {
  padding: 8px 0;
}

.settings-section {
  margin-bottom: 8px;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  margin: 0 0 4px;
  color: rgb(var(--v-theme-on-surface));
}

.section-desc {
  font-size: 13px;
  color: rgba(var(--v-theme-on-surface), 0.6);
  margin: 0 0 16px;
}

.provider-list {
  background: rgba(var(--v-theme-surface-variant), 0.3);
  border-radius: 12px;
  overflow: hidden;
}

.provider-item {
  transition: background-color 0.15s ease;
}

.provider-item:hover {
  background: rgba(var(--v-theme-on-surface), 0.05);
}

.provider-item.configured {
  background: rgba(var(--v-theme-primary), 0.05);
}

.provider-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: rgba(var(--v-theme-surface-variant), 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgb(var(--v-theme-primary));
}

.provider-status {
  margin-right: 8px;
}

.slider-container {
  display: flex;
  align-items: center;
  gap: 12px;
}

.slider-value {
  font-size: 13px;
  font-weight: 600;
  color: rgb(var(--v-theme-primary));
  min-width: 40px;
  text-align: right;
}
</style>
