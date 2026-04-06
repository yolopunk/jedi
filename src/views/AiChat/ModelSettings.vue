<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="600"
  >
    <v-card class="scifi-card model-settings-dialog">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">[ MODEL_SETTINGS ]</span>
        <v-spacer />
        <button class="console-btn icon-only" @click="$emit('update:modelValue', false)">
          <span class="btn-icon">✕</span>
        </button>
      </v-card-title>

      <v-card-text class="console-card-text settings-content">
        <!-- Providers List -->
        <div class="section-header">
          <span class="section-title">CONFIGURE PROVIDERS</span>
        </div>

        <div class="providers-list">
          <div
            v-for="p in providerList"
            :key="p.id"
            class="provider-item"
            :class="{ configured: isProviderConfigured(p.id) }"
          >
            <ProviderIcon :provider-id="p.id" class="provider-icon" />
            <div class="provider-info">
              <span class="provider-name">{{ p.name }}</span>
              <span class="provider-status">
                {{ isProviderConfigured(p.id) ? 'CONFIGURED' : 'NOT CONFIGURED' }}
              </span>
            </div>
            <button class="console-btn small" @click="openConfig(p.id)">
              {{ isProviderConfigured(p.id) ? 'EDIT' : 'CONFIGURE' }}
            </button>
          </div>
        </div>

        <!-- API Key Config Dialog -->
        <v-dialog v-model="showConfigDialog" max-width="400">
          <v-card class="scifi-card">
            <v-card-title class="console-title-bar">
              <span class="dialog-title">[ {{ currentProvider?.name }}_CONFIG ]</span>
            </v-card-title>
            <v-card-text class="console-card-text">
              <div class="form-field">
                <label class="field-label">API KEY</label>
                <div class="input-wrapper">
                  <span class="input-prompt">>></span>
                  <input
                    v-model="configApiKey"
                    :type="showKey ? 'text' : 'password'"
                    class="console-input"
                    placeholder="sk-..."
                  />
                  <button class="console-btn icon-only small" @click="showKey = !showKey">
                    <span class="btn-icon">{{ showKey ? '👁' : '👁‍🗨' }}</span>
                  </button>
                </div>
              </div>
              <div class="form-field mt-3">
                <label class="field-label">API ENDPOINT (OPTIONAL)</label>
                <div class="input-wrapper">
                  <span class="input-prompt">>></span>
                  <input
                    v-model="configEndpoint"
                    type="text"
                    class="console-input"
                    :placeholder="currentProvider?.defaultEndpoint"
                  />
                </div>
              </div>
            </v-card-text>
            <v-card-actions class="console-card-actions">
              <button
                v-if="isProviderConfigured(currentProvider?.id)"
                class="console-btn danger"
                @click="deleteKey"
              >
                <span class="btn-text">DELETE</span>
              </button>
              <v-spacer />
              <button class="console-btn" @click="showConfigDialog = false">
                <span class="btn-text">CANCEL</span>
              </button>
              <button class="console-btn primary" @click="saveKey">
                <span class="btn-text">SAVE</span>
              </button>
            </v-card-actions>
          </v-card>
        </v-dialog>

        <div class="divider-line my-4"></div>

        <!-- Parameters -->
        <div class="section-header">
          <span class="section-title">PARAMETERS</span>
        </div>

        <div class="param-group">
          <div class="param-row">
            <span class="param-label">TEMPERATURE</span>
            <span class="param-value">{{ localSettings.temperature.toFixed(1) }}</span>
          </div>
          <input
            type="range"
            v-model.number="localSettings.temperature"
            min="0" max="2" step="0.1"
            class="console-slider"
          />
        </div>

        <div class="param-group">
          <div class="param-row">
            <span class="param-label">MAX TOKENS</span>
          </div>
          <div class="input-wrapper">
            <span class="input-prompt">>></span>
            <input
              v-model.number="localSettings.maxTokens"
              type="number"
              min="256"
              max="128000"
              class="console-input"
            />
          </div>
        </div>

        <div class="param-group">
          <div class="param-row">
            <span class="param-label">STREAM RESPONSE</span>
            <label class="toggle-switch" :class="{ active: localSettings.streamEnabled }">
              <input type="checkbox" v-model="localSettings.streamEnabled" />
              <span class="toggle-handle"></span>
            </label>
          </div>
        </div>
      </v-card-text>

      <v-card-actions class="console-card-actions">
        <v-spacer />
        <button class="console-btn" @click="saveAndClose">
          <span class="btn-text">SAVE</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useAiChatStore } from '@/stores/aiChat'
import ProviderIcon from '@/components/common/ProviderIcon.vue'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const store = useAiChatStore()

const providerList = [
  { id: 'openai', name: 'OpenAI', defaultEndpoint: 'https://api.openai.com/v1' },
  { id: 'anthropic', name: 'Anthropic', defaultEndpoint: 'https://api.anthropic.com' },
  { id: 'google', name: 'Google', defaultEndpoint: 'https://generativelanguage.googleapis.com' },
  { id: 'deepseek', name: 'DeepSeek', defaultEndpoint: 'https://api.deepseek.com' },
  { id: 'openrouter', name: 'OpenRouter', defaultEndpoint: 'https://openrouter.ai/api/v1' },
  { id: 'ollama', name: 'Ollama', defaultEndpoint: 'http://localhost:11434/v1' },
]

const showConfigDialog = ref(false)
const currentProvider = ref<typeof providerList[0] | null>(null)
const configApiKey = ref('')
const configEndpoint = ref('')
const showKey = ref(false)

const localSettings = ref({
  temperature: 0.7,
  maxTokens: 4096,
  streamEnabled: true,
})

watch(() => props.modelValue, (open) => {
  if (open) {
    localSettings.value = {
      temperature: store.temperature,
      maxTokens: store.maxTokens,
      streamEnabled: store.streamEnabled,
    }
    store.loadProviders()
  }
})

function isProviderConfigured(id: string | undefined) {
  return store.providers.some(p => p.provider === id && p.has_key)
}

function openConfig(id: string) {
  currentProvider.value = providerList.find(p => p.id === id) || null
  configApiKey.value = ''
  configEndpoint.value = ''
  showConfigDialog.value = true
}

async function saveKey() {
  if (!currentProvider.value) return
  await store.saveApiKey(
    currentProvider.value.id,
    configApiKey.value,
    configEndpoint.value || undefined
  )
  showConfigDialog.value = false
}

async function deleteKey() {
  if (!currentProvider.value) return
  await store.deleteApiKey(currentProvider.value.id)
  showConfigDialog.value = false
}

function saveAndClose() {
  store.temperature = localSettings.value.temperature
  store.maxTokens = localSettings.value.maxTokens
  store.streamEnabled = localSettings.value.streamEnabled
  store.saveSettings()
  emit('update:modelValue', false)
}
</script>

<style scoped>
.settings-content {
  padding: 20px !important;
}

.section-header {
  margin-bottom: 12px;
}

.section-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #00ff88;
}

.providers-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.provider-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 6px;
}

.provider-item.configured {
  border-color: rgba(0, 255, 136, 0.3);
  background: rgba(0, 255, 136, 0.03);
}

.provider-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 255, 255, 0.05);
  border-radius: 6px;
  font-size: 16px;
}

.provider-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.provider-name {
  font-size: 13px;
  font-weight: 600;
  color: #e4e4e7;
}

.provider-status {
  font-size: 10px;
  color: #52525b;
  letter-spacing: 1px;
}

.provider-item.configured .provider-status {
  color: #00ff88;
}

.divider-line {
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(0, 255, 255, 0.2), transparent);
  margin: 16px 0;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #00ffff;
}

.param-group {
  margin-bottom: 16px;
}

.param-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.param-label {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #a1a1aa;
}

.param-value {
  font-size: 12px;
  font-weight: 600;
  color: #00ff88;
}

.console-slider {
  width: 100%;
  height: 4px;
  -webkit-appearance: none;
  background: rgba(0, 255, 255, 0.2);
  border-radius: 2px;
}

.console-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #00ffff;
  cursor: pointer;
}

.toggle-switch {
  display: inline-flex;
  align-items: center;
  width: 40px;
  height: 22px;
  background: rgba(82, 82, 91, 0.3);
  border: 1px solid rgba(82, 82, 91, 0.5);
  border-radius: 11px;
  cursor: pointer;
}

.toggle-switch input { display: none; }

.toggle-switch.active {
  background: rgba(0, 255, 136, 0.15);
  border-color: rgba(0, 255, 136, 0.5);
}

.toggle-handle {
  width: 16px;
  height: 16px;
  background: #52525b;
  border-radius: 50%;
  margin: 2px;
  transition: all 0.2s;
}

.toggle-switch.active .toggle-handle {
  background: #00ff88;
  margin-left: 20px;
}
</style>
