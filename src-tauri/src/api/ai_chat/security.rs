// AI Chat 安全审计日志 API
// Phase 1: 安全审计日志 Tauri commands

use crate::utils::security::{
  AuditLogFilter, AuditLogger, OperationResult, SecurityEvent, SecurityEventType,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

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

  let logger = state.logger.lock().map_err(|e| format!("Lock error: {}", e))?;
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

  let logger = state.logger.lock().map_err(|e| format!("Lock error: {}", e))?;
  let events = logger.query_events(filter)?;

  Ok(events.into_iter().map(SecurityEventResponse::from).collect())
}
