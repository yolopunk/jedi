# Models.dev Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace hardcoded provider/model lists with dynamic fetch from models.dev/api.json. Remove temperature/maxTokens/streamEnabled parameters from UI.

**Architecture:** Fetch provider+model list from models.dev API on app load, cache locally. ProviderConfigDialog for API key setup per provider. No more hardcoded provider lists.

**Tech Stack:** Vue 3, Pinia, TypeScript, Tauri

---

## File Structure

- Modify: `src/stores/aiChat.ts` - Replace PROVIDER_CONFIGS with models.dev fetch, remove temperature/maxTokens/stream settings
- Modify: `src/views/AiChat/ModelSettings.vue` - Remove PARAMETERS section, fetch providers from store
- Modify: `src/views/AiChat/index.vue` - Update model selection logic
- Create: `src/types/models.dev.ts` - TypeScript interfaces for models.dev API response

---

## Task 1: Create models.dev TypeScript Types

**Files:**
- Create: `src/types/models.dev.ts`

- [ ] **Step 1: Create type definitions**

```typescript
// src/types/models.dev.ts

export interface ModelInfo {
  id: string
  name: string
  family?: string
  attachment?: boolean
  reasoning?: boolean
  tool_call?: boolean
  temperature?: boolean
  structured_output?: boolean
  knowledge?: string
  release_date?: string
  last_updated?: string
  modalities?: { input: string[]; output: string[] }
  open_weights?: boolean
  cost?: { input: number; output: number; cache_read?: number; cache_write?: number }
  limit?: { context: number; output: number; input?: number }
}

export interface ProviderInfo {
  id: string
  name: string
  api: string
  npm?: string
  env?: string[]
  doc?: string
  models: Record<string, ModelInfo>
}

export type ModelsDevResponse = Record<string, ProviderInfo>
```

- [ ] **Step 2: Commit**

```bash
git add src/types/models.dev.ts
git commit -m "feat: add models.dev TypeScript types"
```

---

## Task 2: Update aiChat Store - Remove Hardcoded Config, Add API Fetch

**Files:**
- Modify: `src/stores/aiChat.ts:67-130` (remove PROVIDER_CONFIGS)
- Modify: `src/stores/aiChat.ts` (add models.dev fetch logic)

- [ ] **Step 1: Remove hardcoded PROVIDER_CONFIGS**

Delete lines 67-130 containing the hardcoded `PROVIDER_CONFIGS` object.

- [ ] **Step 2: Add models.dev fetch to store**

Add these imports:
```typescript
import type { ModelsDevResponse, ProviderInfo, ModelInfo } from '@/types/models.dev'
```

Add these refs after `error`:
```typescript
const modelsDevData = ref<ModelsDevResponse | null>(null)
const modelsDevLoading = ref(false)
const modelsDevError = ref<string | null>(null)
```

Add these functions:
```typescript
async function fetchModelsDev() {
  modelsDevLoading.value = true
  modelsDevError.value = null
  try {
    const response = await fetch('https://models.dev/api.json')
    if (!response.ok) throw new Error('Failed to fetch')
    modelsDevData.value = await response.json()
  } catch (e) {
    modelsDevError.value = 'Failed to load providers'
    console.error('fetchModelsDev error:', e)
  } finally {
    modelsDevLoading.value = false
  }
}

function getProvidersFromModelsDev(): ProviderInfo[] {
  if (!modelsDevData.value) return []
  return Object.values(modelsDevData.value).map(p => ({
    id: p.id,
    name: p.name,
    api: p.api,
    npm: p.npm,
    env: p.env,
    doc: p.doc,
    models: p.models
  }))
}

function getModelsForProvider(providerId: string): ModelInfo[] {
  if (!modelsDevData.value) return []
  const provider = modelsDevData.value[providerId]
  if (!provider) return []
  return Object.values(provider.models)
}
```

Add to return statement:
```typescript
modelsDevData, modelsDevLoading, modelsDevError,
fetchModelsDev, getProvidersFromModelsDev, getModelsForProvider,
```

- [ ] **Step 3: Commit**

```bash
git add src/stores/aiChat.ts
git commit -m "feat: remove hardcoded PROVIDER_CONFIGS, add models.dev fetch"
```

---

## Task 3: Remove Temperature/MaxTokens/Stream Parameters from ModelSettings

**Files:**
- Modify: `src/views/AiChat/ModelSettings.vue` (remove PARAMETERS section and related state)

- [ ] **Step 1: Remove PARAMETERS section from template**

Delete lines 96-140 (the divider-line, section-header, param-group elements).

- [ ] **Step 2: Remove localSettings ref**

Change:
```typescript
const localSettings = ref({
  temperature: 0.7,
  maxTokens: 4096,
  streamEnabled: true,
})
```
To:
```typescript
// Settings removed - using provider defaults
```

- [ ] **Step 3: Update watch handler**

Change:
```typescript
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
```
To:
```typescript
watch(() => props.modelValue, (open) => {
  if (open) {
    store.loadProviders()
    if (!store.modelsDevData) {
      store.fetchModelsDev()
    }
  }
})
```

- [ ] **Step 4: Remove saveAndClose or simplify it**

Change `saveAndClose` to:
```typescript
function saveAndClose() {
  emit('update:modelValue', false)
}
```

- [ ] **Step 5: Remove unused style classes**

Delete `.param-group`, `.param-row`, `.param-label`, `.param-value`, `.console-slider`, `.toggle-switch` style blocks (lines 323-394).

- [ ] **Step 6: Commit**

```bash
git add src/views/AiChat/ModelSettings.vue
git commit -m "feat: remove temperature/maxTokens/stream parameters from ModelSettings"
```

---

## Task 4: Update Provider List to Use Dynamic Data

**Files:**
- Modify: `src/views/AiChat/ModelSettings.vue`

- [ ] **Step 1: Change providerList to computed from store**

Replace:
```typescript
const providerList = [
  { id: 'openai', name: 'OpenAI', defaultEndpoint: 'https://api.openai.com/v1' },
  { id: 'anthropic', name: 'Anthropic', defaultEndpoint: 'https://api.anthropic.com' },
  { id: 'google', name: 'Google', defaultEndpoint: 'https://generativelanguage.googleapis.com' },
  { id: 'deepseek', name: 'DeepSeek', defaultEndpoint: 'https://api.deepseek.com' },
  { id: 'openrouter', name: 'OpenRouter', defaultEndpoint: 'https://openrouter.ai/api/v1' },
  { id: 'ollama', name: 'Ollama', defaultEndpoint: 'http://localhost:11434/v1' },
]
```

With:
```typescript
const providerList = computed(() => store.getProvidersFromModelsDev())
```

- [ ] **Step 2: Update openConfig to use dynamic data**

Change:
```typescript
function openConfig(id: string) {
  currentProvider.value = providerList.find(p => p.id === id) || null
  configApiKey.value = ''
  configEndpoint.value = ''
  showConfigDialog.value = true
}
```
To:
```typescript
function openConfig(id: string) {
  const providers = store.getProvidersFromModelsDev()
  currentProvider.value = providers.find(p => p.id === id) || null
  configApiKey.value = ''
  configEndpoint.value = currentProvider.value?.api || ''
  showConfigDialog.value = true
}
```

- [ ] **Step 3: Update template placeholder**

Change:
```html
:placeholder="currentProvider?.defaultEndpoint"
```
To:
```html
:placeholder="currentProvider?.api"
```

- [ ] **Step 4: Build and verify**

Run: `pnpm build`
Expected: Build succeeds

- [ ] **Step 5: Commit**

```bash
git add src/views/AiChat/ModelSettings.vue
git commit -m "feat: use dynamic provider list from models.dev"
```

---

## Task 5: Update Model Selection in Chat Header

**Files:**
- Modify: `src/views/AiChat/index.vue`

- [ ] **Step 1: Update currentModelName computed**

The current implementation uses `store.availableModels.find(m => m.id === store.selectedModelId)`. Update to use `store.getModelsForProvider()`.

```typescript
const currentModelName = computed(() => {
  const models = store.getModelsForProvider(store.selectedProvider)
  const model = models.find(m => m.id === store.selectedModelId)
  return model?.name || store.selectedModelId || 'SELECT MODEL'
})
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AiChat/index.vue
git commit -m "feat: update model selection to use models.dev data"
```

---

## Self-Review Checklist

1. **Spec coverage:**
   - [x] models.dev/api.json fetch - Task 2
   - [x] Dynamic provider list - Task 4
   - [x] Remove temperature/maxTokens/stream - Task 3
   - [x] No hardcoded models - All tasks

2. **Placeholder scan:** No TODOs, all code concrete

3. **Type consistency:** `ProviderInfo`, `ModelInfo` defined in Task 1, used in Task 2 and 4

---

## Execution Options

**Plan complete and saved to `docs/superpowers/plans/2026-04-06-models-dev-integration.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
