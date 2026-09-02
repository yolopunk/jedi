<template>
  <div class="settings-section">
    <div class="section-header">
      <span class="section-title">{{ t('settings.aiProvider') }}</span>
      <span class="section-desc">{{ t('settings.providerConfig') }}</span>
    </div>

    <div class="mb-4">
      <button class="console-btn" @click="showModelsBrowser = true">
        <span class="btn-icon">📚</span>
        <span class="btn-text">BROWSE ALL MODELS</span>
      </button>
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

    <div class="divider-line my-4"></div>

    <div class="section-header">
      <span class="section-title">第三方 MCP 服务器</span>
      <span class="section-desc">接入外部 MCP server（stdio / SSE / Streamable HTTP），扩展 Agent 可用工具</span>
    </div>

    <div class="setting-item" v-for="srv in store.thirdPartyMcpServers" :key="srv.id">
      <div class="setting-icon">🔌</div>
      <div class="setting-info">
        <div class="setting-label">{{ srv.name || srv.id }}</div>
        <div class="setting-subtitle">{{ srv.command }} {{ (srv.args || []).join(' ') }}</div>
      </div>
      <div class="setting-action">
        <span class="status-chip">{{ (srv.transport || 'stdio').toUpperCase() }}</span>
        <span class="status-chip" :class="{ success: isConnected(srv.id) }">
          {{ isConnected(srv.id) ? 'CONNECTED' : 'OFFLINE' }}
        </span>
        <button class="console-btn small" @click="toggleConnect(srv)">
          {{ isConnected(srv.id) ? 'Disconnect' : 'Connect' }}
        </button>
        <button class="console-btn small danger" @click="store.removeMcpServer(srv.id)">✕</button>
      </div>
    </div>

    <div class="mcp-add-form">
      <div class="mcp-transport-tabs">
        <button
          class="mcp-tab"
          :class="{ active: newServer.transport === 'stdio' }"
          @click="newServer.transport = 'stdio'"
        >STDIO</button>
        <button
          class="mcp-tab"
          :class="{ active: newServer.transport === 'sse' }"
          @click="newServer.transport = 'sse'"
        >SSE</button>
        <button
          class="mcp-tab"
          :class="{ active: newServer.transport === 'streamable-http' }"
          @click="newServer.transport = 'streamable-http'"
        >HTTP</button>
      </div>
      <input v-model="newServer.id" class="console-input" placeholder="id (唯一)" />
      <input v-model="newServer.name" class="console-input" placeholder="名称" />
      <template v-if="newServer.transport === 'stdio'">
        <input v-model="newServer.command" class="console-input" placeholder="命令，例如 npx" />
        <input v-model="newServer.argsText" class="console-input" placeholder="参数（空格分隔）" />
      </template>
      <template v-else>
        <input
          v-model="newServer.url"
          class="console-input"
          :placeholder="newServer.transport === 'sse'
            ? 'URL，例如 http://127.0.0.1:9000/sse'
            : 'URL，例如 http://127.0.0.1:9000/mcp'"
        />
      </template>
      <button class="console-btn small primary" @click="addServer">+ 添加服务器</button>
    </div>
    <div v-if="mcpError" class="mcp-error">{{ mcpError }}</div>

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

    <!-- Models Browser Dialog -->
    <ModelsBrowser v-model="showModelsBrowser" @select="handleModelSelect" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAiChatStore, type McpServer } from '@/stores/aiChat'
import ModelsBrowser from '@/views/AiChat/ModelsBrowser.vue'
import type { ModelsDevModel } from '@/api/ai-chat'

const { t } = useI18n()
const store = useAiChatStore()

const showModelsBrowser = ref(false)

function handleModelSelect(model: ModelsDevModel, provider: string) {
  console.log('Selected model:', model, 'from provider:', provider)
}

const providerList = [
  { id: 'openai', name: 'OpenAI', defaultEndpoint: 'https://api.openai.com/v1' },
  { id: 'anthropic', name: 'Anthropic', defaultEndpoint: 'https://api.anthropic.com' },
  { id: 'google', name: 'Google (Gemini)', defaultEndpoint: 'https://generativelanguage.googleapis.com' },
  { id: 'deepseek', name: 'DeepSeek', defaultEndpoint: 'https://api.deepseek.com' },
  { id: 'openrouter', name: 'OpenRouter', defaultEndpoint: 'https://openrouter.ai/api/v1' },
  { id: 'custom', name: 'Custom', defaultEndpoint: '' },
]

function providerIcon(id: string) {
  const icons: Record<string, string> = {
    openai: '○',
    anthropic: '◎',
    google: '⊕',
    deepseek: '▶',
    openrouter: '◎',
    custom: '◈'
  }
  return icons[id] || '○'
}

const chatSettings = ref({
  temperature: 0.7,
  maxTokens: 4096,
  streamEnabled: true,
})

const mcpServers = ref<McpServer[]>([...store.mcpServers])

// 第三方 MCP server 管理
const newServer = ref<{
  transport: 'stdio' | 'sse' | 'streamable-http'
  id: string
  name: string
  command: string
  argsText: string
  url: string
}>({ transport: 'stdio', id: '', name: '', command: '', argsText: '', url: '' })
const mcpError = ref('')

function isConnected(id: string) {
  return store.mcpConnectedIds.includes(id)
}

async function toggleConnect(srv: { id: string }) {
  mcpError.value = ''
  try {
    if (isConnected(srv.id)) {
      await store.disconnectMcp(srv.id)
    } else {
      await store.connectMcp(srv as any)
    }
  } catch (e) {
    mcpError.value = String(e)
  }
}

function addServer() {
  const s = newServer.value
  if (!s.id) {
    mcpError.value = 'id 为必填'
    return
  }
  if (s.transport === 'stdio' && !s.command) {
    mcpError.value = 'stdio 需要命令'
    return
  }
  if (s.transport !== 'stdio' && !s.url) {
    mcpError.value = '远程传输需要 URL'
    return
  }
  mcpError.value = ''
  if (s.transport === 'stdio') {
    const argsText = s.argsText.trim()
    store.addMcpServer({
      id: s.id,
      name: s.name || s.id,
      transport: 'stdio',
      command: s.command,
      args: argsText ? argsText.split(/\s+/) : [],
    })
  } else {
    store.addMcpServer({
      id: s.id,
      name: s.name || s.id,
      transport: s.transport,
      url: s.url,
    })
  }
  newServer.value = { transport: s.transport, id: '', name: '', command: '', argsText: '', url: '' }
}

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

async function loadSettings() {
  loadAiChatSettings()
  store.loadMcpServers()
  await store.loadProviders()
}

defineExpose({ loadSettings })
</script>

<style scoped>
.settings-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
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

.divider-line {
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(0, 255, 255, 0.2), transparent);
  margin: 16px 0;
}

.my-4 {
  margin-top: 16px;
  margin-bottom: 16px;
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

.setting-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 4px;
  transition: background-color 0.15s ease;
}

.setting-item:hover {
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

.input-wrapper.small {
  padding: 4px 8px;
}

.input-wrapper.small .console-input {
  font-size: 11px;
  padding: 4px 8px;
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

.ml-2 {
  margin-left: 8px;
}

.mb-3 {
  margin-bottom: 12px;
}

.mb-4 {
  margin-bottom: 16px;
}

/* 第三方 MCP 添加表单 */
.mcp-add-form {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  margin-top: 4px;
  background: rgba(0, 255, 255, 0.03);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 4px;
}

.mcp-add-form .console-input {
  font-size: 11px;
  padding: 6px 8px;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(0, 255, 255, 0.15);
  border-radius: 3px;
  color: #e4e4e7;
  font-family: 'JetBrains Mono', monospace;
}

.console-btn.danger {
  color: #f87171;
  border-color: rgba(248, 113, 113, 0.3);
}

.mcp-error {
  font-size: 10px;
  color: #f87171;
  font-family: 'JetBrains Mono', monospace;
  padding: 6px 12px;
}

.mcp-transport-tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 4px;
}

.mcp-tab {
  flex: 1;
  padding: 4px 8px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  color: #52525b;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.12);
  border-radius: 3px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.mcp-tab.active {
  color: #00ffff;
  border-color: rgba(0, 255, 255, 0.4);
  background: rgba(0, 255, 255, 0.08);
}
</style>
