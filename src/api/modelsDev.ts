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
