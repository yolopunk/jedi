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
