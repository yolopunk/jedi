// 安全工具模块
// Phase 1: 安全基础设施

pub(crate) mod audit_log;
pub(crate) mod keyring;
pub(crate) mod validation;

// 重新导出审计日志相关类型
pub use audit_log::{
  AuditLogFilter, AuditLogger, OperationResult, SecurityEvent, SecurityEventType,
};

// 重新导出 API Key 存储相关类型
pub use keyring::{ApiKey, KeyringManager, ModelProvider};

// 重新导出验证相关类型
pub use validation::{
  encode_html, sanitize_html, validate_api_key, validate_chat_message, validate_endpoint,
  validate_user_input, ApiKeyValidation, ChatMessageValidation, ValidationOptions,
  ValidationResult,
};
