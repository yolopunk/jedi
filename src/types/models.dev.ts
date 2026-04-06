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
