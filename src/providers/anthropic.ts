// src/providers/anthropic.ts

import type { ProviderAdapter, ProviderConfig, Model, ChatCompletionRequest, ChatCompletionResponse } from './types'
import { invoke } from '@tauri-apps/api/core'

const defaultModels: Model[] = [
  { id: 'claude-sonnet-4-20250514', name: 'Claude Sonnet 4', provider: 'anthropic', contextLength: 200000 },
  { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet', provider: 'anthropic', contextLength: 200000 },
  { id: 'claude-3-opus-20240229', name: 'Claude 3 Opus', provider: 'anthropic', contextLength: 200000 },
]

export const anthropicAdapter: ProviderAdapter = {
  id: 'anthropic',
  name: 'Anthropic',
  defaultBaseUrl: 'https://api.anthropic.com/v1',
  models: defaultModels,

  async validateConfig(config: ProviderConfig): Promise<boolean> {
    try {
      await invoke('validate_api_key', { provider: 'anthropic', key: config.apiKey, endpoint: config.baseUrl || null })
      return true
    } catch {
      return false
    }
  },

  async listModels(_config: ProviderConfig): Promise<Model[]> {
    return defaultModels
  },

  async createChatCompletion(_config: ProviderConfig, request: ChatCompletionRequest): Promise<ChatCompletionResponse> {
    const result = await invoke<any>('chat_completion', {
      provider: 'anthropic',
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
