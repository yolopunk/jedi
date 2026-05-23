// AI Chat 安全审计日志 API
// Phase 1: 安全审计日志 Tauri commands
// Phase 1: API Key SQLite Tauri commands
// Phase 1: 输入验证和输出编码 Tauri commands

use crate::utils::security::{
  encode_html, sanitize_html, validate_api_key, validate_chat_message, validate_endpoint,
  validate_user_input, ApiKey, ApiKeyValidation, AuditLogFilter, AuditLogger,
  ChatMessageValidation, KeyringManager, ModelProvider, OperationResult, SecurityEvent,
  SecurityEventType, ValidationResult,
};
use chrono::{DateTime, Utc};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

/// 审计日志记录器状态
pub struct AuditLoggerState {
  logger: Mutex<AuditLogger>,
}

impl AuditLoggerState {
  pub fn new() -> Result<Self, String> {
    let logger = AuditLogger::new()?;
    Ok(Self {
      logger: Mutex::new(logger),
    })
  }
}

/// API Key 存储管理器状态
pub struct KeyringManagerState {
  manager: Mutex<KeyringManager>,
  /// has_api_key 结果缓存，避免频繁访问 SQLite
  has_key_cache: Mutex<HashMap<ModelProvider, bool>>,
}

impl KeyringManagerState {
  pub fn new() -> Result<Self, String> {
    let manager = KeyringManager::new()?;
    Ok(Self {
      manager: Mutex::new(manager),
      has_key_cache: Mutex::new(HashMap::new()),
    })
  }

  pub fn get_manager(&self) -> KeyringManager {
    self.manager.lock().unwrap().clone()
  }

  /// 检查 API Key 是否存在（带缓存）
  pub fn has_api_key_cached(&self, provider: ModelProvider) -> Result<bool, String> {
    // 先查缓存
    {
      let cache = self
        .has_key_cache
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
      if let Some(&cached) = cache.get(&provider) {
        return Ok(cached);
      }
    }

    // 缓存未命中，查询 SQLite
    let manager = self
      .manager
      .lock()
      .map_err(|e| format!("Lock error: {}", e))?;
    let result = manager.has_api_key(provider.clone());

    // 写入缓存
    if let Ok(val) = result {
      if let Ok(mut cache) = self
        .has_key_cache
        .lock()
        .map_err(|e| format!("Lock error: {}", e))
      {
        cache.insert(provider, val);
      }
    }

    result
  }

  /// 使缓存失效（当 API Key 被存储或删除时调用）
  fn invalidate_cache(&self) {
    if let Ok(mut cache) = self.has_key_cache.lock() {
      cache.clear();
    }
  }
}

/// 记录安全事件的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSecurityEventRequest {
  /// 事件类型
  pub event_type: String,
  /// 操作结果
  pub result: String,
  /// 用户 ID
  pub user_id: Option<String>,
  /// 资源
  pub resource: Option<String>,
  /// 操作
  pub action: Option<String>,
  /// IP 地址
  pub ip_address: Option<String>,
  /// 用户代理
  pub user_agent: Option<String>,
  /// 额外信息
  pub metadata: Option<serde_json::Value>,
}

/// 查询安全日志的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySecurityLogsRequest {
  /// 起始时间（RFC3339 格式）
  pub start_time: Option<String>,
  /// 结束时间（RFC3339 格式）
  pub end_time: Option<String>,
  /// 事件类型
  pub event_type: Option<String>,
  /// 用户 ID
  pub user_id: Option<String>,
  /// 资源
  pub resource: Option<String>,
  /// 操作结果
  pub result: Option<String>,
  /// 最大返回数量
  pub limit: Option<usize>,
}

/// 安全事件响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEventResponse {
  /// 时间戳
  pub timestamp: String,
  /// 事件类型
  pub event_type: String,
  /// 用户 ID
  pub user_id: Option<String>,
  /// 资源
  pub resource: Option<String>,
  /// 操作
  pub action: Option<String>,
  /// 结果
  pub result: String,
  /// IP 地址
  pub ip_address: Option<String>,
  /// 用户代理
  pub user_agent: Option<String>,
  /// 额外信息
  pub metadata: Option<serde_json::Value>,
}

impl From<SecurityEvent> for SecurityEventResponse {
  fn from(event: SecurityEvent) -> Self {
    Self {
      timestamp: event.timestamp.to_rfc3339(),
      event_type: format!("{:?}", event.event_type),
      user_id: event.user_id,
      resource: event.resource,
      action: event.action,
      result: format!("{:?}", event.result),
      ip_address: event.ip_address,
      user_agent: event.user_agent,
      metadata: event.metadata,
    }
  }
}

/// 解析事件类型字符串
fn parse_event_type(s: &str) -> Result<SecurityEventType, String> {
  match s.to_lowercase().as_str() {
    "authentication" => Ok(SecurityEventType::Authentication),
    "authorization" => Ok(SecurityEventType::Authorization),
    "dataaccess" | "data_access" => Ok(SecurityEventType::DataAccess),
    "configchange" | "config_change" => Ok(SecurityEventType::ConfigChange),
    "apicall" | "api_call" => Ok(SecurityEventType::ApiCall),
    "systemerror" | "system_error" => Ok(SecurityEventType::SystemError),
    "other" => Ok(SecurityEventType::Other),
    _ => Err(format!("Invalid event type: {}", s)),
  }
}

/// 解析操作结果字符串
fn parse_operation_result(s: &str) -> Result<OperationResult, String> {
  match s.to_lowercase().as_str() {
    "success" => Ok(OperationResult::Success),
    "failure" => Ok(OperationResult::Failure),
    "denied" => Ok(OperationResult::Denied),
    _ => Err(format!("Invalid operation result: {}", s)),
  }
}

/// Tauri command: 记录安全事件
#[tauri::command]
pub async fn log_security_event(
  state: State<'_, AuditLoggerState>,
  request: LogSecurityEventRequest,
) -> Result<(), String> {
  let event_type = parse_event_type(&request.event_type)?;
  let result = parse_operation_result(&request.result)?;

  let mut event = SecurityEvent::new(event_type, result);

  if let Some(user_id) = request.user_id {
    event = event.with_user_id(user_id);
  }
  if let Some(resource) = request.resource {
    event = event.with_resource(resource);
  }
  if let Some(action) = request.action {
    event = event.with_action(action);
  }
  if let Some(ip_address) = request.ip_address {
    event = event.with_ip_address(ip_address);
  }
  if let Some(user_agent) = request.user_agent {
    event = event.with_user_agent(user_agent);
  }
  if let Some(metadata) = request.metadata {
    event = event.with_metadata(metadata);
  }

  let logger = state
    .logger
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  logger.log_event(event)
}

/// Tauri command: 查询安全日志
#[tauri::command]
pub async fn query_security_logs(
  state: State<'_, AuditLoggerState>,
  request: QuerySecurityLogsRequest,
) -> Result<Vec<SecurityEventResponse>, String> {
  let mut filter = AuditLogFilter::default();

  // 解析时间
  if let Some(start_str) = request.start_time {
    filter.start_time = Some(
      DateTime::parse_from_rfc3339(&start_str)
        .map_err(|e| format!("Invalid start_time: {}", e))?
        .with_timezone(&Utc),
    );
  }
  if let Some(end_str) = request.end_time {
    filter.end_time = Some(
      DateTime::parse_from_rfc3339(&end_str)
        .map_err(|e| format!("Invalid end_time: {}", e))?
        .with_timezone(&Utc),
    );
  }

  // 解析事件类型
  if let Some(event_type_str) = request.event_type {
    filter.event_type = Some(parse_event_type(&event_type_str)?);
  }

  filter.user_id = request.user_id;
  filter.resource = request.resource;

  // 解析操作结果
  if let Some(result_str) = request.result {
    filter.result = Some(parse_operation_result(&result_str)?);
  }

  filter.limit = request.limit;

  let logger = state
    .logger
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  let events = logger.query_events(filter)?;

  Ok(
    events
      .into_iter()
      .map(SecurityEventResponse::from)
      .collect(),
  )
}

// ========== API Key 管理相关 ==========

/// 存储 API Key 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreApiKeyRequest {
  /// 提供商
  pub provider: String,
  /// API Key
  pub key: String,
  /// API 端点（可选，用于自定义提供商）
  pub endpoint: Option<String>,
}

/// API Key 信息响应（不包含实际 Key，仅用于显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfoResponse {
  /// 提供商
  pub provider: String,
  /// 脱敏显示的 Key
  pub masked_key: String,
  /// API 端点
  pub endpoint: Option<String>,
}

/// API Key 响应（包含实际 Key，供前端 AI SDK 使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
  /// 提供商
  pub provider: String,
  /// 实际 API Key
  pub key: String,
  /// API 端点
  pub endpoint: Option<String>,
}

/// 提供商信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfoResponse {
  /// 提供商
  pub provider: String,
  /// 是否已配置 API Key
  pub has_key: bool,
}

/// 解析提供商字符串
fn parse_provider(s: &str) -> Result<ModelProvider, String> {
  match s.to_lowercase().as_str() {
    "openai" | "open_ai" => Ok(ModelProvider::OpenAi),
    "anthropic" => Ok(ModelProvider::Anthropic),
    "google" => Ok(ModelProvider::Google),
    "ollama" => Ok(ModelProvider::Ollama),
    "cohere" => Ok(ModelProvider::Cohere),
    "deepseek" => Ok(ModelProvider::DeepSeek),
    "moonshot" => Ok(ModelProvider::Moonshot),
    "zhipuai" | "zhipu" => Ok(ModelProvider::ZhipuAI),
    "minimax" => Ok(ModelProvider::MiniMax),
    s if s.starts_with("custom:") => {
      let name = s.strip_prefix("custom:").unwrap_or_default().to_string();
      if name.is_empty() {
        Err("Custom provider name cannot be empty".to_string())
      } else {
        Ok(ModelProvider::Custom(name))
      }
    }
    _ => Err(format!("Invalid provider: {}", s)),
  }
}

/// Tauri command: 存储 API Key
#[tauri::command]
pub async fn store_api_key(
  state: State<'_, KeyringManagerState>,
  request: StoreApiKeyRequest,
) -> Result<(), String> {
  let provider = parse_provider(&request.provider)?;
  let api_key = ApiKey::new(provider, request.key, request.endpoint);

  let manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  let result = manager.store_api_key(api_key);

  // 存储成功后使缓存失效
  if result.is_ok() {
    drop(manager);
    state.invalidate_cache();
  }

  result
}

/// Tauri command: 获取 API Key 信息（不返回实际 Key）
#[tauri::command]
pub async fn get_api_key_info(
  state: State<'_, KeyringManagerState>,
  provider: String,
) -> Result<Option<ApiKeyInfoResponse>, String> {
  let provider_enum = parse_provider(&provider)?;

  let manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  let api_key = manager.get_api_key(provider_enum)?;

  Ok(api_key.map(|key| ApiKeyInfoResponse {
    provider: key.provider().to_string(),
    masked_key: key.mask_key(),
    endpoint: key.endpoint().map(|s| s.to_string()),
  }))
}

/// Tauri command: 获取 API Key（返回实际 key，供前端 AI SDK 使用）
#[tauri::command]
pub async fn get_api_key(
  state: State<'_, KeyringManagerState>,
  provider: String,
) -> Result<Option<ApiKeyResponse>, String> {
  let provider_enum = parse_provider(&provider)?;

  let manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  let api_key = manager.get_api_key(provider_enum)?;

  Ok(api_key.map(|key| ApiKeyResponse {
    provider: key.provider().to_string(),
    key: key.key().expose_secret().to_string(),
    endpoint: key.endpoint().map(|s| s.to_string()),
  }))
}

/// Tauri command: 删除 API Key
#[tauri::command]
pub async fn delete_api_key(
  state: State<'_, KeyringManagerState>,
  provider: String,
) -> Result<(), String> {
  let provider_enum = parse_provider(&provider)?;

  let manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  let result = manager.delete_api_key(provider_enum);

  // 删除成功后使缓存失效
  if result.is_ok() {
    drop(manager);
    state.invalidate_cache();
  }

  result
}

/// Tauri command: 检查 API Key 是否存在（使用缓存减少 SQLite 调用）
#[tauri::command]
pub async fn has_api_key(
  state: State<'_, KeyringManagerState>,
  provider: String,
) -> Result<bool, String> {
  let provider_enum = parse_provider(&provider)?;
  state.has_api_key_cached(provider_enum)
}

/// Tauri command: 列出所有已配置的提供商
#[tauri::command]
pub async fn list_api_key_providers(
  state: State<'_, KeyringManagerState>,
) -> Result<Vec<ProviderInfoResponse>, String> {
  let manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  let providers = manager.list_providers()?;

  Ok(
    providers
      .into_iter()
      .map(|provider| ProviderInfoResponse {
        provider: provider.to_string(),
        has_key: true,
      })
      .collect(),
  )
}

// ========== 输入验证和输出编码相关 ==========

/// 验证用户输入的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateUserInputRequest {
  pub input: String,
  pub max_length: Option<usize>,
}

/// 验证 API Key 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateApiKeyRequest {
  pub key: String,
  pub provider: Option<String>,
}

/// 验证聊天消息的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateChatMessageRequest {
  pub content: String,
  pub is_user: Option<bool>,
}

/// 清理 HTML 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizeHtmlRequest {
  pub html: String,
}

/// Tauri command: 验证用户输入
#[tauri::command]
pub async fn validate_input(request: ValidateUserInputRequest) -> Result<ValidationResult, String> {
  let opts = request
    .max_length
    .map(|len| crate::utils::security::ValidationOptions {
      max_length: len,
      ..Default::default()
    });
  Ok(validate_user_input(&request.input, opts.as_ref()))
}

/// Tauri command: 验证 API Key
#[tauri::command]
pub async fn validate_key(request: ValidateApiKeyRequest) -> Result<ApiKeyValidation, String> {
  Ok(validate_api_key(
    &request.key,
    request.provider.as_deref(),
    None,
  ))
}

/// Tauri command: 验证端点 URL
#[tauri::command]
pub async fn validate_url(url: String) -> Result<ValidationResult, String> {
  Ok(validate_endpoint(&url, None))
}

/// Tauri command: 验证聊天消息
#[tauri::command]
pub async fn validate_message(
  request: ValidateChatMessageRequest,
) -> Result<ChatMessageValidation, String> {
  Ok(validate_chat_message(
    &request.content,
    request.is_user.unwrap_or(true),
  ))
}

/// Tauri command: 清理 HTML
#[tauri::command]
pub async fn sanitize(request: SanitizeHtmlRequest) -> Result<String, String> {
  Ok(sanitize_html(&request.html))
}

/// Tauri command: HTML 实体编码
#[tauri::command]
pub async fn encode_html_entities(text: String) -> Result<String, String> {
  Ok(encode_html(&text))
}
