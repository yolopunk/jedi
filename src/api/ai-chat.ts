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
  metadata?: any;
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
  metadata?: any;
}

/**
 * 记录安全事件
 */
export async function logSecurityEvent(request: LogSecurityEventRequest) {
  return await invoke('log_security_event', { request });
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
export async function storeApiKey(request: StoreApiKeyRequest) {
  return await invoke('store_api_key', { request });
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
export async function deleteApiKey(provider: string) {
  return await invoke('delete_api_key', { provider });
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
