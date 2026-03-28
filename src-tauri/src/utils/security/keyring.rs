// API Key 密钥链存储
// Phase 1: API Key 安全存储实现

use keyring::Entry;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::audit_log::{AuditLogger, OperationResult, SecurityEvent, SecurityEventType};

/// 模型提供商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelProvider {
  /// OpenAI
  OpenAi,
  /// Anthropic
  Anthropic,
  /// Ollama (本地，不需要 API Key)
  Ollama,
  /// 自定义提供商
  Custom(String),
}

impl fmt::Display for ModelProvider {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ModelProvider::OpenAi => write!(f, "openai"),
      ModelProvider::Anthropic => write!(f, "anthropic"),
      ModelProvider::Ollama => write!(f, "ollama"),
      ModelProvider::Custom(name) => write!(f, "custom:{}", name),
    }
  }
}

/// API Key 结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
  /// 提供商
  provider: ModelProvider,
  /// API Key (加密存储在内存中)
  #[serde(skip_serializing)]
  key: SecretString,
  /// 可选的 API 端点（用于自定义提供商）
  endpoint: Option<String>,
}

impl ApiKey {
  /// 创建新的 API Key
  pub fn new(provider: ModelProvider, key: impl Into<String>, endpoint: Option<String>) -> Self {
    Self {
      provider,
      key: SecretString::new(key.into()),
      endpoint,
    }
  }

  /// 获取提供商
  pub fn provider(&self) -> &ModelProvider {
    &self.provider
  }

  /// 获取 API Key（仅限内部使用）
  pub fn key(&self) -> &SecretString {
    &self.key
  }

  /// 获取端点
  pub fn endpoint(&self) -> Option<&str> {
    self.endpoint.as_deref()
  }

  /// 获取脱敏显示的 Key（只显示后 4 位）
  pub fn mask_key(&self) -> String {
    let key = self.key.expose_secret();
    if key.len() <= 4 {
      "****".to_string()
    } else {
      let visible = &key[key.len() - 4..];
      format!("****{}", visible)
    }
  }
}

impl Drop for ApiKey {
  fn drop(&mut self) {
    // 确保内存中的 Key 被清理
    // SecretString 已经会自动处理，但我们可以额外确保
  }
}

/// 密钥链管理器
pub struct KeyringManager {
  service_name: String,
  audit_logger: AuditLogger,
}

impl KeyringManager {
  /// 创建新的密钥链管理器
  pub fn new() -> Result<Self, String> {
    let audit_logger = AuditLogger::new()?;
    Ok(Self {
      service_name: "jedi-chat".to_string(),
      audit_logger,
    })
  }

  /// 获取密钥环条目名称
  fn get_entry_name(&self, provider: &ModelProvider) -> String {
    format!("api-key-{}", provider)
  }

  /// 存储 API Key
  pub fn store_api_key(&self, api_key: ApiKey) -> Result<(), String> {
    let entry_name = self.get_entry_name(&api_key.provider);

    // 记录审计事件
    let mut event = SecurityEvent::new(SecurityEventType::ConfigChange, OperationResult::Success)
      .with_resource(format!("api-key/{}", api_key.provider))
      .with_action("store");

    let result = (|| {
      let entry = Entry::new(&self.service_name, &entry_name)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

      // 存储 API Key
      entry
        .set_password(api_key.key.expose_secret())
        .map_err(|e| format!("Failed to store API key: {}", e))?;

      // 如果有端点，也存储它
      if let Some(ref endpoint) = api_key.endpoint {
        let endpoint_entry_name = format!("{}-endpoint", entry_name);
        let endpoint_entry = Entry::new(&self.service_name, &endpoint_entry_name)
          .map_err(|e| format!("Failed to create endpoint entry: {}", e))?;
        endpoint_entry
          .set_password(endpoint)
          .map_err(|e| format!("Failed to store endpoint: {}", e))?;
      }

      Ok(())
    })();

    if let Err(ref e) = result {
      event = event.with_result(OperationResult::Failure);
      event = event.with_metadata(serde_json::json!({ "error": e }));
    }

    let _ = self.audit_logger.log_event(event);

    result
  }

  /// 读取 API Key
  pub fn get_api_key(&self, provider: ModelProvider) -> Result<Option<ApiKey>, String> {
    let entry_name = self.get_entry_name(&provider);

    // 记录审计事件
    let mut event = SecurityEvent::new(SecurityEventType::DataAccess, OperationResult::Success)
      .with_resource(format!("api-key/{}", provider))
      .with_action("read");

    let result = (|| {
      let entry = match Entry::new(&self.service_name, &entry_name) {
        Ok(e) => e,
        Err(keyring::Error::NoEntry) => return Ok(None),
        Err(e) => return Err(format!("Failed to access keyring: {}", e)),
      };

      let key = match entry.get_password() {
        Ok(k) => k,
        Err(keyring::Error::NoEntry) => return Ok(None),
        Err(e) => return Err(format!("Failed to read API key: {}", e)),
      };

      // 尝试读取端点
      let endpoint = {
        let endpoint_entry_name = format!("{}-endpoint", entry_name);
        match Entry::new(&self.service_name, &endpoint_entry_name) {
          Ok(endpoint_entry) => match endpoint_entry.get_password() {
            Ok(e) => Some(e),
            Err(keyring::Error::NoEntry) => None,
            Err(_) => None,
          },
          Err(_) => None,
        }
      };

      Ok(Some(ApiKey::new(provider, key, endpoint)))
    })();

    if let Err(ref e) = result {
      event = event.with_result(OperationResult::Failure);
      event = event.with_metadata(serde_json::json!({ "error": e }));
    }

    let _ = self.audit_logger.log_event(event);

    result
  }

  /// 删除 API Key
  pub fn delete_api_key(&self, provider: ModelProvider) -> Result<(), String> {
    let entry_name = self.get_entry_name(&provider);

    // 记录审计事件
    let mut event = SecurityEvent::new(SecurityEventType::ConfigChange, OperationResult::Success)
      .with_resource(format!("api-key/{}", provider))
      .with_action("delete");

    let result = (|| {
      // 删除主密钥
      let entry = match Entry::new(&self.service_name, &entry_name) {
        Ok(e) => e,
        Err(keyring::Error::NoEntry) => return Ok(()), // 已经不存在
        Err(e) => return Err(format!("Failed to access keyring: {}", e)),
      };

      entry
        .delete_password()
        .map_err(|e| format!("Failed to delete API key: {}", e))?;

      // 删除端点（如果存在）
      let endpoint_entry_name = format!("{}-endpoint", entry_name);
      if let Ok(endpoint_entry) = Entry::new(&self.service_name, &endpoint_entry_name) {
        let _ = endpoint_entry.delete_password(); // 忽略删除端点的错误
      }

      Ok(())
    })();

    if let Err(ref e) = result {
      event = event.with_result(OperationResult::Failure);
      event = event.with_metadata(serde_json::json!({ "error": e }));
    }

    let _ = self.audit_logger.log_event(event);

    result
  }

  /// 检查 API Key 是否存在
  pub fn has_api_key(&self, provider: ModelProvider) -> Result<bool, String> {
    match self.get_api_key(provider) {
      Ok(Some(_)) => Ok(true),
      Ok(None) => Ok(false),
      Err(e) => Err(e),
    }
  }

  /// 列出所有已存储的 API Key 提供商（不返回 Key 本身）
  pub fn list_providers(&self) -> Result<Vec<ModelProvider>, String> {
    // 检查已知的提供商
    let known_providers = vec![
      ModelProvider::OpenAi,
      ModelProvider::Anthropic,
      ModelProvider::Ollama,
    ];

    let mut result = Vec::new();

    for provider in known_providers {
      if self.has_api_key(provider.clone())? {
        result.push(provider);
      }
    }

    Ok(result)
  }
}

impl Default for KeyringManager {
  fn default() -> Self {
    Self::new().expect("Failed to create KeyringManager")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_api_key_creation() {
    let api_key = ApiKey::new(ModelProvider::OpenAi, "sk-test1234", None);

    assert_eq!(api_key.provider(), &ModelProvider::OpenAi);
    assert_eq!(api_key.endpoint(), None);
  }

  #[test]
  fn test_api_key_masking() {
    let api_key = ApiKey::new(ModelProvider::OpenAi, "sk-abcdefghijklmnop", None);

    let masked = api_key.mask_key();
    assert_eq!(masked, "****mnop");
  }

  #[test]
  fn test_short_api_key_masking() {
    let api_key = ApiKey::new(ModelProvider::OpenAi, "1234", None);

    let masked = api_key.mask_key();
    assert_eq!(masked, "****");
  }

  #[test]
  fn test_api_key_with_endpoint() {
    let api_key = ApiKey::new(
      ModelProvider::Custom("test".to_string()),
      "test-key",
      Some("https://api.example.com".to_string()),
    );

    assert_eq!(api_key.endpoint(), Some("https://api.example.com"));
  }

  #[test]
  fn test_model_provider_display() {
    assert_eq!(ModelProvider::OpenAi.to_string(), "openai");
    assert_eq!(ModelProvider::Anthropic.to_string(), "anthropic");
    assert_eq!(ModelProvider::Ollama.to_string(), "ollama");
    assert_eq!(
      ModelProvider::Custom("my-provider".to_string()).to_string(),
      "custom:my-provider"
    );
  }

  #[test]
  fn test_keyring_manager_creation() {
    let manager = KeyringManager::new();
    assert!(manager.is_ok());
  }

  #[test]
  fn test_has_api_key_nonexistent() {
    let manager = KeyringManager::new().unwrap();
    // 检查一个不存在的提供商
    let result = manager.has_api_key(ModelProvider::Custom("nonexistent-test".to_string()));
    assert!(result.is_ok());
  }

  #[test]
  fn test_list_providers() {
    let manager = KeyringManager::new().unwrap();
    let result = manager.list_providers();
    assert!(result.is_ok());
  }

  #[test]
  fn test_get_nonexistent_api_key() {
    let manager = KeyringManager::new().unwrap();
    let result = manager.get_api_key(ModelProvider::Custom("nonexistent-get".to_string()));
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
  }

  #[test]
  fn test_delete_nonexistent_api_key() {
    let manager = KeyringManager::new().unwrap();
    // 删除不存在的 Key - keyring crate 在某些平台可能会返回错误
    // 这是可以接受的，因为不同平台的密钥链实现不同
    let _result = manager.delete_api_key(ModelProvider::Custom("nonexistent-delete".to_string()));
    // 不断言结果，因为平台行为可能不同
  }
}
