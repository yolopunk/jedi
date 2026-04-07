<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="520"
  >
    <v-card class="model-settings-card">
      <!-- Header -->
      <div class="card-header">
        <div class="header-brand">
          <div class="brand-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
              <circle cx="12" cy="12" r="4" fill="currentColor"/>
              <line x1="12" y1="2" x2="12" y2="6" stroke="currentColor" stroke-width="1.5"/>
              <line x1="12" y1="18" x2="12" y2="22" stroke="currentColor" stroke-width="1.5"/>
              <line x1="2" y1="12" x2="6" y2="12" stroke="currentColor" stroke-width="1.5"/>
              <line x1="18" y1="12" x2="22" y2="12" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>Configure Provider</h2>
            <p v-if="!selectedProvider">Select a provider to configure</p>
          </div>
        </div>
        <button class="close-btn" @click="$emit('update:modelValue', false)">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <!-- Loading State -->
      <div v-if="loading" class="loading-state">
        <div class="spinner"></div>
        <span>Loading providers...</span>
      </div>

      <!-- Error State -->
      <div v-else-if="error" class="error-state">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
          <line x1="12" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="2"/>
          <circle cx="12" cy="16" r="1" fill="currentColor"/>
        </svg>
        <span>{{ error }}</span>
        <button class="retry-btn" @click="retryLoad">Retry</button>
      </div>

      <!-- Main Content -->
      <div v-else class="card-body">
        <!-- Provider List (shown when no provider selected) -->
        <div v-if="!selectedProvider" class="provider-list">
          <div class="list-header">
            <span class="list-label">PROVIDERS</span>
            <div class="category-pills">
              <button
                v-for="cat in categories"
                :key="cat.id"
                class="pill"
                :class="{ active: activeCategory === cat.id }"
                @click="activeCategory = cat.id"
              >
                {{ cat.label }}
                <span class="pill-count">{{ getCategoryCount(cat.id) }}</span>
              </button>
            </div>
          </div>

          <div class="providers-scroll">
            <div
              v-for="provider in filteredProviders"
              :key="provider.id"
              class="provider-item"
              :class="{ configured: isConfigured(provider.id) }"
              @click="selectProvider(provider)"
            >
              <div class="provider-logo">
                <img
                  :src="`https://models.dev/logos/${provider.id}.svg`"
                  :alt="provider.name"
                  @error="(e) => (e.target as HTMLImageElement).style.display='none'"
                />
              </div>
              <div class="provider-info">
                <span class="provider-name">{{ provider.name }}</span>
                <span class="provider-models">{{ Object.keys(provider.models).length }} models</span>
              </div>
              <div class="provider-status">
                <span v-if="isConfigured(provider.id)" class="status-badge configured">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="10"/></svg>
                  Ready
                </span>
                <span v-else class="status-badge">
                  Not set
                </span>
              </div>
              <svg class="chevron" width="16" height="16" viewBox="0 0 24 24" fill="none">
                <polyline points="9 18 15 12 9 6" stroke="currentColor" stroke-width="2"/>
              </svg>
            </div>

            <div v-if="filteredProviders.length === 0" class="empty-list">
              <span>No providers in this category</span>
            </div>
          </div>
        </div>

        <!-- Config View (shown when provider selected) -->
        <div v-else class="config-view">
          <!-- Back button + Provider info -->
          <div class="config-header">
            <button class="back-btn" @click="selectedProvider = null">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <polyline points="15 18 9 12 15 6" stroke="currentColor" stroke-width="2"/>
              </svg>
              Back
            </button>
            <div class="selected-provider">
              <div class="provider-logo large">
                <img
                  :src="`https://models.dev/logos/${selectedProvider.id}.svg`"
                  :alt="selectedProvider.name"
                  @error="(e) => (e.target as HTMLImageElement).style.display='none'"
                />
              </div>
              <div class="provider-info">
                <span class="provider-name">{{ selectedProvider.name }}</span>
                <a
                  v-if="selectedProvider.doc"
                  :href="selectedProvider.doc"
                  target="_blank"
                  class="doc-link"
                >
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                    <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" stroke="currentColor" stroke-width="2"/>
                    <polyline points="15 3 21 3 21 9" stroke="currentColor" stroke-width="2"/>
                    <line x1="10" y1="14" x2="21" y2="3" stroke="currentColor" stroke-width="2"/>
                  </svg>
                  Documentation
                </a>
              </div>
            </div>
          </div>

          <!-- API Key Form -->
          <div class="config-form">
            <div class="form-group">
              <label>API KEY</label>
              <div class="input-row">
                <input
                  v-model="configApiKey"
                  :type="showKey ? 'text' : 'password'"
                  placeholder="sk-..."
                  class="api-input"
                />
                <button class="toggle-btn" @click="showKey = !showKey">
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
              <input
                v-model="configEndpoint"
                type="text"
                :placeholder="selectedProvider.api || 'https://api.example.com'"
                class="api-input"
              />
            </div>

            <div class="form-actions">
              <button
                v-if="isCurrentProviderConfigured"
                class="btn danger"
                @click="deleteKey"
              >
                Delete Key
              </button>
              <button
                v-if="isCurrentProviderConfigured && configApiKey"
                class="btn secondary"
                @click="testConnection"
                :disabled="testing"
              >
                <span v-if="testing" class="btn-spinner"></span>
                {{ testing ? 'Testing...' : 'Test Connection' }}
              </button>
              <div class="spacer"></div>
              <button class="btn secondary" @click="selectedProvider = null">
                Cancel
              </button>
              <button
                class="btn primary"
                @click="saveKey"
                :disabled="!configApiKey"
              >
                Save
              </button>
            </div>
          </div>

          <!-- Model List -->
          <div class="model-section">
            <div class="section-header">
              <span class="section-label">AVAILABLE MODELS</span>
              <span class="model-count">{{ selectedProviderModels.length }} models</span>
            </div>
            <div class="models-scroll">
              <div
                v-for="model in selectedProviderModels"
                :key="model.id"
                class="model-item"
                @click="selectModel(model)"
              >
                <div class="model-info">
                  <span class="model-name">{{ model.name }}</span>
                  <span class="model-id">{{ model.id }}</span>
                </div>
                <div class="model-badges">
                  <span v-if="model.reasoning" class="badge reasoning">R</span>
                  <span v-if="model.tool_call" class="badge tools">T</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
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
const selectedProvider = ref<ModelsDevProvider | null>(null)
const configApiKey = ref('')
const configEndpoint = ref('')
const showKey = ref(false)
const testing = ref(false)

const categories = [
  { id: 'popular' as const, label: 'Popular' },
  { id: 'other' as const, label: 'Other' },
  { id: 'custom' as const, label: 'Custom' },
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

const selectedProviderModels = computed(() => {
  if (!selectedProvider.value) return []
  return Object.values(selectedProvider.value.models)
})

const isCurrentProviderConfigured = computed(() =>
  selectedProvider.value
    ? providerConfigStore.isProviderConfigured(selectedProvider.value.id)
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
  selectedProvider.value = provider
  configApiKey.value = ''
  configEndpoint.value = provider.api || ''
  showKey.value = false
}

function selectModel(model: ModelsDevModel) {
  modelsDevStore.selectModel(model.id)
}

watch(() => modelsDevStore.selectedModelId, (modelId) => {
  if (modelId && selectedProvider.value) {
    modelsDevStore.selectProvider(selectedProvider.value.id)
  }
})

watch(() => props.modelValue, async (open) => {
  if (open) {
    selectedProvider.value = null
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

async function saveKey() {
  if (!selectedProvider.value || !configApiKey.value) return
  await providerConfigStore.saveApiKey(
    selectedProvider.value.id,
    configApiKey.value,
    configEndpoint.value || undefined
  )
  selectedProvider.value = null
}

async function deleteKey() {
  if (!selectedProvider.value) return
  await providerConfigStore.deleteApiKey(selectedProvider.value.id)
  selectedProvider.value = null
}

async function testConnection() {
  testing.value = true
  await new Promise(r => setTimeout(r, 1500))
  testing.value = false
}
</script>

<style scoped>
.model-settings-card {
  background: #0a0e14 !important;
  border-radius: 16px !important;
  overflow: hidden;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
}

/* Header */
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(20, 30, 40, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.brand-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.15) 0%, rgba(0, 255, 136, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.25);
  border-radius: 10px;
  color: #00ffff;
}

.brand-text h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.brand-text p {
  margin: 2px 0 0;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.close-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: rgba(255, 107, 107, 0.1);
  border-color: rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

/* Loading & Error */
.loading-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  gap: 12px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 13px;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 2px solid rgba(0, 255, 255, 0.2);
  border-top-color: #00ffff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-state {
  color: #ff6b6b;
}

.retry-btn {
  padding: 8px 16px;
  background: rgba(0, 255, 255, 0.1);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 6px;
  color: #00ffff;
  font-size: 12px;
  cursor: pointer;
}

/* Body */
.card-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* Provider List */
.provider-list {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

.list-header {
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.list-label {
  display: block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1.5px;
  color: rgba(255, 255, 255, 0.3);
  margin-bottom: 10px;
}

.category-pills {
  display: flex;
  gap: 6px;
}

.pill {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 20px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.pill:hover {
  background: rgba(255, 255, 255, 0.03);
  border-color: rgba(255, 255, 255, 0.15);
}

.pill.active {
  background: rgba(0, 255, 255, 0.1);
  border-color: rgba(0, 255, 255, 0.35);
  color: #00ffff;
}

.pill-count {
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  font-size: 10px;
}

.pill.active .pill-count {
  background: rgba(0, 255, 255, 0.2);
}

.providers-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.provider-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.provider-item:hover {
  background: rgba(255, 255, 255, 0.03);
}

.provider-item.configured {
  background: rgba(0, 255, 136, 0.03);
}

.provider-logo {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  flex-shrink: 0;
  overflow: hidden;
}

.provider-logo.large {
  width: 44px;
  height: 44px;
  border-radius: 10px;
}

.provider-logo img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.provider-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.provider-name {
  font-size: 14px;
  font-weight: 600;
  color: #ffffff;
}

.provider-models {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
}

.doc-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: #00ffff;
  text-decoration: none;
  opacity: 0.7;
  transition: opacity 0.15s;
}

.doc-link:hover {
  opacity: 1;
}

.provider-status {
  flex-shrink: 0;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  font-size: 10px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.05);
  color: rgba(255, 255, 255, 0.4);
}

.status-badge.configured {
  background: rgba(0, 255, 136, 0.12);
  color: #00ff88;
}

.status-badge svg {
  opacity: 0.7;
}

.chevron {
  color: rgba(255, 255, 255, 0.2);
  flex-shrink: 0;
}

.empty-list {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: rgba(255, 255, 255, 0.25);
  font-size: 13px;
}

/* Config View */
.config-view {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

.config-header {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.back-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 0;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
  cursor: pointer;
  transition: color 0.15s;
}

.back-btn:hover {
  color: #00ffff;
}

.selected-provider {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* Config Form */
.config-form {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
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

.input-row {
  display: flex;
  gap: 8px;
}

.api-input {
  flex: 1;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #ffffff;
  font-size: 13px;
  font-family: 'JetBrains Mono', monospace;
  transition: border-color 0.15s;
}

.api-input:focus {
  outline: none;
  border-color: rgba(0, 255, 255, 0.4);
}

.api-input::placeholder {
  color: rgba(255, 255, 255, 0.2);
}

.toggle-btn {
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  transition: all 0.15s;
}

.toggle-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.7);
}

.form-actions {
  display: flex;
  gap: 8px;
  padding-top: 4px;
}

.spacer {
  flex: 1;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 9px 16px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 7px;
  cursor: pointer;
  transition: all 0.15s ease;
  border: 1px solid transparent;
}

.btn.primary {
  background: rgba(0, 255, 255, 0.15);
  border-color: rgba(0, 255, 255, 0.4);
  color: #00ffff;
}

.btn.primary:hover:not(:disabled) {
  background: rgba(0, 255, 255, 0.25);
}

.btn.primary:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.btn.secondary {
  background: transparent;
  border-color: rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.6);
}

.btn.secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.2);
  color: #ffffff;
}

.btn.secondary:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.btn.danger {
  background: transparent;
  border-color: rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

.btn.danger:hover {
  background: rgba(255, 107, 107, 0.1);
}

.btn-spinner {
  width: 12px;
  height: 12px;
  border: 1.5px solid rgba(255, 255, 255, 0.2);
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

/* Model Section */
.model-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px 8px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1.5px;
  color: rgba(255, 255, 255, 0.3);
}

.model-count {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
}

.models-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 8px;
}

.model-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.1s;
}

.model-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.model-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.model-name {
  font-size: 13px;
  font-weight: 500;
  color: #ffffff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.model-id {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.3);
  font-family: 'JetBrains Mono', monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.model-badges {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.badge {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 9px;
  font-weight: 700;
  border-radius: 4px;
}

.badge.reasoning {
  background: rgba(168, 85, 247, 0.2);
  color: #a855f7;
}

.badge.tools {
  background: rgba(59, 130, 246, 0.2);
  color: #3b82f6;
}

/* Scrollbar */
::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.15);
}
</style>
