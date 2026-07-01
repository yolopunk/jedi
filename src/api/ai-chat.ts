/**
 * AI Chat API
 * 封装与Tauri后端的交互
 */
import { invoke } from '@tauri-apps/api/core';

// ========== 输入验证 API ==========

export interface ValidationResult {
  valid: boolean;
  sanitized: string;
  warnings: string[];
}

export interface ApiKeyValidation {
  valid: boolean;
  warnings: string[];
  provider_hint?: string;
}

export interface MessageMetadata {
  word_count: number;
  char_count: number;
  has_urls: boolean;
  has_code: boolean;
}

export interface ChatMessageValidation {
  valid: boolean;
  content: string;
  warnings: string[];
  metadata: MessageMetadata;
}

export interface ValidateUserInputRequest {
  input: string;
  max_length?: number;
}

export interface ValidateApiKeyRequest {
  key: string;
  provider?: string;
}

export interface ValidateChatMessageRequest {
  content: string;
  is_user?: boolean;
}

/**
 * 验证用户输入
 */
export async function validateInput(request: ValidateUserInputRequest) {
  return await invoke<ValidationResult>('validate_input', { request });
}

/**
 * 验证 API Key
 */
export async function validateKey(request: ValidateApiKeyRequest) {
  return await invoke<ApiKeyValidation>('validate_key', { request });
}

/**
 * 验证端点 URL
 */
export async function validateUrl(url: string) {
  return await invoke<ValidationResult>('validate_url', { url });
}

/**
 * 验证聊天消息
 */
export async function validateMessage(request: ValidateChatMessageRequest) {
  return await invoke<ChatMessageValidation>('validate_message', { request });
}

/**
 * 清理 HTML
 */
export async function sanitizeHtml(html: string) {
  return await invoke<string>('sanitize', { request: { html } });
}

/**
 * HTML 实体编码
 */
export async function encodeHtmlEntities(text: string) {
  return await invoke<string>('encode_html_entities', { text });
}

// ========== 安全审计日志 API ==========

export interface LogSecurityEventRequest {
  event_type: string;
  result: string;
  user_id?: string;
  resource?: string;
  action?: string;
  ip_address?: string;
  user_agent?: string;
  metadata?: Record<string, unknown>;
}

export interface QuerySecurityLogsRequest {
  start_time?: string;
  end_time?: string;
  event_type?: string;
  user_id?: string;
  resource?: string;
  result?: string;
  limit?: number;
}

export interface SecurityEventResponse {
  timestamp: string;
  event_type: string;
  user_id?: string;
  resource?: string;
  action?: string;
  result: string;
  ip_address?: string;
  user_agent?: string;
  metadata?: Record<string, unknown>;
}

/**
 * 记录安全事件
 */
export async function logSecurityEvent(request: LogSecurityEventRequest): Promise<void> {
  return await invoke<void>('log_security_event', { request });
}

/**
 * 查询安全日志
 */
export async function querySecurityLogs(request: QuerySecurityLogsRequest) {
  return await invoke<SecurityEventResponse[]>('query_security_logs', { request });
}

// ========== API Key 管理 API ==========

export interface StoreApiKeyRequest {
  provider: string;
  key: string;
  endpoint?: string;
}

export interface ApiKeyInfoResponse {
  provider: string;
  masked_key: string;
  endpoint?: string;
}

export interface ProviderInfoResponse {
  provider: string;
  has_key: boolean;
}

/**
 * 存储 API Key
 */
export async function storeApiKey(request: StoreApiKeyRequest): Promise<void> {
  return await invoke<void>('store_api_key', { request });
}

/**
 * 获取 API Key 信息（不返回实际 Key）
 */
export async function getApiKeyInfo(provider: string) {
  return await invoke<ApiKeyInfoResponse | null>('get_api_key_info', { provider });
}

/**
 * 删除 API Key
 */
export async function deleteApiKey(provider: string): Promise<void> {
  return await invoke<void>('delete_api_key', { provider });
}

/**
 * 检查 API Key 是否存在
 */
export async function hasApiKey(provider: string) {
  return await invoke<boolean>('has_api_key', { provider });
}

/**
 * 列出所有已配置的提供商
 */
export async function listApiKeyProviders() {
  return await invoke<ProviderInfoResponse[]>('list_api_key_providers');
}

// ========== Models.dev API ==========

export interface ModelsDevModalities {
  input: string[]
  output: string[]
}

export interface ModelsDevModelCost {
  input?: number
  output?: number
  cache_read?: number
  cache_write?: number
}

export interface ModelsDevModelLimits {
  context?: number
  input?: number
  output?: number
}

export interface ModelsDevInterleavedConfig {
  field: string
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
  modalities: ModelsDevModalities
  open_weights: boolean
  cost?: ModelsDevModelCost
  limit?: ModelsDevModelLimits
  interleaved?: ModelsDevInterleavedConfig
}

export interface ModelsDevProvider {
  id: string
  name: string
  api?: string
  doc?: string
  npm?: string
  env?: string[]
  models: Record<string, ModelsDevModel>
}

export interface ProviderSummary {
  id: string
  name: string
  api?: string
  doc?: string
  model_count: number
}

export type ModelsDevResponse = Record<string, ModelsDevProvider>

/**
 * 从 models.dev 获取所有提供商和模型信息
 */
export async function fetchModelsDev(forceRefresh?: boolean) {
  return await invoke<ModelsDevResponse>('fetch_models_dev', { forceRefresh })
}

/**
 * 获取指定提供商的信息
 */
export async function getModelsDevProvider(providerId: string) {
  return await invoke<ModelsDevProvider | null>('get_models_dev_provider', { providerId })
}

/**
 * 搜索模型（按名称或 ID）
 */
export async function searchModelsDev(query: string, providerFilter?: string) {
  return await invoke<Array<[string, ModelsDevModel]>>('search_models_dev', { query, providerFilter })
}

/**
 * 获取某个提供商的模型列表
 */
export async function getModelsForProvider(providerId: string) {
  return await invoke<ModelsDevModel[]>('get_models_for_provider', { providerId })
}

/**
 * 获取所有支持的提供商列表（精简信息）
 */
export async function getModelsProviders() {
  return await invoke<ProviderSummary[]>('get_models_providers')
}

// ========== Agent / MCP 工具调用 API ==========

/** MCP 工具信息（供前端展示） */
export interface ToolInfo {
  server: string
  name: string
  description?: string
  input_schema: Record<string, unknown>
}

/** MCP 工具调用结果 */
export interface CallToolResult {
  content: Array<{ type: string; text?: string; [k: string]: unknown }>
  is_error?: boolean
}

/** 消息格式（后端 Message 结构） */
export interface AgentMessage {
  role: 'system' | 'user' | 'assistant'
  content: string
}

/**
 * Agent 执行过程事件（通过 `agent-event-{requestId}` 事件推送）
 */
export type AgentEvent =
  | { type: 'thinking'; text: string }
  | { type: 'tool_call'; id: string; server: string; name: string; arguments: unknown }
  | { type: 'tool_result'; id: string; name: string; content: string; is_error: boolean }
  | { type: 'content'; text: string }
  | { type: 'done' }
  | { type: 'error'; message: string }

/**
 * 列出已启用 MCP 服务的所有工具
 */
export async function mcpListTools(servers: string[]) {
  return await invoke<ToolInfo[]>('mcp_list_tools', { servers })
}

/**
 * 直接调用某个 MCP 工具（手动/调试）
 */
export async function mcpCallTool(
  server: string,
  name: string,
  args?: Record<string, unknown>
) {
  return await invoke<CallToolResult>('mcp_call_tool', { server, name, arguments: args ?? null })
}

/**
 * Agent 聊天：携带 MCP 工具运行工具调用回路，返回最终回答。
 * 过程事件需通过监听 `agent-event-{requestId}` 获取。
 */
export async function agentChat(params: {
  provider: string
  model: string
  messages: AgentMessage[]
  servers: string[]
  temperature?: number
  maxTokens?: number
  requestId: string
}) {
  return await invoke<string>('agent_chat', {
    provider: params.provider,
    model: params.model,
    messages: params.messages,
    servers: params.servers,
    temperature: params.temperature ?? null,
    maxTokens: params.maxTokens ?? null,
    requestId: params.requestId,
  })
}
