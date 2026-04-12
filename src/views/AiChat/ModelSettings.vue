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
            <input
              v-model="providerSearch"
              type="text"
              class="provider-search"
              placeholder="Search providers..."
            />
          </div>

          <div class="providers-scroll">
            <div
              v-for="provider in filteredProviders"
              :key="provider.id"
              class="provider-item"
              :class="{ configured: isConfigured(provider.id) }"
            >
              <div class="provider-main" @click="handleProviderClick(provider)">
                <div class="provider-logo">
                  <img
                    v-if="provider.id !== '__custom__'"
                    :src="`https://models.dev/logos/${provider.id}.svg`"
                    :alt="provider.name"
                    @error="(e) => (e.target as HTMLImageElement).style.display='none'"
                  />
                  <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="none">
                    <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
                    <line x1="12" y1="8" x2="12" y2="16" stroke="currentColor" stroke-width="2"/>
                    <line x1="8" y1="12" x2="16" y2="12" stroke="currentColor" stroke-width="2"/>
                  </svg>
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
              </div>
              <!-- Select button for configured providers -->
              <button
                v-if="isConfigured(provider.id)"
                class="select-btn"
                :class="{ active: modelsDevStore.selectedProviderId === provider.id }"
                @click.stop="selectProviderDirectly(provider)"
              >
                {{ modelsDevStore.selectedProviderId === provider.id ? 'Using' : 'Select' }}
              </button>
              <!-- Chevron for unconfigured providers (click to configure) -->
              <svg v-else class="chevron" width="16" height="16" viewBox="0 0 24 24" fill="none">
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
          <!-- Back button + Provider info + Endpoint (inline) -->
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
                <div class="provider-meta">
                  <span v-if="selectedProvider.api && isUrl(selectedProvider.api)" class="endpoint-tag">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none">
                      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" stroke="currentColor" stroke-width="2"/>
                      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" stroke="currentColor" stroke-width="2"/>
                    </svg>
                    {{ selectedProvider.api }}
                  </span>
                  <a
                    v-if="selectedProvider.doc"
                    :href="selectedProvider.doc"
                    target="_blank"
                    class="doc-link"
                  >
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none">
                      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" stroke="currentColor" stroke-width="2"/>
                      <polyline points="15 3 21 3 21 9" stroke="currentColor" stroke-width="2"/>
                      <line x1="10" y1="14" x2="21" y2="3" stroke="currentColor" stroke-width="2"/>
                    </svg>
                    Doc
                  </a>
                </div>
              </div>
            </div>
          </div>

          <!-- API Key Config (both custom and non-custom) -->
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

            <!-- Custom provider: Base URL field -->
            <div v-if="selectedProvider.isCustom" class="form-group">
              <label>BASE URL</label>
              <input
                v-model="configBaseUrl"
                type="text"
                placeholder="https://api.example.com/v1"
                class="api-input"
              />
            </div>

            <!-- Custom provider: Model ID field -->
            <div v-if="selectedProvider.isCustom" class="form-group">
              <label>MODEL ID</label>
              <input
                v-model="configModelId"
                type="text"
                placeholder="e.g., gpt-4o, claude-3-opus"
                class="api-input"
              />
            </div>

            <div class="form-actions">
              <v-tooltip text="Verify credentials work">
                <template v-slot:activator="{ props }">
                  <v-btn
                    v-bind="props"
                    size="small"
                    variant="tonal"
                    :loading="testing"
                    :disabled="!configApiKey"
                    @click="testConnection"
                  >
                    <v-icon icon="mdi-lightning-bolt" start size="14" />
                    Test
                  </v-btn>
                </template>
              </v-tooltip>
              <button
                v-if="isCurrentProviderConfigured"
                class="delete-btn"
                title="Remove saved credentials"
                @click="deleteKey"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                  <path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2m3 0v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6h14z" stroke="currentColor" stroke-width="2"/>
                </svg>
                Remove Key
              </button>
              <div class="spacer"></div>
              <v-btn size="small" variant="text" @click="selectedProvider = null">
                Cancel
              </v-btn>
              <v-btn
                v-if="!isCurrentProviderConfigured"
                size="small"
                color="primary"
                :disabled="!configApiKey"
                @click="saveKey"
              >
                Save
              </v-btn>
            </div>

            <div v-if="testResult" class="test-result" :class="testResult.type">
              <v-icon :icon="testResult.type === 'success' ? 'mdi-check-circle' : 'mdi-alert-circle'" size="16" />
              {{ testResult.message }}
            </div>
          </div>
          <!-- Model List -->
          <div class="model-section">
            <div class="section-header">
              <span class="section-label">AVAILABLE MODELS</span>
              <span class="model-count">{{ filteredModels.length }} / {{ selectedProviderModels.length }}</span>
            </div>

            <!-- Model Search -->
            <div class="model-search">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                <circle cx="11" cy="11" r="8" stroke="currentColor" stroke-width="2"/>
                <line x1="21" y1="21" x2="16.65" y2="16.65" stroke="currentColor" stroke-width="2"/>
              </svg>
              <input
                v-model="modelSearch"
                type="text"
                placeholder="Search models..."
                class="search-input"
              />
              <button v-if="modelSearch" class="clear-search" @click="modelSearch = ''">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                  <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
                  <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
                </svg>
              </button>
            </div>

            <div class="models-scroll">
              <div
                v-for="model in filteredModels"
                :key="model.id"
                class="model-item"
                :class="{ selected: model.id === modelsDevStore.selectedModelId }"
                @click="selectModel(model)"
              >
                <div class="model-info">
                  <div class="model-name-row">
                    <span class="model-name">{{ model.name }}</span>
                    <div class="model-badges">
                      <v-tooltip :text="`Context: ${formatContext(model.limit?.context)}`">
                        <template v-slot:activator="{ props }">
                          <span v-bind="props" class="badge context">
                            {{ formatContextShort(model.limit?.context) }}
                          </span>
                        </template>
                      </v-tooltip>
                      <v-tooltip v-if="model.reasoning" text="Supports reasoning">
                        <template v-slot:activator="{ props }">
                          <span v-bind="props" class="badge reasoning">R</span>
                        </template>
                      </v-tooltip>
                      <v-tooltip v-if="model.tool_call" text="Supports tool calling">
                        <template v-slot:activator="{ props }">
                          <span v-bind="props" class="badge tools">T</span>
                        </template>
                      </v-tooltip>
                    </div>
                  </div>
                  <span class="model-id">{{ model.id }}</span>
                </div>
                <div v-if="model.cost?.input || model.cost?.output" class="model-cost">
                  <v-tooltip :text="buildCostTooltip(model)">
                    <template v-slot:activator="{ props }">
                      <span v-bind="props" class="cost-chip">
                        {{ formatCost(model) }}
                      </span>
                    </template>
                  </v-tooltip>
                </div>
              </div>

              <div v-if="filteredModels.length === 0" class="empty-models">
                <span>No models match "{{ modelSearch }}"</span>
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

const activeCategory = ref<'popular' | 'configured' | 'other' | 'custom'>('popular')
const providerSearch = ref('')
const selectedProvider = ref<ModelsDevProvider | null>(null)
const configApiKey = ref('')
const configBaseUrl = ref('')
const configModelId = ref('')
const configEndpoint = ref('')
const showKey = ref(false)
const testing = ref(false)
const modelSearch = ref('')
const testResult = ref<{ type: 'success' | 'error'; message: string } | null>(null)

const categories = [
  { id: 'popular' as const, label: 'Popular' },
  { id: 'configured' as const, label: 'Configured' },
  { id: 'other' as const, label: 'Other' },
  { id: 'custom' as const, label: 'Custom' },
]

const loading = computed(() => modelsDevStore.loading || providerConfigStore.loading)
const error = computed(() => modelsDevStore.error || providerConfigStore.error)

// Special built-in custom provider entry
const CUSTOM_PROVIDER_ENTRY: ModelsDevProvider = {
  id: '__custom__',
  name: 'Custom Provider',
  models: {},
  isCustom: true,
}

const filteredProviders = computed(() => {
  let providers: ModelsDevProvider[]
  switch (activeCategory.value) {
    case 'popular': providers = modelsDevStore.popularProviders; break
    case 'configured': providers = [...modelsDevStore.popularProviders, ...modelsDevStore.otherProviders].filter(p => isConfigured(p.id)); break
    case 'other': providers = modelsDevStore.otherProviders; break
    case 'custom': providers = [...modelsDevStore.customProviders, CUSTOM_PROVIDER_ENTRY]; break
    default: return []
  }
  if (!providerSearch.value) return providers
  const q = providerSearch.value.toLowerCase()
  return providers.filter(p => p.name.toLowerCase().includes(q) || p.id.toLowerCase().includes(q))
})

const selectedProviderModels = computed(() => {
  if (!selectedProvider.value) return []
  return Object.values(selectedProvider.value.models)
})

const filteredModels = computed(() => {
  if (!modelSearch.value) return selectedProviderModels.value
  const q = modelSearch.value.toLowerCase()
  return selectedProviderModels.value.filter(m =>
    m.name.toLowerCase().includes(q) ||
    m.id.toLowerCase().includes(q)
  )
})

const isCurrentProviderConfigured = computed(() =>
  selectedProvider.value
    ? providerConfigStore.isProviderConfigured(selectedProvider.value.id)
    : false
)

function getCategoryCount(category: string) {
  switch (category) {
    case 'popular': return modelsDevStore.popularProviders.length
    case 'configured': return modelsDevStore.popularProviders.filter(p => isConfigured(p.id)).length + modelsDevStore.otherProviders.filter(p => isConfigured(p.id)).length
    case 'other': return modelsDevStore.otherProviders.length
    case 'custom': return modelsDevStore.customProviders.length + 1 // +1 for Add Custom Provider entry
    default: return 0
  }
}

function isConfigured(providerId: string) {
  return providerConfigStore.isProviderConfigured(providerId)
}

function selectProvider(provider: ModelsDevProvider) {
  selectedProvider.value = provider
  configApiKey.value = ''
  configBaseUrl.value = ''
  configModelId.value = ''
  configEndpoint.value = provider.api || ''
  showKey.value = false
  testResult.value = null
  modelSearch.value = ''
}

function selectModel(model: ModelsDevModel) {
  modelsDevStore.selectModel(model.id)
  modelsDevStore.selectProvider(selectedProvider.value!.id)
}

// Handle provider main area click - opens config for all providers
function handleProviderClick(provider: ModelsDevProvider) {
  // Always open config view, but populate endpoint from stored config if configured
  selectProvider(provider)
  // If provider is configured, load its stored endpoint
  if (isConfigured(provider.id)) {
    // Load stored endpoint for this provider
    loadStoredEndpoint(provider.id)
  }
}

async function loadStoredEndpoint(providerId: string) {
  try {
    const keyInfo = await providerConfigStore.getApiKey(providerId)
    if (keyInfo?.endpoint) {
      configEndpoint.value = keyInfo.endpoint
    }
  } catch (e) {
    console.warn('Failed to load stored endpoint:', e)
  }
}

// Directly switch to a configured provider and close dialog
function selectProviderDirectly(provider: ModelsDevProvider) {
  modelsDevStore.selectProvider(provider.id)
  // Close the dialog
  emit('update:modelValue', false)
}

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
    selectedProvider.value.isCustom
      ? (configBaseUrl.value || undefined)
      : (configEndpoint.value || undefined)
  )
  if (configModelId.value) {
    modelsDevStore.selectModel(configModelId.value)
  }
  selectedProvider.value = null
}

async function deleteKey() {
  if (!selectedProvider.value) return
  await providerConfigStore.deleteApiKey(selectedProvider.value.id)
  selectedProvider.value = null
}

async function testConnection() {
  testing.value = true
  testResult.value = null
  try {
    // Simulate test - in real impl, this would call backend
    await new Promise(r => setTimeout(r, 1500))
    testResult.value = { type: 'success', message: 'Connection successful!' }
  } catch (e) {
    testResult.value = { type: 'error', message: 'Connection failed. Check your credentials.' }
  } finally {
    testing.value = false
  }
}

function formatContext(len?: number): string {
  if (!len) return 'N/A'
  if (len >= 1000000) return `${(len / 1000000).toFixed(1)}M`
  if (len >= 1000) return `${(len / 1000).toFixed(0)}K`
  return len.toString()
}

function formatContextShort(len?: number): string {
  if (!len) return 'N/A'
  if (len >= 1000000) return `${(len / 1000000).toFixed(0)}M`
  if (len >= 1000) return `${(len / 1000).toFixed(0)}K`
  return len.toString()
}

function formatCost(model: ModelsDevModel): string {
  const input = model.cost?.input
  const output = model.cost?.output
  if (!input && !output) return ''
  const fmt = (n: number) => `$${(n / 1000000).toFixed(1)}`
  if (input && output) return `${fmt(input)}/${fmt(output)}`
  if (input !== undefined) return `in: ${fmt(input)}`
  return output !== undefined ? `out: ${fmt(output)}` : ''
}

function buildCostTooltip(model: ModelsDevModel): string {
  const parts: string[] = []
  if (model.cost?.input) parts.push(`Input: $${(model.cost.input / 1000000).toFixed(2)}/1M tokens`)
  if (model.cost?.output) parts.push(`Output: $${(model.cost.output / 1000000).toFixed(2)}/1M tokens`)
  if (model.cost?.cache_read) parts.push(`Cache read: $${(model.cost.cache_read / 1000000).toFixed(2)}/1M`)
  if (model.cost?.cache_write) parts.push(`Cache write: $${(model.cost.cache_write / 1000000).toFixed(2)}/1M`)
  return parts.length ? parts.join('\n') : 'No cost data'
}

function isUrl(str: string): boolean {
  try {
    new URL(str)
    return true
  } catch {
    return false
  }
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

.provider-search {
  margin-top: 10px;
  padding: 8px 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: #ffffff;
  font-size: 12px;
  outline: none;
  width: 100%;
  box-sizing: border-box;
}

.provider-search:focus {
  border-color: rgba(0, 255, 255, 0.3);
}

.provider-search::placeholder {
  color: rgba(255, 255, 255, 0.3);
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
  transition: all 0.15s ease;
}

.provider-main {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  cursor: pointer;
}

.provider-main:hover {
  background: rgba(255, 255, 255, 0.03);
  border-radius: 10px;
  margin: -4px;
  padding: 4px;
}

.provider-item:hover {
  background: rgba(255, 255, 255, 0.03);
}

.provider-item.configured {
  background: rgba(0, 255, 136, 0.03);
}

.select-btn {
  padding: 6px 14px;
  background: rgba(0, 255, 255, 0.1);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 6px;
  color: #00ffff;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.select-btn:hover {
  background: rgba(0, 255, 255, 0.2);
  border-color: rgba(0, 255, 255, 0.5);
}

.select-btn.active {
  background: rgba(0, 255, 136, 0.15);
  border-color: rgba(0, 255, 136, 0.4);
  color: #00ff88;
  cursor: default;
}

.provider-logo {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.2) 0%, rgba(0, 200, 255, 0.1) 100%);
  border: 1px solid rgba(0, 255, 255, 0.4);
  border-radius: 10px;
  flex-shrink: 0;
  overflow: hidden;
  box-shadow:
    0 0 16px rgba(0, 255, 255, 0.25),
    0 0 32px rgba(0, 255, 255, 0.1),
    inset 0 0 12px rgba(0, 255, 255, 0.1);
  animation: logo-pulse 3s ease-in-out infinite;
}

@keyframes logo-pulse {
  0%, 100% {
    box-shadow:
      0 0 16px rgba(0, 255, 255, 0.25),
      0 0 32px rgba(0, 255, 255, 0.1),
      inset 0 0 12px rgba(0, 255, 255, 0.1);
  }
  50% {
    box-shadow:
      0 0 20px rgba(0, 255, 255, 0.35),
      0 0 40px rgba(0, 255, 255, 0.15),
      inset 0 0 16px rgba(0, 255, 255, 0.15);
  }
}

.provider-logo.large {
  width: 48px;
  height: 48px;
  border-radius: 12px;
}

.provider-logo img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  filter: brightness(1.2) saturate(1.1);
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

.provider-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.endpoint-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  padding: 2px 6px;
  background: rgba(0, 255, 255, 0.05);
  border: 1px solid rgba(0, 255, 255, 0.15);
  border-radius: 4px;
  color: rgba(255, 255, 255, 0.4);
  font-family: 'JetBrains Mono', monospace;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.endpoint-tag svg {
  flex-shrink: 0;
  opacity: 0.5;
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

/* Endpoint Display (non-custom) */
.endpoint-display {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.endpoint-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  color: rgba(255, 255, 255, 0.4);
}

.endpoint-value {
  font-size: 13px;
  font-family: 'JetBrains Mono', monospace;
  color: rgba(255, 255, 255, 0.6);
  word-break: break-all;
}

.test-btn {
  align-self: flex-start;
}

/* Config Form (custom) */
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
  align-items: center;
  padding-top: 4px;
}

.spacer {
  flex: 1;
}

.delete-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: rgba(255, 107, 107, 0.08);
  border: 1px solid rgba(255, 107, 107, 0.3);
  border-radius: 6px;
  color: #ff6b6b;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.delete-btn:hover {
  background: rgba(255, 107, 107, 0.15);
  border-color: rgba(255, 107, 107, 0.5);
}

.delete-btn svg {
  flex-shrink: 0;
}

/* Test Result */
.test-result {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  font-size: 12px;
}

.test-result.success {
  background: rgba(0, 255, 136, 0.1);
  color: #00ff88;
}

.test-result.error {
  background: rgba(255, 107, 107, 0.1);
  color: #ff6b6b;
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

/* Model Search */
.model-search {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0 16px 8px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
}

.model-search svg {
  color: rgba(255, 255, 255, 0.3);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  color: #ffffff;
  font-size: 13px;
  outline: none;
}

.search-input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}

.clear-search {
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.3);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.clear-search:hover {
  color: rgba(255, 255, 255, 0.6);
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

.model-item.selected {
  background: rgba(0, 255, 255, 0.08);
}

.model-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.model-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
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
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 9px;
  font-weight: 700;
  border-radius: 4px;
  cursor: help;
}

.badge.context {
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.6);
}

.badge.reasoning {
  width: 18px;
  height: 18px;
  background: rgba(168, 85, 247, 0.2);
  color: #a855f7;
}

.badge.tools {
  width: 18px;
  height: 18px;
  background: rgba(0, 212, 255, 0.2);
  color: #00d4ff;
}

.model-cost {
  flex-shrink: 0;
}

.cost-chip {
  font-size: 10px;
  padding: 2px 6px;
  background: rgba(0, 255, 136, 0.08);
  color: #00ff88;
  border-radius: 4px;
  cursor: help;
}

.empty-models {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  color: rgba(255, 255, 255, 0.25);
  font-size: 12px;
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
