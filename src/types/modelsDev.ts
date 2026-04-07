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

export interface ConfiguredProvider {
  provider: string
  has_key: boolean
}
