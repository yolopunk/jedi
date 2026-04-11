// AI Chat API 模块
// Phase 1: 安全基础设施
// Phase 2: 模型提供商抽象层和会话管理
// Phase 3: Models.dev 集成

pub(crate) mod models;
pub(crate) mod security;
pub(crate) mod sessions;
pub(crate) mod models_dev;

// 重新导出安全相关类型和 commands
pub use security::{
  delete_api_key, encode_html_entities, get_api_key_info, has_api_key, list_api_key_providers,
  log_security_event, query_security_logs, sanitize, store_api_key, validate_input, validate_key,
  validate_message, validate_url, AuditLoggerState, KeyringManagerState,
};

// 重新导出模型相关类型和 commands
pub use models::{send_chat_message, send_chat_message_stream, ModelProviderManagerState};

// 重新导出会话相关类型和 commands
pub use sessions::{
  append_message, create_session, delete_session, get_session, list_sessions,
  ChatSessionManagerState,
};

// 重新导出 models.dev 相关类型和 commands
pub use models_dev::{
  fetch_models_dev, get_models_dev_provider, get_models_for_provider,
  get_models_providers, search_models_dev, ModelsDevManagerState,
};
