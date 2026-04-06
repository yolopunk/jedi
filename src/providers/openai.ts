// src/providers/openai.ts

import type { ProviderAdapter, ProviderConfig, Model, ChatCompletionRequest, ChatCompletionResponse } from './types'
import { invoke } from '@tauri-apps/api/core'

const defaultModels: Model[] = [
  { id: 'gpt-4o', name: 'GPT-4o', provider: 'openai', contextLength: 128000 },
  { id: 'gpt-4o-mini', name: 'GPT-4o Mini', provider: 'openai', contextLength: 128000 },
  { id: 'gpt-4-turbo', name: 'GPT-4 Turbo', provider: 'openai', contextLength: 128000 },
  { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo', provider: 'openai', contextLength: 16384 },
]

export const openaiAdapter: ProviderAdapter = {
  id: 'openai',
  name: 'OpenAI',
  defaultBaseUrl: 'https://api.openai.com/v1',
  models: defaultModels,

  async validateConfig(config: ProviderConfig): Promise<boolean> {
    try {
      await invoke('validate_api_key', { provider: 'openai', key: config.apiKey, endpoint: config.baseUrl || null })
      return true
    } catch {
      return false
    }
  },

  async listModels(config: ProviderConfig): Promise<Model[]> {
    try {
      const models = await invoke<any[]>('list_provider_models', { provider: 'openai', endpoint: config.baseUrl || null })
      return models.map(m => ({ id: m.id, name: m.name || m.id, provider: 'openai', contextLength: m.context_length }))
    } catch {
      return defaultModels
    }
  },

  async createChatCompletion(_config: ProviderConfig, request: ChatCompletionRequest): Promise<ChatCompletionResponse> {
    const result = await invoke<any>('chat_completion', {
      provider: 'openai',
      model: request.model,
      messages: request.messages,
      temperature: request.temperature ?? 0.7,
      maxTokens: request.maxTokens ?? 4096
    })
    return {
      content: result.content,
      model: result.model,
      usage: result.usage ? {
        promptTokens: result.usage.prompt_tokens,
        completionTokens: result.usage.completion_tokens,
        totalTokens: result.usage.total_tokens
      } : undefined
    }
  }
}
