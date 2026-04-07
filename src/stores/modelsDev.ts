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
