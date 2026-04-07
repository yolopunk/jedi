<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="900"
    fullscreen
  >
    <v-card class="model-settings-panel">
      <!-- Header -->
      <div class="panel-header">
        <div class="header-left">
          <div class="header-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
              <circle cx="12" cy="12" r="4" fill="currentColor"/>
              <line x1="12" y1="2" x2="12" y2="6" stroke="currentColor" stroke-width="1.5"/>
              <line x1="12" y1="18" x2="12" y2="22" stroke="currentColor" stroke-width="1.5"/>
              <line x1="2" y1="12" x2="6" y2="12" stroke="currentColor" stroke-width="1.5"/>
              <line x1="18" y1="12" x2="22" y2="12" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          </div>
          <div class="header-title">
            <h2>Model Selection</h2>
            <p>Choose your AI provider and model</p>
          </div>
        </div>
        <button class="close-btn" @click="$emit('update:modelValue', false)">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <!-- Loading State -->
      <div v-if="loading" class="loading-state">
        <div class="loading-spinner">
          <div class="spinner-ring"></div>
          <div class="spinner-ring"></div>
          <div class="spinner-ring"></div>
        </div>
        <p>Connecting to models.dev...</p>
      </div>

      <!-- Error State -->
      <div v-else-if="error" class="error-state">
        <div class="error-icon">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
            <line x1="12" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="2"/>
            <circle cx="12" cy="16" r="1" fill="currentColor"/>
          </svg>
        </div>
        <h3>Connection Failed</h3>
        <p>{{ error }}</p>
        <button class="retry-btn" @click="retryLoad">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <path d="M1 4v6h6M23 20v-6h-6" stroke="currentColor" stroke-width="2"/>
            <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15" stroke="currentColor" stroke-width="2"/>
          </svg>
          Retry
        </button>
      </div>

      <!-- Main Content -->
      <div v-else class="panel-content">
        <!-- Left: Provider List -->
        <div class="provider-panel">
          <div class="panel-section-header">
            <span class="section-label">PROVIDERS</span>
            <div class="category-tabs">
              <button
                v-for="cat in categories"
                :key="cat.id"
                class="category-tab"
                :class="{ active: activeCategory === cat.id }"
                @click="activeCategory = cat.id"
              >
                <v-icon :icon="cat.icon" size="14" />
                {{ cat.label }}
                <span class="count">{{ getCategoryCount(cat.id) }}</span>
              </button>
            </div>
          </div>

          <div class="provider-list">
            <div
              v-for="provider in filteredProviders"
              :key="provider.id"
              class="provider-card"
              :class="{
                selected: provider.id === modelsDevStore.selectedProviderId,
                configured: isConfigured(provider.id)
              }"
              @click="selectProvider(provider)"
            >
              <div class="provider-logo" :style="{ background: getProviderGradient(provider.id) }">
                <v-icon :icon="getProviderIcon(provider.id)" size="20" color="white" />
              </div>
              <div class="provider-details">
                <span class="provider-name">{{ provider.name }}</span>
                <span class="provider-meta">
                  {{ Object.keys(provider.models).length }} models
                  <span v-if="isConfigured(provider.id)" class="status-dot configured"></span>
                </span>
              </div>
              <div class="provider-check" v-if="provider.id === modelsDevStore.selectedProviderId">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <path d="M20 6L9 17l-5-5" stroke="currentColor" stroke-width="2.5"/>
                </svg>
              </div>
            </div>

            <div v-if="filteredProviders.length === 0" class="empty-list">
              <p>No providers in this category</p>
            </div>
          </div>
        </div>

        <!-- Right: Model Grid -->
        <div class="model-panel" v-if="modelsDevStore.selectedProvider">
          <div class="panel-section-header">
            <div class="selected-provider-info">
              <span class="section-label">MODELS</span>
              <span class="selected-provider-name">{{ modelsDevStore.selectedProvider.name }}</span>
            </div>
            <div class="model-search">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                <circle cx="11" cy="11" r="8" stroke="currentColor" stroke-width="2"/>
                <line x1="21" y1="21" x2="16.65" y2="16.65" stroke="currentColor" stroke-width="2"/>
              </svg>
              <input
                v-model="modelSearch"
                type="text"
                placeholder="Filter models..."
              />
            </div>
          </div>

          <div class="model-grid">
            <div
              v-for="model in filteredModels"
              :key="model.id"
              class="model-card"
              :class="{ selected: model.id === modelsDevStore.selectedModelId }"
              @click="selectModel(model)"
            >
              <div class="model-header">
                <div class="model-title">
                  <span class="model-name">{{ model.name }}</span>
                  <span class="model-id">{{ model.id }}</span>
                </div>
                <div class="model-check" v-if="model.id === modelsDevStore.selectedModelId">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                    <path d="M20 6L9 17l-5-5" stroke="currentColor" stroke-width="2.5"/>
                  </svg>
                </div>
              </div>

              <div class="model-badges">
                <span v-if="model.reasoning" class="badge reasoning">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="10"/></svg>
                  Reasoning
                </span>
                <span v-if="model.tool_call" class="badge tools">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor"><rect x="3" y="3" width="18" height="18" rx="2"/></svg>
                  Tools
                </span>
                <span v-if="model.structured_output" class="badge structured">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor"><polygon points="12,2 22,20 2,20"/></svg>
                  Structured
                </span>
              </div>

              <div class="model-stats">
                <div class="stat">
                  <span class="stat-label">Context</span>
                  <span class="stat-value">{{ formatContext(model.limit?.context) }}</span>
                </div>
                <div class="stat" v-if="model.cost?.input">
                  <span class="stat-label">Input</span>
                  <span class="stat-value">${{ model.cost.input.toFixed(2) }}</span>
                </div>
                <div class="stat" v-if="model.cost?.output">
                  <span class="stat-label">Output</span>
                  <span class="stat-value">${{ model.cost.output.toFixed(2) }}</span>
                </div>
              </div>

              <div class="model-modalities">
                <span
                  v-for="mod in getModalities(model)"
                  :key="mod.type"
                  class="modality-tag"
                  :class="mod.type.toLowerCase()"
                >
                  {{ mod.label }}
                </span>
              </div>
            </div>

            <div v-if="filteredModels.length === 0" class="empty-list">
              <p>No models match your search</p>
            </div>
          </div>

          <!-- Configure Button -->
          <div class="config-bar">
            <div class="current-selection" v-if="modelsDevStore.selectedModel">
              <span class="selection-label">Selected:</span>
              <span class="selection-value">{{ modelsDevStore.selectedModel.name }}</span>
            </div>
            <button
              class="config-btn"
              :class="{ configured: isCurrentProviderConfigured }"
              @click="openConfig"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" stroke="currentColor" stroke-width="1.5"/>
              </svg>
              {{ isCurrentProviderConfigured ? 'Update API Key' : 'Configure Provider' }}
            </button>
          </div>
        </div>

        <!-- Empty State: No Provider Selected -->
        <div v-else class="no-selection">
          <div class="no-selection-icon">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none">
              <rect x="3" y="3" width="18" height="18" rx="2" stroke="currentColor" stroke-width="1.5" stroke-dasharray="4 2"/>
              <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          </div>
          <h3>Select a Provider</h3>
          <p>Choose an AI provider from the list to see available models</p>
        </div>
      </div>

      <!-- API Key Config Dialog -->
      <v-dialog v-model="showConfigDialog" max-width="420">
        <v-card class="config-dialog">
          <div class="dialog-header">
            <h3>{{ currentProviderName }}_CONFIG</h3>
            <button class="dialog-close" @click="showConfigDialog = false">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
                <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
              </svg>
            </button>
          </div>

          <div class="dialog-content">
            <div class="form-group">
              <label>API KEY</label>
              <div class="input-wrapper">
                <input
                  v-model="configApiKey"
                  :type="showKey ? 'text' : 'password'"
                  placeholder="sk-..."
                />
                <button class="toggle-visibility" @click="showKey = !showKey">
                  <svg v-if="showKey" width="16" height="16" viewBox="0 0 24 24" fill="none">
                    <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" stroke="currentColor" stroke-width="2"/>
                    <line x1="1" y1="1" x2="23" y2="23" stroke="currentColor" stroke-width="2"/>
                  </svg>
                  <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none">
                    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" stroke="currentColor" stroke-width="2"/>
                    <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
                  </svg>
                </button>
              </div>
            </div>

            <div class="form-group">
              <label>API ENDPOINT <span class="optional">(optional)</span></label>
              <div class="input-wrapper">
                <input
                  v-model="configEndpoint"
                  type="text"
                  :placeholder="currentProviderEndpoint || 'https://api.example.com'"
                />
              </div>
            </div>
          </div>

          <div class="dialog-actions">
            <button
              v-if="isCurrentProviderConfigured"
              class="btn danger"
              @click="deleteKey"
            >
              Delete Key
            </button>
            <div class="spacer"></div>
            <button class="btn secondary" @click="showConfigDialog = false">
              Cancel
            </button>
            <button class="btn primary" @click="saveKey" :disabled="!configApiKey">
              Save
            </button>
          </div>
        </v-card>
      </v-dialog>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useModelsDevStore } from '@/stores/modelsDev'
import { useProviderConfigStore } from '@/stores/providerConfig'
import type { ModelsDevProvider, ModelsDevModel } from '@/types/modelsDev'

const props = defineProps<{ modelValue: boolean }>()
defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const modelsDevStore = useModelsDevStore()
const providerConfigStore = useProviderConfigStore()

const activeCategory = ref<'popular' | 'other' | 'custom'>('popular')
const modelSearch = ref('')
const showConfigDialog = ref(false)
const showKey = ref(false)
const configApiKey = ref('')
const configEndpoint = ref('')

const categories = [
  { id: 'popular' as const, label: 'Popular', icon: 'mdi-star' },
  { id: 'other' as const, label: 'Other', icon: 'mdi-dots-horizontal' },
  { id: 'custom' as const, label: 'Custom', icon: 'mdi-plus-circle' },
]

const loading = computed(() => modelsDevStore.loading || providerConfigStore.loading)
const error = computed(() => modelsDevStore.error || providerConfigStore.error)

const filteredProviders = computed(() => {
  switch (activeCategory.value) {
    case 'popular': return modelsDevStore.popularProviders
    case 'other': return modelsDevStore.otherProviders
    case 'custom': return modelsDevStore.customProviders
    default: return []
  }
})

const filteredModels = computed(() => {
  const models = modelsDevStore.selectedProviderModels
  if (!modelSearch.value) return models
  const q = modelSearch.value.toLowerCase()
  return models.filter(m =>
    m.name.toLowerCase().includes(q) ||
    m.id.toLowerCase().includes(q)
  )
})

const currentProviderName = computed(() => modelsDevStore.selectedProvider?.name || '')
const currentProviderEndpoint = computed(() => modelsDevStore.selectedProvider?.api || '')
const isCurrentProviderConfigured = computed(() =>
  modelsDevStore.selectedProviderId
    ? providerConfigStore.isProviderConfigured(modelsDevStore.selectedProviderId)
    : false
)

function getCategoryCount(category: 'popular' | 'other' | 'custom') {
  switch (category) {
    case 'popular': return modelsDevStore.popularProviders.length
    case 'other': return modelsDevStore.otherProviders.length
    case 'custom': return modelsDevStore.customProviders.length
  }
}

function isConfigured(providerId: string) {
  return providerConfigStore.isProviderConfigured(providerId)
}

function selectProvider(provider: ModelsDevProvider) {
  modelsDevStore.selectProvider(provider.id)
  modelSearch.value = ''
}

function selectModel(model: ModelsDevModel) {
  modelsDevStore.selectModel(model.id)
}

function getProviderIcon(id: string): string {
  const icons: Record<string, string> = {
    openai: 'mdi-openai',
    anthropic: 'mdi-brain',
    google: 'mdi-google',
    deepseek: 'mdi-deepseek',
    xai: 'mdi-robot',
    mistral: 'mdi-weather-cloudy',
    moonshot: 'mdi-moon-waning-crescent',
    zhipu: 'mdi-alpha-z-circle',
  }
  return icons[id] || 'mdi-cloud'
}

function getProviderGradient(id: string): string {
  const gradients: Record<string, string> = {
    openai: 'linear-gradient(135deg, #10a37f 0%, #0d7a5f 100%)',
    anthropic: 'linear-gradient(135deg, #ff6b6b 0%, #c0392b 100%)',
    google: 'linear-gradient(135deg, #4285f4 0%, #0d47a1 100%)',
    deepseek: 'linear-gradient(135deg, #6366f1 0%, #4338ca 100%)',
    xai: 'linear-gradient(135deg, #f97316 0%, #c2410c 100%)',
    mistral: 'linear-gradient(135deg, #64748b 0%, #334155 100%)',
    moonshot: 'linear-gradient(135deg, #8b5cf6 0%, #6d28d9 100%)',
    zhipu: 'linear-gradient(135deg, #06b6d4 0%, #0891b2 100%)',
  }
  return gradients[id] || 'linear-gradient(135deg, #6b7280 0%, #374151 100%)'
}

function formatContext(context?: number): string {
  if (!context) return '-'
  if (context >= 1000000) return `${(context / 1000000).toFixed(0)}M`
  if (context >= 1000) return `${Math.round(context / 1000)}K`
  return context.toString()
}

function getModalities(model: ModelsDevModel) {
  const mods = []
  if (model.modalities.input.includes('text')) mods.push({ type: 'text', label: 'Text' })
  if (model.modalities.input.includes('image')) mods.push({ type: 'image', label: 'Image' })
  if (model.modalities.input.includes('audio')) mods.push({ type: 'audio', label: 'Audio' })
  if (model.modalities.output.includes('audio')) mods.push({ type: 'audio-out', label: 'Audio Out' })
  return mods
}

watch(() => props.modelValue, async (open) => {
  if (open) {
    await Promise.all([
      providerConfigStore.loadConfiguredProviders(),
      modelsDevStore.fetchProviders()
    ])
  }
})

async function retryLoad() {
  await Promise.all([
    providerConfigStore.loadConfiguredProviders(),
    modelsDevStore.fetchProviders(true)
  ])
}

function openConfig() {
  configApiKey.value = ''
  configEndpoint.value = currentProviderEndpoint.value
  showKey.value = false
  showConfigDialog.value = true
}

async function saveKey() {
  const providerId = modelsDevStore.selectedProviderId
  if (!providerId || !configApiKey.value) return
  await providerConfigStore.saveApiKey(providerId, configApiKey.value, configEndpoint.value || undefined)
  showConfigDialog.value = false
}

async function deleteKey() {
  const providerId = modelsDevStore.selectedProviderId
  if (!providerId) return
  await providerConfigStore.deleteApiKey(providerId)
  showConfigDialog.value = false
}
</script>

<style scoped>
.model-settings-panel {
  background: #0a0e14 !important;
  border-radius: 16px !important;
  overflow: hidden;
  height: 85vh;
  display: flex;
  flex-direction: column;
}

/* Header */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  background: linear-gradient(180deg, rgba(20, 30, 40, 0.8) 0%, transparent 100%);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.15) 0%, rgba(0, 255, 136, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 12px;
  color: #00ffff;
}

.header-title h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #ffffff;
  letter-spacing: -0.5px;
}

.header-title p {
  margin: 4px 0 0;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
}

.close-btn {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: rgba(255, 107, 107, 0.1);
  border-color: rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

/* Loading & Error States */
.loading-state,
.error-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 20px;
}

.loading-spinner {
  position: relative;
  width: 60px;
  height: 60px;
}

.spinner-ring {
  position: absolute;
  width: 100%;
  height: 100%;
  border: 2px solid transparent;
  border-radius: 50%;
  animation: spin 1.5s linear infinite;
}

.spinner-ring:nth-child(1) {
  border-top-color: #00ffff;
  animation-delay: 0s;
}

.spinner-ring:nth-child(2) {
  width: 80%;
  height: 80%;
  top: 10%;
  left: 10%;
  border-top-color: #00ff88;
  animation-delay: 0.1s;
  animation-direction: reverse;
}

.spinner-ring:nth-child(3) {
  width: 60%;
  height: 60%;
  top: 20%;
  left: 20%;
  border-top-color: #ffb800;
  animation-delay: 0.2s;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-state p,
.error-state p {
  color: rgba(255, 255, 255, 0.6);
  font-size: 14px;
}

.error-icon {
  color: #ff6b6b;
}

.error-state h3 {
  margin: 0;
  font-size: 18px;
  color: #ffffff;
}

.retry-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: rgba(0, 255, 255, 0.1);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 8px;
  color: #00ffff;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.retry-btn:hover {
  background: rgba(0, 255, 255, 0.15);
}

/* Main Content */
.panel-content {
  flex: 1;
  display: grid;
  grid-template-columns: 320px 1fr;
  overflow: hidden;
}

/* Provider Panel */
.provider-panel {
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-section-header {
  padding: 16px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.section-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1.5px;
  color: rgba(255, 255, 255, 0.3);
  margin-bottom: 12px;
}

.category-tabs {
  display: flex;
  gap: 6px;
}

.category-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.category-tab:hover {
  background: rgba(255, 255, 255, 0.03);
  border-color: rgba(255, 255, 255, 0.15);
}

.category-tab.active {
  background: rgba(0, 255, 255, 0.1);
  border-color: rgba(0, 255, 255, 0.3);
  color: #00ffff;
}

.category-tab .count {
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 4px;
  font-size: 10px;
}

.category-tab.active .count {
  background: rgba(0, 255, 255, 0.2);
}

.provider-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.provider-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.provider-card:hover {
  background: rgba(255, 255, 255, 0.04);
  border-color: rgba(255, 255, 255, 0.1);
}

.provider-card.selected {
  background: rgba(0, 255, 255, 0.08);
  border-color: rgba(0, 255, 255, 0.25);
}

.provider-card.configured {
  border-left: 3px solid #00ff88;
}

.provider-logo {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  flex-shrink: 0;
}

.provider-details {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.provider-name {
  font-size: 14px;
  font-weight: 600;
  color: #ffffff;
}

.provider-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
}

.status-dot.configured {
  background: #00ff88;
  box-shadow: 0 0 8px rgba(0, 255, 136, 0.5);
}

.provider-check {
  color: #00ffff;
}

.empty-list {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
  color: rgba(255, 255, 255, 0.3);
  font-size: 13px;
}

/* Model Panel */
.model-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.selected-provider-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.selected-provider-name {
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.model-search {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  margin-top: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.model-search input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #ffffff;
  font-size: 13px;
}

.model-search input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}

.model-grid {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
  align-content: start;
}

.model-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.model-card:hover {
  background: rgba(255, 255, 255, 0.04);
  border-color: rgba(255, 255, 255, 0.12);
  transform: translateY(-2px);
}

.model-card.selected {
  background: rgba(0, 255, 136, 0.06);
  border-color: rgba(0, 255, 136, 0.3);
}

.model-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.model-title {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.model-name {
  font-size: 15px;
  font-weight: 600;
  color: #ffffff;
}

.model-id {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  font-family: 'JetBrains Mono', monospace;
}

.model-check {
  color: #00ff88;
}

.model-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  font-size: 10px;
  font-weight: 600;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.badge.reasoning {
  color: #a855f7;
  background: rgba(168, 85, 247, 0.15);
  border: 1px solid rgba(168, 85, 247, 0.3);
}

.badge.tools {
  color: #3b82f6;
  background: rgba(59, 130, 246, 0.15);
  border: 1px solid rgba(59, 130, 246, 0.3);
}

.badge.structured {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.15);
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.model-stats {
  display: flex;
  gap: 16px;
}

.stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-label {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.35);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-value {
  font-size: 13px;
  font-weight: 600;
  color: #ffffff;
}

.model-modalities {
  display: flex;
  gap: 6px;
}

.modality-tag {
  padding: 3px 8px;
  font-size: 10px;
  font-weight: 600;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.05);
  color: rgba(255, 255, 255, 0.6);
}

.modality-tag.text { background: rgba(255, 255, 255, 0.1); color: #ffffff; }
.modality-tag.image { background: rgba(255, 107, 157, 0.15); color: #ff6b9d; }
.modality-tag.audio,
.modality-tag.audio-out { background: rgba(255, 217, 61, 0.15); color: #ffd93d; }

/* Config Bar */
.config-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(20, 30, 40, 0.5);
}

.current-selection {
  display: flex;
  align-items: center;
  gap: 8px;
}

.selection-label {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.selection-value {
  font-size: 13px;
  font-weight: 600;
  color: #00ff88;
}

.config-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  background: rgba(0, 255, 255, 0.1);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 8px;
  color: #00ffff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.config-btn:hover {
  background: rgba(0, 255, 255, 0.15);
}

.config-btn.configured {
  background: rgba(0, 255, 136, 0.1);
  border-color: rgba(0, 255, 136, 0.3);
  color: #00ff88;
}

.config-btn.configured:hover {
  background: rgba(0, 255, 136, 0.15);
}

/* No Selection */
.no-selection {
  grid-column: 2;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: rgba(255, 255, 255, 0.3);
}

.no-selection-icon {
  opacity: 0.3;
}

.no-selection h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.6);
}

.no-selection p {
  margin: 0;
  font-size: 14px;
}

/* Config Dialog */
.config-dialog {
  background: #0a0e14 !important;
  border-radius: 12px !important;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.dialog-header h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
  color: #00ffff;
}

.dialog-close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  border-radius: 6px;
}

.dialog-close:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #ffffff;
}

.dialog-content {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  color: #00ffff;
}

.form-group .optional {
  font-weight: 400;
  color: rgba(255, 255, 255, 0.3);
}

.input-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
}

.input-wrapper:focus-within {
  border-color: rgba(0, 255, 255, 0.3);
}

.input-wrapper input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #ffffff;
  font-size: 13px;
  font-family: 'JetBrains Mono', monospace;
}

.input-wrapper input::placeholder {
  color: rgba(255, 255, 255, 0.25);
}

.toggle-visibility {
  padding: 4px;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
}

.toggle-visibility:hover {
  color: rgba(255, 255, 255, 0.7);
}

.dialog-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.spacer {
  flex: 1;
}

.btn {
  padding: 10px 18px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn.primary {
  background: rgba(0, 255, 255, 0.15);
  border: 1px solid rgba(0, 255, 255, 0.4);
  color: #00ffff;
}

.btn.primary:hover:not(:disabled) {
  background: rgba(0, 255, 255, 0.25);
}

.btn.primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn.secondary {
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.6);
}

.btn.secondary:hover {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.25);
  color: #ffffff;
}

.btn.danger {
  background: rgba(255, 107, 107, 0.1);
  border: 1px solid rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

.btn.danger:hover {
  background: rgba(255, 107, 107, 0.2);
}

/* Scrollbar */
::-webkit-scrollbar {
  width: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.15);
}
</style>
