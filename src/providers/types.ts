// src/providers/types.ts

export interface ProviderConfig {
  id: string
  name: string
  apiKey: string
  baseUrl?: string
  enabled: boolean
}

export interface Model {
  id: string
  name: string
  provider: string
  contextLength?: number
}

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant'
  content: string
}

export interface ChatCompletionRequest {
  model: string
  messages: ChatMessage[]
  temperature?: number
  maxTokens?: number
  stream?: boolean
}

export interface ChatCompletionResponse {
  content: string
  model: string
  usage?: {
    promptTokens: number
    completionTokens: number
    totalTokens: number
  }
}

export interface ChatCompletionChunk {
  content: string
  done: boolean
}

export interface ProviderAdapter {
  id: string
  name: string
  defaultBaseUrl: string
  models: Model[]

  validateConfig(config: ProviderConfig): Promise<boolean>
  listModels(config: ProviderConfig): Promise<Model[]>
  createChatCompletion(
    config: ProviderConfig,
    request: ChatCompletionRequest
  ): Promise<ChatCompletionResponse>
}
