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

// ========== Agent / 统一工具调用 API ==========

/** 工具风险等级 */
export type RiskLevel = 'read' | 'write' | 'system'

/** 工具来源 */
export type ToolSource =
  | { kind: 'native' }
  | { kind: 'mcp'; server_id: string; remote_name: string }

/** 工具声明（后端 ToolDeclaration） */
export interface ToolDeclaration {
  name: string
  description: string
  input_schema: Record<string, unknown>
  risk: RiskLevel
  source: ToolSource
  group: string
}

/** 工具执行结果（后端 ToolOutcome） */
export interface ToolOutcome {
  content: string
  is_error: boolean
  undo_token?: string | null
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
  | { type: 'notice'; text: string }
  | { type: 'content_delta'; text: string }
  | { type: 'tool_call'; id: string; server: string; name: string; arguments: unknown }
  | {
      type: 'confirm_request'
      call_id: string
      server: string
      name: string
      risk: RiskLevel
      arguments: unknown
      diff: string
    }
  | { type: 'tool_result'; id: string; name: string; content: string; is_error: boolean; undo_token?: string | null }
  | { type: 'content'; text: string }
  | { type: 'done' }
  | { type: 'error'; message: string }

/**
 * 列出全部已注册工具（供工具浏览器）
 */
export async function toolListAll() {
  return await invoke<ToolDeclaration[]>('tool_list_all')
}

/**
 * 直接调用某个工具（手动/调试）
 */
export async function toolCall(name: string, args?: Record<string, unknown>) {
  return await invoke<ToolOutcome>('tool_call', { name, args: args ?? null })
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
  confirmMode?: 'normal' | 'auto'
  autoApprove?: string[]
  /** 所选模型是否支持工具调用；false 时后端降级为纯对话 */
  supportsTools?: boolean
  /** 单轮注入工具数上限；超出时后端按相关性取 top-K */
  maxTools?: number
}) {
  return await invoke<string>('agent_chat', {
    provider: params.provider,
    model: params.model,
    messages: params.messages,
    servers: params.servers,
    temperature: params.temperature ?? null,
    maxTokens: params.maxTokens ?? null,
    requestId: params.requestId,
    confirmMode: params.confirmMode ?? null,
    autoApprove: params.autoApprove ?? null,
    supportsTools: params.supportsTools ?? null,
    maxTools: params.maxTools ?? null,
  })
}

/**
 * 对某个挂起的工具调用做出确认
 */
export async function toolConfirm(
  requestId: string,
  callId: string,
  approve: boolean,
  editedArgs?: Record<string, unknown>
) {
  return await invoke<void>('tool_confirm', {
    requestId,
    callId,
    approve,
    editedArgs: editedArgs ?? null,
  })
}

/** 取消整个 Agent 回路 */
export async function agentCancel(requestId: string) {
  return await invoke<void>('agent_cancel', { requestId })
}

/** 整回合逆序回滚 */
export async function turnUndo(requestId: string) {
  return await invoke<string[]>('turn_undo', { requestId })
}

/** 单步回滚指定 undo_token */
export async function toolUndo(requestId: string, undoToken: string) {
  return await invoke<string>('tool_undo', { requestId, undoToken })
}

// ========== 第三方 MCP server（P3）==========

/** 第三方 MCP server 配置 */
export interface McpServerConfig {
  id: string
  name: string
  transport?: 'stdio' | 'sse' | 'streamable-http'
  /** stdio */
  command?: string
  args?: string[]
  env?: Record<string, string>
  /** sse */
  url?: string
  headers?: Record<string, string>
  /** 前端本地状态：是否期望连接（后端忽略此字段） */
  enabled?: boolean
}

/** MCP server 连接状态 */
export interface McpServerStatus {
  id: string
  name: string
  connected: boolean
  tool_count: number
  tools: string[]
}

/** 连接一个第三方 MCP server，注入其工具 */
export async function mcpConnect(config: McpServerConfig) {
  return await invoke<McpServerStatus>('mcp_connect', { config })
}

/** 断开某个 MCP server */
export async function mcpDisconnect(serverId: string) {
  return await invoke<void>('mcp_disconnect', { serverId })
}

/** 列出已连接的 MCP server */
export async function mcpListConnected() {
  return await invoke<McpServerStatus[]>('mcp_list_connected')
}

/** 测试并连接一个 MCP server */
export async function mcpServerTest(config: McpServerConfig) {
  return await invoke<McpServerStatus>('mcp_server_test', { config })
}
