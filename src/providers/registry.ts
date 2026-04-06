// src/providers/registry.ts

import type { ProviderAdapter, ProviderConfig, Model } from './types'
import { openaiAdapter } from './openai'
import { anthropicAdapter } from './anthropic'

export class ProviderRegistry {
  private adapters: Map<string, ProviderAdapter> = new Map()
  private configs: Map<string, ProviderConfig> = new Map()

  register(adapter: ProviderAdapter): void {
    this.adapters.set(adapter.id, adapter)
  }

  getAdapter(id: string): ProviderAdapter | undefined {
    return this.adapters.get(id)
  }

  listAdapters(): ProviderAdapter[] {
    return Array.from(this.adapters.values())
  }

  setConfig(config: ProviderConfig): void {
    this.configs.set(config.id, config)
  }

  getConfig(id: string): ProviderConfig | undefined {
    return this.configs.get(id)
  }

  listConfigs(): ProviderConfig[] {
    return Array.from(this.configs.values())
  }

  listConfigured(): ProviderConfig[] {
    return this.listConfigs().filter(c => c.enabled && c.apiKey)
  }

  getAllModels(): Model[] {
    const models: Model[] = []
    for (const config of this.listConfigured()) {
      const adapter = this.adapters.get(config.id)
      if (adapter) {
        models.push(...adapter.models)
      }
    }
    return models
  }
}

export const providerRegistry = new ProviderRegistry()

// Register built-in adapters
providerRegistry.register(openaiAdapter)
providerRegistry.register(anthropicAdapter)
