# Models.dev Provider Selection Refactoring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor AI Chat provider/model selection to use models.dev API as the single source of truth, with Popular/Other tabs for providers and a separate custom provider configuration flow.

**Architecture:** Create a dedicated `useModelsDevStore` (Pinia) that owns all models.dev state — provider list, selected provider/model, models data. The existing `useAiChatStore` is stripped of models-dev logic and keeps only session/chat state. API key management lives in a separate `useProviderConfigStore`.

---

## File Structure

### New Files
- `src/stores/modelsDev.ts` — Models.dev state store (provider list, popular/other, selected model)
- `src/stores/providerConfig.ts` — API key management per provider
- `src/types/modelsDev.ts` — TypeScript types for models.dev entities
- `src/api/modelsDev.ts` — API calls for models.dev (fetch, search, etc.)
- `src/views/AiChat/components/ProviderSelector.vue` — Popular/Other tab UI
- `src/views/AiChat/components/ModelList.vue` — Model list for selected provider
- `src/views/AiChat/components/CustomProviderForm.vue` — Custom provider config form
- `src/components/common/ProviderIcon.vue` (update) — handle all provider icons

### Modified Files
- `src/views/AiChat/ModelSettings.vue` — Replace flat provider list with ProviderSelector
- `src/views/AiChat/ModelSelector.vue` — Wire to useModelsDevStore instead of useAiChatStore
- `src/views/AiChat/index.vue` — Update model name display, remove legacy store usage
- `src/stores/aiChat.ts` — Remove models-dev logic, keep only session/chat state
- `src/stores/providers.ts` — Remove (replaced by providerConfig.ts)
- `src/api/ai-chat.ts` — Remove models-dev types (moved to types/modelsDev.ts)
- `src/agent/loop.ts` — Update to use new store

---

## Task 1: Create Type Definitions

**Files:**
- Create: `src/types/modelsDev.ts`

- [ ] **Step 1: Create type definitions**

```typescript
// src/types/modelsDev.ts

export interface Modalities {
  input: string[]
  output: string[]
}

export interface ModelCost {
  input?: number
  output?: number
  cache_read?: number
  cache_write?: number
}

export interface ModelLimits {
  context?: number
  input?: number
  output?: number
}

export interface ModelsDevModel {
  id: string
  name: string
  family?: string
  attachment: boolean
  reasoning: boolean
  tool_call: boolean
  structured_output?: boolean
  temperature: boolean
  knowledge?: string
  release_date?: string
  last_updated?: string
  modalities: Modalities
  open_weights: boolean
  cost?: ModelCost
  limit?: ModelLimits
}

export type ProviderCategory = 'popular' | 'other'

export interface ModelsDevProvider {
  id: string
  name: string
  api?: string
  doc?: string
  npm?: string
  env?: string[]
  models: Record<string, ModelsDevModel>
  // App-specific
  isCustom?: boolean
}

export interface ProviderSummary {
  id: string
  name: string
  api?: string
  doc?: string
  model_count: number
  category: ProviderCategory
}

// Configured provider (from backend)
export interface ConfiguredProvider {
  provider: string
  has_key: boolean
}
```

- [ ] **Step 2: Commit**

```bash
git add src/types/modelsDev.ts
git commit -m "feat(ai-chat): add models.dev type definitions"
```

---

## Task 2: Create models.dev API Layer

**Files:**
- Create: `src/api/modelsDev.ts`

- [ ] **Step 1: Create API layer**

```typescript
// src/api/modelsDev.ts
import { invoke } from '@tauri-apps/api/core'
import type { ModelsDevProvider, ModelsDevModel, ProviderSummary } from '@/types/modelsDev'

export type ModelsDevResponse = Record<string, ModelsDevProvider>

/**
 * Fetch all providers and models from models.dev
 */
export async function fetchModelsDev(forceRefresh?: boolean): Promise<ModelsDevResponse> {
  return await invoke<ModelsDevResponse>('fetch_models_dev', { forceRefresh })
}

/**
 * Get a single provider's info
 */
export async function getModelsDevProvider(providerId: string): Promise<ModelsDevProvider | null> {
  return await invoke<ModelsDevProvider | null>('get_models_dev_provider', { providerId })
}

/**
 * Search models by name across all or a specific provider
 */
export async function searchModelsDev(
  query: string,
  providerFilter?: string
): Promise<Array<[string, ModelsDevModel]>> {
  return await invoke('search_models_dev', { query, providerFilter })
}

/**
 * Get models for a specific provider (with ID mapping)
 */
export async function getModelsForProvider(providerId: string): Promise<ModelsDevModel[]> {
  return await invoke('get_models_for_provider', { providerId })
}

/**
 * Get provider summaries (lightweight list)
 */
export async function getModelsProviders(): Promise<ProviderSummary[]> {
  return await invoke('get_models_providers')
}
```

- [ ] **Step 2: Commit**

```bash
git add src/api/modelsDev.ts
git commit -m "feat(ai-chat): add models.dev API layer"
```

---

## Task 3: Create ProviderConfig Store

**Files:**
- Create: `src/stores/providerConfig.ts`

- [ ] **Step 1: Create provider config store**

```typescript
// src/stores/providerConfig.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ConfiguredProvider } from '@/types/modelsDev'

export const useProviderConfigStore = defineStore('providerConfig', () => {
  const configuredProviders = ref<ConfiguredProvider[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadConfiguredProviders(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      configuredProviders.value = await invoke<ConfiguredProvider[]>('list_api_key_providers')
    } catch (e: any) {
      error.value = e.toString()
    } finally {
      loading.value = false
    }
  }

  async function saveApiKey(provider: string, key: string, endpoint?: string): Promise<void> {
    await invoke('store_api_key', {
      request: { provider, key, endpoint: endpoint || null }
    })
    await loadConfiguredProviders()
  }

  async function deleteApiKey(provider: string): Promise<void> {
    await invoke('delete_api_key', { provider })
    await loadConfiguredProviders()
  }

  async function hasApiKey(provider: string): Promise<boolean> {
    return await invoke<boolean>('has_api_key', { provider })
  }

  function isProviderConfigured(providerId: string): boolean {
    return configuredProviders.value.some(p => p.provider === providerId && p.has_key)
  }

  return {
    configuredProviders,
    loading,
    error,
    loadConfiguredProviders,
    saveApiKey,
    deleteApiKey,
    hasApiKey,
    isProviderConfigured,
  }
})
```

- [ ] **Step 2: Commit**

```bash
git add src/stores/providerConfig.ts
git commit -m "feat(ai-chat): add provider config store for API key management"
```

---

## Task 4: Create useModelsDevStore

**Files:**
- Create: `src/stores/modelsDev.ts`

- [ ] **Step 1: Create the main models.dev store**

```typescript
// src/stores/modelsDev.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  fetchModelsDev as fetchModelsDevApi,
  getModelsForProvider as getModelsForProviderApi,
} from '@/api/modelsDev'
import type { ModelsDevProvider, ModelsDevModel, ProviderCategory } from '@/types/modelsDev'
import { useProviderConfigStore } from './providerConfig'

// Popular provider IDs (hardcoded for now — could be from config/API)
const POPULAR_PROVIDER_IDS = new Set([
  'openai',
  'anthropic',
  'google',
  'deepseek',
  'xai',
  'mistral',
  'moonshot',
  'zhipu',
])

export const useModelsDevStore = defineStore('modelsDev', () => {
  // Raw data
  const providersData = ref<Record<string, ModelsDevProvider>>({})
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Selection state
  const selectedProviderId = ref<string | null>(null)
  const selectedModelId = ref<string | null>(null)

  // Provider config store reference
  const providerConfigStore = useProviderConfigStore()

  // Computed: all providers as array
  const allProviders = computed((): ModelsDevProvider[] => {
    return Object.values(providersData.value)
  })

  // Computed: popular providers (subset with API keys or in popular list)
  const popularProviders = computed((): ModelsDevProvider[] => {
    return allProviders.value.filter(p => {
      if (p.isCustom) return false
      return POPULAR_PROVIDER_IDS.has(p.id) || providerConfigStore.isProviderConfigured(p.id)
    })
  })

  // Computed: other providers (not in popular list, not custom)
  const otherProviders = computed((): ModelsDevProvider[] => {
    return allProviders.value.filter(p => {
      if (p.isCustom) return false
      return !POPULAR_PROVIDER_IDS.has(p.id)
    })
  })

  // Computed: custom providers
  const customProviders = computed((): ModelsDevProvider[] => {
    return allProviders.value.filter(p => p.isCustom)
  })

  // Computed: current provider
  const selectedProvider = computed((): ModelsDevProvider | null => {
    if (!selectedProviderId.value) return null
    return providersData.value[selectedProviderId.value] || null
  })

  // Computed: models for selected provider
  const selectedProviderModels = computed((): ModelsDevModel[] => {
    if (!selectedProvider.value) return []
    return Object.values(selectedProvider.value.models)
  })

  // Computed: current model
  const selectedModel = computed((): ModelsDevModel | null => {
    if (!selectedModelId.value || !selectedProvider.value) return null
    return selectedProvider.value.models[selectedModelId.value] || null
  })

  // Computed: check if a provider has API key configured
  function isProviderConfigured(providerId: string): boolean {
    return providerConfigStore.isProviderConfigured(providerId)
  }

  // Actions
  async function fetchProviders(forceRefresh = false): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const data = await fetchModelsDevApi(forceRefresh)
      providersData.value = data
      // Auto-select first provider if none selected
      if (!selectedProviderId.value && allProviders.value.length > 0) {
        const firstConfigured = allProviders.value.find(p => isProviderConfigured(p.id))
        selectedProviderId.value = firstConfigured?.id || allProviders.value[0]?.id || null
      }
    } catch (e: any) {
      error.value = e.toString()
    } finally {
      loading.value = false
    }
  }

  async function fetchModelsForProvider(providerId: string): Promise<ModelsDevModel[]> {
    return await getModelsForProviderApi(providerId)
  }

  function selectProvider(providerId: string): void {
    selectedProviderId.value = providerId
    // Clear model selection when provider changes
    selectedModelId.value = null
  }

  function selectModel(modelId: string): void {
    selectedModelId.value = modelId
  }

  function getProviderModels(providerId: string): ModelsDevModel[] {
    const provider = providersData.value[providerId]
    if (!provider) return []
    return Object.values(provider.models)
  }

  return {
    // State
    providersData,
    loading,
    error,
    selectedProviderId,
    selectedModelId,

    // Computed
    allProviders,
    popularProviders,
    otherProviders,
    customProviders,
    selectedProvider,
    selectedProviderModels,
    selectedModel,

    // Methods
    isProviderConfigured,
    fetchProviders,
    fetchModelsForProvider,
    selectProvider,
    selectModel,
    getProviderModels,
  }
})
```

- [ ] **Step 2: Commit**

```bash
git add src/stores/modelsDev.ts
git commit -m "feat(ai-chat): add useModelsDevStore for models.dev state management"
```

---

## Task 5: Create ProviderSelector Component

**Files:**
- Create: `src/views/AiChat/components/ProviderSelector.vue`

- [ ] **Step 1: Create ProviderSelector component**

```vue
<!-- src/views/AiChat/components/ProviderSelector.vue -->
<template>
  <div class="provider-selector">
    <!-- Tabs: Popular / Other / Custom -->
    <v-tabs v-model="activeTab" color="primary" density="compact">
      <v-tab value="popular">
        <v-icon icon="mdi-star" start size="14" />
        Popular
        <v-chip v-if="popularCount > 0" size="x-small" class="ml-1">
          {{ popularCount }}
        </v-chip>
      </v-tab>
      <v-tab value="other">
        <v-icon icon="mdi-dots-horizontal" start size="14" />
        Other
        <v-chip v-if="otherCount > 0" size="x-small" class="ml-1">
          {{ otherCount }}
        </v-chip>
      </v-tab>
      <v-tab value="custom">
        <v-icon icon="mdi-plus-circle" start size="14" />
        Custom
        <v-chip v-if="customCount > 0" size="x-small" class="ml-1">
          {{ customCount }}
        </v-chip>
      </v-tab>
    </v-tabs>

    <v-divider />

    <!-- Provider Grid -->
    <div class="provider-grid">
      <div
        v-for="provider in displayedProviders"
        :key="provider.id"
        class="provider-card"
        :class="{
          selected: provider.id === store.selectedProviderId,
          configured: store.isProviderConfigured(provider.id),
        }"
        @click="selectProvider(provider)"
      >
        <div class="provider-icon">
          <v-icon :icon="getProviderIcon(provider.id)" size="24" />
        </div>
        <div class="provider-info">
          <span class="provider-name">{{ provider.name }}</span>
          <span class="provider-models">{{ Object.keys(provider.models).length }} models</span>
        </div>
        <div class="provider-status">
          <v-chip
            v-if="store.isProviderConfigured(provider.id)"
            size="x-small"
            color="success"
          >
            CONFIGURED
          </v-chip>
          <v-chip v-else size="x-small" variant="outlined">
            NOT SET
          </v-chip>
        </div>
        <v-icon
          v-if="provider.id === store.selectedProviderId"
          icon="mdi-check-circle"
          color="primary"
          size="18"
          class="check-icon"
        />
      </div>

      <!-- Empty state -->
      <div v-if="displayedProviders.length === 0" class="empty-state">
        <v-icon icon="mdi-inbox-outline" size="32" color="surface-variant" />
        <p>No providers in this category</p>
      </div>
    </div>

    <!-- Configure Button -->
    <div class="config-section">
      <v-btn
        block
        color="primary"
        :disabled="!store.selectedProviderId"
        @click="$emit('configure', store.selectedProviderId)"
      >
        <v-icon icon="mdi-cog" start />
        Configure Selected Provider
      </v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useModelsDevStore } from '@/stores/modelsDev'
import type { ModelsDevProvider } from '@/types/modelsDev'

const emit = defineEmits<{
  (e: 'configure', providerId: string): void
}>()

const store = useModelsDevStore()
const activeTab = ref<'popular' | 'other' | 'custom'>('popular')

const displayedProviders = computed(() => {
  switch (activeTab.value) {
    case 'popular':
      return store.popularProviders
    case 'other':
      return store.otherProviders
    case 'custom':
      return store.customProviders
    default:
      return []
  }
})

const popularCount = computed(() => store.popularProviders.length)
const otherCount = computed(() => store.otherProviders.length)
const customCount = computed(() => store.customProviders.length)

function selectProvider(provider: ModelsDevProvider) {
  store.selectProvider(provider.id)
}

function getProviderIcon(providerId: string): string {
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
  return icons[providerId] || 'mdi-cloud'
}
</script>

<style scoped>
.provider-selector {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.provider-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  max-height: 300px;
  overflow-y: auto;
}

.provider-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.provider-card:hover {
  background: rgba(0, 255, 255, 0.05);
  border-color: rgba(0, 255, 255, 0.3);
}

.provider-card.selected {
  border-color: rgba(0, 255, 136, 0.5);
  background: rgba(0, 255, 136, 0.05);
}

.provider-card.configured {
  border-left: 3px solid #00ff88;
}

.provider-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 255, 255, 0.05);
  border-radius: 8px;
}

.provider-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.provider-name {
  font-size: 14px;
  font-weight: 600;
  color: #e4e4e7;
}

.provider-models {
  font-size: 11px;
  color: #71717a;
}

.provider-status {
  display: flex;
  align-items: center;
}

.check-icon {
  position: absolute;
  right: 12px;
}

.config-section {
  padding: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: rgba(255, 255, 255, 0.4);
  gap: 8px;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AiChat/components/ProviderSelector.vue
git commit -m "feat(ai-chat): add ProviderSelector component with Popular/Other/Custom tabs"
```

---

## Task 6: Create ModelList Component

**Files:**
- Create: `src/views/AiChat/components/ModelList.vue`

- [ ] **Step 1: Create ModelList component**

```vue
<!-- src/views/AiChat/components/ModelList.vue -->
<template>
  <div class="model-list">
    <!-- Provider Header -->
    <div class="provider-header">
      <v-btn variant="text" size="small" @click="$emit('back')">
        <v-icon icon="mdi-arrow-left" start />
        Back
      </v-btn>
      <span class="provider-name">{{ providerName }}</span>
      <v-chip size="small" variant="outlined">
        {{ models.length }} models
      </v-chip>
    </div>

    <!-- Search -->
    <div class="search-wrapper">
      <v-text-field
        v-model="searchQuery"
        placeholder="Search models..."
        prepend-inner-icon="mdi-magnify"
        variant="solo-filled"
        density="compact"
        hide-details
        single-line
        clearable
      />
    </div>

    <!-- Model Cards -->
    <div class="models-grid">
      <div
        v-for="model in filteredModels"
        :key="model.id"
        class="model-card"
        :class="{ selected: model.id === store.selectedModelId }"
        @click="selectModel(model)"
      >
        <div class="model-header">
          <span class="model-name">{{ model.name }}</span>
          <div class="model-badges">
            <v-chip v-if="model.reasoning" size="x-small" color="purple">
              REASONING
            </v-chip>
            <v-chip v-if="model.tool_call" size="x-small" color="blue">
              TOOLS
            </v-chip>
          </div>
        </div>

        <div class="model-meta">
          <span v-if="model.limit?.context" class="meta-item">
            <v-icon icon="mdi-memory" size="12" />
            {{ formatContext(model.limit.context) }}
          </span>
          <span v-if="model.cost?.input && model.cost?.output" class="meta-item">
            <v-icon icon="mdi-currency-usd" size="12" />
            ${{ model.cost.input }}/{{ model.cost.output }}
          </span>
        </div>

        <div class="model-modalities">
          <span v-if="model.modalities.input.includes('text')" class="modality">T</span>
          <span v-if="model.modalities.input.includes('image')" class="modality">I</span>
          <span v-if="model.modalities.output.includes('audio')" class="modality">A</span>
        </div>

        <v-icon
          v-if="model.id === store.selectedModelId"
          icon="mdi-check-circle"
          color="primary"
          size="20"
          class="selected-check"
        />
      </div>

      <!-- Empty state -->
      <div v-if="filteredModels.length === 0" class="empty-state">
        <v-icon icon="mdi-magnify-close" size="32" color="surface-variant" />
        <p>No models found</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useModelsDevStore } from '@/stores/modelsDev'
import type { ModelsDevModel } from '@/types/modelsDev'

defineProps<{
  providerName: string
  models: ModelsDevModel[]
}>()

const emit = defineEmits<{
  (e: 'back'): void
  (e: 'select', model: ModelsDevModel): void
}>()

const store = useModelsDevStore()
const searchQuery = ref('')

const filteredModels = computed(() => {
  if (!searchQuery.value) return store.selectedProviderModels
  const query = searchQuery.value.toLowerCase()
  return store.selectedProviderModels.filter(
    m => m.name.toLowerCase().includes(query) || m.id.toLowerCase().includes(query)
  )
})

function selectModel(model: ModelsDevModel) {
  store.selectModel(model.id)
  emit('select', model)
}

function formatContext(length: number): string {
  if (length >= 1000000) return `${(length / 1000000).toFixed(0)}M`
  if (length >= 1000) return `${(length / 1000).toFixed(0)}K`
  return length.toString()
}
</script>

<style scoped>
.model-list {
  display: flex;
  flex-direction: column;
}

.provider-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.provider-name {
  flex: 1;
  font-size: 14px;
  font-weight: 600;
}

.search-wrapper {
  padding: 12px;
}

.models-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 12px 12px;
  max-height: 350px;
  overflow-y: auto;
}

.model-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.model-card:hover {
  background: rgba(0, 255, 255, 0.05);
  border-color: rgba(0, 255, 255, 0.3);
}

.model-card.selected {
  border-color: rgba(0, 255, 136, 0.5);
  background: rgba(0, 255, 136, 0.05);
}

.model-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.model-name {
  font-size: 13px;
  font-weight: 600;
  color: #e4e4e7;
}

.model-badges {
  display: flex;
  gap: 4px;
}

.model-meta {
  display: flex;
  gap: 12px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: #71717a;
}

.model-modalities {
  display: flex;
  gap: 4px;
}

.modality {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 255, 255, 0.1);
  border-radius: 4px;
  font-size: 10px;
  font-weight: bold;
  color: #00ffff;
}

.selected-check {
  position: absolute;
  top: 8px;
  right: 8px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: rgba(255, 255, 255, 0.4);
  gap: 8px;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AiChat/components/ModelList.vue
git commit -m "feat(ai-chat): add ModelList component for model selection"
```

---

## Task 7: Create CustomProviderForm Component

**Files:**
- Create: `src/views/AiChat/components/CustomProviderForm.vue`

- [ ] **Step 1: Create CustomProviderForm component**

```vue
<!-- src/views/AiChat/components/CustomProviderForm.vue -->
<template>
  <div class="custom-provider-form">
    <div class="form-header">
      <v-icon icon="mdi-plus-circle" color="primary" />
      <span>Add Custom Provider</span>
    </div>

    <div class="form-fields">
      <div class="form-field">
        <label class="field-label">PROVIDER NAME</label>
        <input
          v-model="form.name"
          type="text"
          class="console-input"
          placeholder="My Custom Provider"
        />
      </div>

      <div class="form-field">
        <label class="field-label">BASE URL</label>
        <input
          v-model="form.baseUrl"
          type="text"
          class="console-input"
          placeholder="https://api.example.com/v1"
        />
      </div>

      <div class="form-field">
        <label class="field-label">API KEY</label>
        <div class="input-wrapper">
          <input
            v-model="form.apiKey"
            :type="showKey ? 'text' : 'password'"
            class="console-input"
            placeholder="sk-..."
          />
          <button class="toggle-btn" @click="showKey = !showKey">
            {{ showKey ? 'HIDE' : 'SHOW' }}
          </button>
        </div>
      </div>

      <div class="form-field">
        <label class="field-label">MODEL ID (OPTIONAL)</label>
        <input
          v-model="form.modelId"
          type="text"
          class="console-input"
          placeholder="e.g., gpt-4o"
        />
      </div>
    </div>

    <div class="form-actions">
      <v-btn variant="text" @click="$emit('cancel')">Cancel</v-btn>
      <v-btn color="primary" :disabled="!isValid" @click="handleSave">
        Save Provider
      </v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useProviderConfigStore } from '@/stores/providerConfig'

const emit = defineEmits<{
  (e: 'save', data: { name: string; baseUrl: string; apiKey: string; modelId?: string }): void
  (e: 'cancel'): void
}>()

const providerConfigStore = useProviderConfigStore()
const showKey = ref(false)
const form = ref({
  name: '',
  baseUrl: '',
  apiKey: '',
  modelId: '',
})

const isValid = computed(() => {
  return form.value.name.trim() && form.value.baseUrl.trim() && form.value.apiKey.trim()
})

async function handleSave() {
  emit('save', {
    name: form.value.name,
    baseUrl: form.value.baseUrl,
    apiKey: form.value.apiKey,
    modelId: form.value.modelId || undefined,
  })
}
</script>

<style scoped>
.custom-provider-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
}

.form-fields {
  display: flex;
  flex-direction: column;
  gap: 12px;
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

.console-input {
  width: 100%;
  padding: 8px 12px;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 6px;
  color: #e4e4e7;
  font-size: 13px;
}

.console-input:focus {
  outline: none;
  border-color: rgba(0, 255, 255, 0.5);
}

.input-wrapper {
  display: flex;
  gap: 8px;
}

.input-wrapper .console-input {
  flex: 1;
}

.toggle-btn {
  padding: 0 12px;
  background: rgba(0, 255, 255, 0.1);
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 6px;
  color: #00ffff;
  font-size: 10px;
  cursor: pointer;
}

.toggle-btn:hover {
  background: rgba(0, 255, 255, 0.2);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AiChat/components/CustomProviderForm.vue
git commit -m "feat(ai-chat): add CustomProviderForm component"
```

---

## Task 8: Update ModelSettings.vue

**Files:**
- Modify: `src/views/AiChat/ModelSettings.vue`

- [ ] **Step 1: Update ModelSettings to use new components**

Replace the existing flat provider list with a step-by-step flow:
1. ProviderSelector (Popular/Other/Custom tabs)
2. ModelList (when provider selected)
3. CustomProviderForm (when adding custom provider)

```vue
<!-- src/views/AiChat/ModelSettings.vue -->
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
        <!-- Loading State -->
        <div v-if="store.loading" class="loading-state">
          <v-progress-circular indeterminate color="primary" />
          <span>Loading providers...</span>
        </div>

        <!-- Error State -->
        <div v-else-if="store.error" class="error-state">
          <v-icon icon="mdi-alert-circle" color="error" />
          <span>{{ store.error }}</span>
          <v-btn size="small" @click="store.fetchProviders(true)">Retry</v-btn>
        </div>

        <!-- Custom Provider Form -->
        <CustomProviderForm
          v-else-if="showCustomForm"
          @save="handleCustomProviderSave"
          @cancel="showCustomForm = false"
        />

        <!-- Provider + Model Selection -->
        <template v-else>
          <!-- Step 1: Provider Selection (always shown) -->
          <div class="section-header">
            <span class="section-title">SELECT PROVIDER</span>
          </div>

          <ProviderSelector @configure="openConfig" />

          <!-- Step 2: Model Selection (when provider selected) -->
          <template v-if="store.selectedProvider && !store.selectedProvider.isCustom">
            <v-divider class="my-4" />
            <div class="section-header">
              <span class="section-title">SELECT MODEL</span>
            </div>
            <ModelList
              :provider-name="store.selectedProvider.name"
              :models="store.selectedProviderModels"
              @back="store.selectProvider('')"
              @select="handleModelSelect"
            />
          </template>
        </template>
      </v-card-text>

      <!-- API Key Config Dialog -->
      <v-dialog v-model="showConfigDialog" max-width="400">
        <v-card class="scifi-card">
          <v-card-title class="console-title-bar">
            <span class="dialog-title">[ {{ currentProviderName }}_CONFIG ]</span>
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
                  :placeholder="currentProviderApi"
                />
              </div>
            </div>
          </v-card-text>
          <v-card-actions class="console-card-actions">
            <button
              v-if="providerConfigStore.isProviderConfigured(currentProviderId)"
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

      <v-card-actions class="console-card-actions">
        <v-spacer />
        <button class="console-btn" @click="saveAndClose">
          <span class="btn-text">DONE</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useModelsDevStore } from '@/stores/modelsDev'
import { useProviderConfigStore } from '@/stores/providerConfig'
import ProviderSelector from './components/ProviderSelector.vue'
import ModelList from './components/ModelList.vue'
import CustomProviderForm from './components/CustomProviderForm.vue'
import type { ModelsDevModel } from '@/types/modelsDev'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const store = useModelsDevStore()
const providerConfigStore = useProviderConfigStore()

const showConfigDialog = ref(false)
const showCustomForm = ref(false)
const currentProviderId = ref('')
const configApiKey = ref('')
const configEndpoint = ref('')
const showKey = ref(false)

const currentProviderName = computed(() => {
  return store.providersData[currentProviderId.value]?.name || currentProviderId.value
})

const currentProviderApi = computed(() => {
  return store.providersData[currentProviderId.value]?.api || ''
})

watch(() => props.modelValue, async (open) => {
  if (open) {
    await providerConfigStore.loadConfiguredProviders()
    if (!store.providersData || Object.keys(store.providersData).length === 0) {
      await store.fetchProviders()
    }
  }
})

function openConfig(providerId: string) {
  currentProviderId.value = providerId
  configApiKey.value = ''
  configEndpoint.value = store.providersData[providerId]?.api || ''
  showKey.value = false
  showConfigDialog.value = true
}

async function saveKey() {
  await providerConfigStore.saveApiKey(
    currentProviderId.value,
    configApiKey.value,
    configEndpoint.value || undefined
  )
  showConfigDialog.value = false
}

async function deleteKey() {
  await providerConfigStore.deleteApiKey(currentProviderId.value)
  showConfigDialog.value = false
}

function handleModelSelect(model: ModelsDevModel) {
  // Model is already selected via store.selectModel() in ModelList
  // Just close if desired, or stay open for more configuration
}

function handleCustomProviderSave(data: { name: string; baseUrl: string; apiKey: string; modelId?: string }) {
  // Save custom provider logic
  console.log('Custom provider:', data)
  showCustomForm.value = false
}

function saveAndClose() {
  emit('update:modelValue', false)
}
</script>

<style scoped>
.settings-content {
  padding: 16px !important;
}

.loading-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 32px;
  color: rgba(255, 255, 255, 0.6);
}

.section-header {
  margin-bottom: 8px;
}

.section-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #00ff88;
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
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AiChat/ModelSettings.vue
git commit -m "refactor(ai-chat): update ModelSettings to use new ProviderSelector and ModelList components"
```

---

## Task 9: Simplify aiChat Store

**Files:**
- Modify: `src/stores/aiChat.ts`

- [ ] **Step 1: Remove models-dev logic from aiChat store**

Remove these items from the store:
- `providers` state
- `modelsDevData`, `modelsDevLoading`, `modelsDevError` state
- `selectedModelId`, `selectedProvider` state
- `fetchModelsDev()`, `getProvidersFromModelsDev()`, `getModelsForProvider()` methods
- `configuredProviders`, `availableModels`, `selectedModel` computed
- `loadProviders()`, `saveApiKey()`, `deleteApiKey()` methods
- `setSelectedModel()` method
- `temperature`, `maxTokens`, `streamEnabled` settings (move to settings store)

Keep only:
- Session management (`sessions`, `currentSessionId`, `currentSession`)
- Chat operations (`isLoading`, `error`, `streamingContent`)
- Message operations (`loadSessions`, `createSession`, `deleteSession`, `sendMessage`)
- MCP server management
- Settings persistence (`loadSettings`, `saveSettings`)

```typescript
// src/stores/aiChat.ts (simplified)
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// Types (keeping for backward compatibility)
export interface McpServer {
  id: string
  name: string
  description?: string
  enabled: boolean
  icon?: string
}

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant'
  content: string
  timestamp?: number
  isStreaming?: boolean
  error?: string
}

export interface Session {
  id: string
  title: string
  messages: ChatMessage[]
  provider: string
  model: string
  created_at: string
  updated_at: string
}

export const DEFAULT_MCP_SERVERS: McpServer[] = [
  { id: 'hosts', name: 'Hosts Manager', description: '管理系统Hosts文件', enabled: false, icon: 'mdi-dns' },
  { id: 'filesystem', name: 'Filesystem', description: '文件系统操作', enabled: false, icon: 'mdi-folder' },
  { id: 'browser', name: 'Browser', description: '网页浏览和搜索', enabled: false, icon: 'mdi-web' },
]

export const useAiChatStore = defineStore('aiChat', () => {
  // Session State
  const sessions = ref<Session[]>([])
  const currentSessionId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const streamingContent = ref<string>('')

  // MCP State
  const enabledMcpServers = ref<string[]>([])
  const mcpServers = ref<McpServer[]>([...DEFAULT_MCP_SERVERS])

  // Computed
  const currentSession = computed(() =>
    sessions.value.find(s => s.id === currentSessionId.value) || null
  )

  // Session Actions
  async function loadSessions() {
    try {
      sessions.value = await invoke('list_sessions')
      error.value = null
    } catch (e) {
      error.value = `加载会话失败: ${e}`
    }
  }

  async function createSession(title: string = '新对话', provider?: string, model?: string) {
    try {
      const session = await invoke<Session>('create_session', {
        title,
        provider: provider || '',
        model: model || ''
      })
      sessions.value.unshift(session)
      currentSessionId.value = session.id
      error.value = null
      return session
    } catch (e) {
      error.value = `创建会话失败: ${e}`
      throw e
    }
  }

  async function deleteSession(sessionId: string) {
    try {
      await invoke('delete_session', { sessionId })
      sessions.value = sessions.value.filter(s => s.id !== sessionId)
      if (currentSessionId.value === sessionId) {
        currentSessionId.value = sessions.value[0]?.id || null
      }
      error.value = null
    } catch (e) {
      error.value = `删除会话失败: ${e}`
      throw e
    }
  }

  async function sendMessage(content: string) {
    if (!currentSession.value) {
      await createSession()
      if (!currentSession.value) {
        throw new Error('没有活动的会话')
      }
    }

    const session = currentSession.value
    const userMessage: ChatMessage = { role: 'user', content, timestamp: Date.now() }
    session.messages.push(userMessage)

    isLoading.value = true
    streamingContent.value = ''
    error.value = null

    try {
      const requestId = `req-${Date.now()}`
      const unlisten = await listen<string>('chat-stream-chunk', (event) => {
        streamingContent.value += event.payload
      })

      try {
        const response = await invoke('send_chat_message_stream', {
          provider: session.provider,
          model: session.model,
          messages: session.messages,
          requestId,
        })

        const assistantMessage: ChatMessage = {
          role: 'assistant',
          content: response as string,
          timestamp: Date.now()
        }
        session.messages.push(assistantMessage)
      } finally {
        unlisten()
      }

      await invoke('append_message', {
        sessionId: session.id,
        message: session.messages[session.messages.length - 1],
      })
    } catch (e) {
      error.value = `发送消息失败: ${e}`
      session.messages.pop()
      throw e
    } finally {
      isLoading.value = false
      streamingContent.value = ''
    }
  }

  // MCP Actions
  function toggleMcpServer(serverId: string) {
    const server = mcpServers.value.find(s => s.id === serverId)
    if (server) {
      server.enabled = !server.enabled
      if (server.enabled) {
        if (!enabledMcpServers.value.includes(serverId)) {
          enabledMcpServers.value.push(serverId)
        }
      } else {
        enabledMcpServers.value = enabledMcpServers.value.filter(id => id !== serverId)
      }
    }
  }

  // Settings (basic localStorage persistence)
  function loadSettings() {
    try {
      const saved = localStorage.getItem('chat-settings')
      if (saved) {
        const settings = JSON.parse(saved)
        if (settings.enabledMcpServers) enabledMcpServers.value = settings.enabledMcpServers
        mcpServers.value.forEach(server => {
          server.enabled = enabledMcpServers.value.includes(server.id)
        })
      }
    } catch (e) {
      console.error('Failed to load AI chat settings:', e)
    }
  }

  function saveSettings() {
    try {
      const settings = {
        enabledMcpServers: enabledMcpServers.value,
      }
      localStorage.setItem('chat-settings', JSON.stringify(settings))
    } catch (e) {
      console.error('Failed to save AI chat settings:', e)
    }
  }

  return {
    // State
    sessions,
    currentSessionId,
    isLoading,
    error,
    streamingContent,
    enabledMcpServers,
    mcpServers,

    // Computed
    currentSession,

    // Actions
    loadSessions,
    createSession,
    deleteSession,
    sendMessage,
    toggleMcpServer,
    loadSettings,
    saveSettings,
  }
})
```

- [ ] **Step 2: Commit**

```bash
git add src/stores/aiChat.ts
git commit -m "refactor(ai-chat): simplify aiChat store, remove models-dev logic"
```

---

## Task 10: Update AiChat index.vue

**Files:**
- Modify: `src/views/AiChat/index.vue`

- [ ] **Step 1: Update currentModelName computed to use modelsDev store**

Change from:
```typescript
const currentModelName = computed(() => {
  const models = store.getModelsForProvider(store.selectedProvider)
  const model = models.find(m => m.id === store.selectedModelId)
  return model?.name || store.selectedModelId || 'SELECT MODEL'
})
```

To:
```typescript
const currentModelName = computed(() => {
  return store.modelsDevStore.selectedModel?.name || 'SELECT MODEL'
})
```

Also add `useModelsDevStore` import and initialization in `onMounted`:
```typescript
const modelsDevStore = useModelsDevStore()

onMounted(async () => {
  skillsStore.loadFromStorage()
  await modelsDevStore.fetchProviders()
  scrollToBottom()
})
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AiChat/index.vue
git commit -m "feat(ai-chat): wire index.vue to useModelsDevStore for model display"
```

---

## Task 11: Update Agent Loop to Use New Store

**Files:**
- Modify: `src/agent/loop.ts`

- [ ] **Step 1: Update agent loop to use modelsDev store**

The agent loop needs to get the selected provider/model from `useModelsDevStore` instead of `useAgentStore` config. Update the relevant code paths that send chat messages.

- [ ] **Step 2: Commit**

```bash
git add src/agent/loop.ts
git commit -m "refactor(agent): update to use modelsDevStore for provider/model selection"
```

---

## Task 12: Delete Deprecated Files

**Files:**
- Delete: `src/stores/providers.ts` (replaced by providerConfig.ts)
- Delete: `src/providers/` directory (old provider adapters — no longer needed with models.dev)

- [ ] **Step 1: Remove deprecated files**

```bash
git rm src/stores/providers.ts
git rm -rf src/providers/
```

- [ ] **Step 2: Commit**

```bash
git commit -m "chore(ai-chat): remove deprecated providers store and adapter directory"
```

---

## Task 13: Update i18n Keys

**Files:**
- Modify: `src/i18n/locales/zh.ts` and `src/i18n/locales/en.ts`

- [ ] **Step 1: Add any new i18n keys needed**

Add keys for:
- `chat.selectProvider`
- `chat.popular`
- `chat.other`
- `chat.custom`
- `chat.configured`
- `chat.notConfigured`

- [ ] **Step 2: Commit**

```bash
git add src/i18n/locales/zh.ts src/i18n/locales/en.ts
git commit -m "i18n: add provider selection UI strings"
```

---

## Self-Review Checklist

- [ ] All spec requirements from screenshots are covered
- [ ] No placeholder comments or TODOs in code
- [ ] Types are consistent across all files
- [ ] Error handling is present for API calls
- [ ] Loading states are handled
- [ ] Back navigation works in ModelList
- [ ] API key management is properly isolated
- [ ] Popular/Other tabs filter correctly
- [ ] Custom provider addition flow works
- [ ] Agent loop is updated to use new store
