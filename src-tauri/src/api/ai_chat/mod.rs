// AI Chat API 模块
// Phase 1: 安全基础设施
// Phase 2: 模型提供商抽象层和会话管理

pub(crate) mod models;
pub(crate) mod sessions;
pub(crate) mod security;

// 重新导出安全相关类型和 commands
pub use security::{
  AuditLoggerState,
  KeyringManagerState,
  delete_api_key,
  encode_html_entities,
  get_api_key_info,
  has_api_key,
  list_api_key_providers,
  log_security_event,
  query_security_logs,
  sanitize,
  store_api_key,
  validate_input,
  validate_key,
  validate_message,
  validate_url,
};

// 重新导出模型相关类型和 commands
pub use models::{
  ModelProviderManagerState,
  send_chat_message,
  send_chat_message_stream,
};

// 重新导出会话相关类型和 commands
pub use sessions::{
  ChatSessionManagerState,
  append_message,
  create_session,
  delete_session,
  get_session,
  list_sessions,
};
