// src/stores/providers.ts

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { providerRegistry } from '@/providers/registry'
import type { ProviderConfig, Model } from '@/providers/types'
import { invoke } from '@tauri-apps/api/core'

export const useProvidersStore = defineStore('providers', () => {
  const configs = ref<ProviderConfig[]>([])
  const selectedModelId = ref<string>('')
  const selectedProviderId = ref<string>('')
  const loading = ref(false)
  const error = ref<string | null>(null)

  const configuredProviders = computed(() =>
    configs.value.filter(c => c.enabled && c.apiKey)
  )

  const availableModels = computed((): Model[] => {
    const models: Model[] = []
    for (const config of configuredProviders.value) {
      const adapter = providerRegistry.getAdapter(config.id)
      if (adapter) {
        models.push(...adapter.models)
      }
    }
    return models
  })

  const selectedModel = computed(() =>
    availableModels.value.find(m => m.id === selectedModelId.value) || null
  )

  async function loadProviders(): Promise<void> {
    loading.value = true
    try {
      const providers = await invoke<any[]>('list_api_key_providers')
      configs.value = providers.map(p => ({
        id: p.provider,
        name: providerRegistry.getAdapter(p.provider)?.name || p.provider,
        apiKey: p.has_key ? '***' : '',
        enabled: p.has_key
      }))
      // Sync to registry
      configs.value.forEach(c => providerRegistry.setConfig(c))
      error.value = null
    } catch (e: any) {
      error.value = e.toString()
    } finally {
      loading.value = false
    }
  }

  async function saveApiKey(providerId: string, key: string, endpoint?: string): Promise<void> {
    try {
      await invoke('store_api_key', {
        config: { provider: providerId, key, endpoint: endpoint || null }
      })
      await loadProviders()
    } catch (e: any) {
      error.value = e.toString()
      throw e
    }
  }

  async function deleteApiKey(providerId: string): Promise<void> {
    try {
      await invoke('delete_api_key', { provider: providerId })
      await loadProviders()
    } catch (e: any) {
      error.value = e.toString()
      throw e
    }
  }

  function selectModel(modelId: string): void {
    selectedModelId.value = modelId
    const model = availableModels.value.find(m => m.id === modelId)
    if (model) {
      selectedProviderId.value = model.provider
    }
  }

  return {
    configs, selectedModelId, selectedProviderId, loading, error,
    configuredProviders, availableModels, selectedModel,
    loadProviders, saveApiKey, deleteApiKey, selectModel
  }
})
