import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ApiKeyResponse, ConfiguredProvider } from '@/types/modelsDev'

// The Rust keyring whitelists a fixed provider enum and accepts anything
// else only as `custom:<name>` (parse_provider in security.rs). models.dev
// exposes hundreds of provider ids (arcee, groq, …), so wrap/unwrap here to
// keep the round trip identity-preserving for every consumer.
const RUST_KNOWN_PROVIDERS = new Set([
  'openai',
  'anthropic',
  'google',
  'ollama',
  'cohere',
  'deepseek',
  'moonshot',
  'zhipuai',
  'zhipu',
  'minimax',
])

function toRustProvider(id: string): string {
  if (!id || RUST_KNOWN_PROVIDERS.has(id) || id.startsWith('custom:')) return id
  return `custom:${id}`
}

function fromRustProvider(id: string): string {
  return id.startsWith('custom:') ? id.slice('custom:'.length) : id
}

export const useProviderConfigStore = defineStore('providerConfig', () => {
  const configuredProviders = ref<ConfiguredProvider[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadConfiguredProviders(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const list = await invoke<ConfiguredProvider[]>('list_api_key_providers')
      configuredProviders.value = list.map(p => ({
        ...p,
        provider: fromRustProvider(p.provider),
      }))
    } catch (e: any) {
      error.value = e.toString()
    } finally {
      loading.value = false
    }
  }

  async function saveApiKey(provider: string, key: string, endpoint?: string): Promise<void> {
    await invoke('store_api_key', {
      request: { provider: toRustProvider(provider), key, endpoint: endpoint || null },
    })
    await loadConfiguredProviders()
  }

  async function deleteApiKey(provider: string): Promise<void> {
    await invoke('delete_api_key', { provider: toRustProvider(provider) })
    await loadConfiguredProviders()
  }

  async function hasApiKey(provider: string): Promise<boolean> {
    return await invoke<boolean>('has_api_key', { provider: toRustProvider(provider) })
  }

  async function getApiKey(provider: string): Promise<ApiKeyResponse | null> {
    const raw = await invoke<ApiKeyResponse | null>('get_api_key', {
      provider: toRustProvider(provider),
    })
    return raw ? { ...raw, provider: fromRustProvider(raw.provider) } : null
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
    getApiKey,
    isProviderConfigured,
  }
})
