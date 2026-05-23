// API Key SQLite 存储
// 保留 KeyringManager 类型名以兼容现有调用链，底层已改为本地 SQLite。

use rusqlite::{params, Connection, OptionalExtension};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use super::audit_log::{AuditLogger, OperationResult, SecurityEvent, SecurityEventType};

/// 模型提供商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelProvider {
  /// OpenAI
  OpenAi,
  /// Anthropic
  Anthropic,
  /// Google (Gemini)
  Google,
  /// Ollama (本地)
  Ollama,
  /// Cohere
  Cohere,
  /// DeepSeek
  DeepSeek,
  /// Moonshot
  Moonshot,
  /// ZhipuAI
  ZhipuAI,
  /// MiniMax
  MiniMax,
  /// 自定义提供商
  Custom(String),
}

impl fmt::Display for ModelProvider {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ModelProvider::OpenAi => write!(f, "openai"),
      ModelProvider::Anthropic => write!(f, "anthropic"),
      ModelProvider::Google => write!(f, "google"),
      ModelProvider::Ollama => write!(f, "ollama"),
      ModelProvider::Cohere => write!(f, "cohere"),
      ModelProvider::DeepSeek => write!(f, "deepseek"),
      ModelProvider::Moonshot => write!(f, "moonshot"),
      ModelProvider::ZhipuAI => write!(f, "zhipuai"),
      ModelProvider::MiniMax => write!(f, "minimax"),
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

/// API Key SQLite 管理器
pub struct KeyringManager {
  db_path: PathBuf,
  audit_logger: AuditLogger,
  /// 内存缓存：避免频繁访问 SQLite
  cache: Mutex<HashMap<String, Option<ApiKey>>>,
}

impl Clone for KeyringManager {
  fn clone(&self) -> Self {
    Self {
      db_path: self.db_path.clone(),
      audit_logger: self.audit_logger.clone(),
      cache: Mutex::new(HashMap::new()),
    }
  }
}

impl KeyringManager {
  /// 创建新的 API Key SQLite 管理器
  pub fn new() -> Result<Self, String> {
    let db_path = Self::default_db_path()?;
    Self::new_with_path(db_path)
  }

  /// 使用指定路径创建管理器，主要用于测试
  pub fn new_with_path(db_path: impl Into<PathBuf>) -> Result<Self, String> {
    let db_path = db_path.into();
    if let Some(parent) = db_path.parent() {
      fs::create_dir_all(parent).map_err(|e| format!("Failed to create API key db dir: {}", e))?;
    }

    let audit_logger = AuditLogger::new()?;
    let manager = Self {
      db_path,
      audit_logger,
      cache: Mutex::new(HashMap::new()),
    };
    manager.init_db()?;
    Ok(manager)
  }

  fn default_db_path() -> Result<PathBuf, String> {
    let base_dir =
      dirs::data_dir().ok_or_else(|| "Failed to resolve user data directory".to_string())?;
    Ok(base_dir.join("Jedi").join("ai_chat.sqlite3"))
  }

  fn connection(&self) -> Result<Connection, String> {
    Connection::open(&self.db_path).map_err(|e| format!("Failed to open API key database: {}", e))
  }

  fn init_db(&self) -> Result<(), String> {
    let conn = self.connection()?;
    conn
      .execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS api_keys (
          provider TEXT PRIMARY KEY NOT NULL,
          api_key TEXT NOT NULL,
          endpoint TEXT,
          created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
      )
      .map_err(|e| format!("Failed to initialize API key database: {}", e))?;
    self.harden_db_permissions();
    Ok(())
  }

  #[cfg(unix)]
  fn harden_db_permissions(&self) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(&self.db_path) {
      let mut permissions = metadata.permissions();
      permissions.set_mode(0o600);
      let _ = fs::set_permissions(&self.db_path, permissions);
    }
  }

  #[cfg(not(unix))]
  fn harden_db_permissions(&self) {}

  fn provider_key(provider: &ModelProvider) -> String {
    provider.to_string()
  }

  fn provider_from_key(value: &str) -> ModelProvider {
    match value {
      "openai" => ModelProvider::OpenAi,
      "anthropic" => ModelProvider::Anthropic,
      "google" => ModelProvider::Google,
      "ollama" => ModelProvider::Ollama,
      "cohere" => ModelProvider::Cohere,
      "deepseek" => ModelProvider::DeepSeek,
      "moonshot" => ModelProvider::Moonshot,
      "zhipuai" => ModelProvider::ZhipuAI,
      "minimax" => ModelProvider::MiniMax,
      custom => ModelProvider::Custom(custom.strip_prefix("custom:").unwrap_or(custom).to_string()),
    }
  }

  /// 存储 API Key
  pub fn store_api_key(&self, api_key: ApiKey) -> Result<(), String> {
    let provider_key = Self::provider_key(&api_key.provider);

    // 记录审计事件
    let mut event = SecurityEvent::new(SecurityEventType::ConfigChange, OperationResult::Success)
      .with_resource(format!("api-key/{}", api_key.provider))
      .with_action("store");

    let result = (|| {
      let conn = self.connection()?;
      let endpoint = api_key.endpoint.as_deref();
      conn
        .execute(
          r#"
          INSERT INTO api_keys (provider, api_key, endpoint, created_at, updated_at)
          VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
          ON CONFLICT(provider) DO UPDATE SET
            api_key = excluded.api_key,
            endpoint = excluded.endpoint,
            updated_at = CURRENT_TIMESTAMP
          "#,
          params![provider_key.as_str(), api_key.key.expose_secret(), endpoint],
        )
        .map_err(|e| format!("Failed to store API key in SQLite: {}", e))?;

      Ok(())
    })();

    if let Err(ref e) = result {
      event = event.with_result(OperationResult::Failure);
      event = event.with_metadata(serde_json::json!({ "error": e }));
    }

    let _ = self.audit_logger.log_event(event);

    // 写入成功后更新缓存
    if result.is_ok() {
      if let Ok(mut cache) = self.cache.lock().map_err(|e| format!("Lock error: {}", e)) {
        cache.insert(api_key.provider.to_string(), Some(api_key));
      }
    }

    result
  }

  /// 读取 API Key（带缓存）
  pub fn get_api_key(&self, provider: ModelProvider) -> Result<Option<ApiKey>, String> {
    let cache_key = Self::provider_key(&provider);

    // 先查缓存
    {
      let cache = self
        .cache
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
      if let Some(cached) = cache.get(&cache_key) {
        return Ok(cached.clone());
      }
    }

    // 记录审计事件
    let mut event = SecurityEvent::new(SecurityEventType::DataAccess, OperationResult::Success)
      .with_resource(format!("api-key/{}", provider))
      .with_action("read");

    let result = (|| {
      let conn = self.connection()?;
      let row = conn
        .query_row(
          "SELECT api_key, endpoint FROM api_keys WHERE provider = ?1",
          params![cache_key.as_str()],
          |row| {
            let key: String = row.get(0)?;
            let endpoint: Option<String> = row.get(1)?;
            Ok((key, endpoint))
          },
        )
        .optional()
        .map_err(|e| format!("Failed to read API key from SQLite: {}", e))?;

      Ok(row.map(|(key, endpoint)| ApiKey::new(provider, key, endpoint)))
    })();

    // 写入缓存
    if let Ok(ref val) = result {
      if let Ok(mut cache) = self.cache.lock().map_err(|e| format!("Lock error: {}", e)) {
        cache.insert(cache_key, val.clone());
      }
    }

    if let Err(ref e) = result {
      event = event.with_result(OperationResult::Failure);
      event = event.with_metadata(serde_json::json!({ "error": e }));
    }

    let _ = self.audit_logger.log_event(event);

    result
  }

  /// 删除 API Key
  pub fn delete_api_key(&self, provider: ModelProvider) -> Result<(), String> {
    let provider_key = Self::provider_key(&provider);

    // 记录审计事件
    let mut event = SecurityEvent::new(SecurityEventType::ConfigChange, OperationResult::Success)
      .with_resource(format!("api-key/{}", provider))
      .with_action("delete");

    let result = (|| {
      let conn = self.connection()?;
      conn
        .execute(
          "DELETE FROM api_keys WHERE provider = ?1",
          params![provider_key.as_str()],
        )
        .map_err(|e| format!("Failed to delete API key from SQLite: {}", e))?;

      Ok(())
    })();

    if let Err(ref e) = result {
      event = event.with_result(OperationResult::Failure);
      event = event.with_metadata(serde_json::json!({ "error": e }));
    }

    let _ = self.audit_logger.log_event(event);

    // 删除成功后更新缓存
    if result.is_ok() {
      if let Ok(mut cache) = self.cache.lock().map_err(|e| format!("Lock error: {}", e)) {
        cache.remove(&provider.to_string());
      }
    }

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
    let conn = self.connection()?;
    let mut statement = conn
      .prepare("SELECT provider FROM api_keys ORDER BY provider ASC")
      .map_err(|e| format!("Failed to list API key providers from SQLite: {}", e))?;
    let rows = statement
      .query_map([], |row| {
        let provider: String = row.get(0)?;
        Ok(provider)
      })
      .map_err(|e| format!("Failed to query API key providers from SQLite: {}", e))?;

    let mut providers = Vec::new();
    for row in rows {
      let provider = row.map_err(|e| format!("Failed to read API key provider row: {}", e))?;
      providers.push(Self::provider_from_key(&provider));
    }

    Ok(providers)
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
    assert_eq!(ModelProvider::Google.to_string(), "google");
    assert_eq!(ModelProvider::Ollama.to_string(), "ollama");
    assert_eq!(ModelProvider::Cohere.to_string(), "cohere");
    assert_eq!(ModelProvider::DeepSeek.to_string(), "deepseek");
    assert_eq!(ModelProvider::Moonshot.to_string(), "moonshot");
    assert_eq!(ModelProvider::ZhipuAI.to_string(), "zhipuai");
    assert_eq!(ModelProvider::MiniMax.to_string(), "minimax");
    assert_eq!(
      ModelProvider::Custom("my-provider".to_string()).to_string(),
      "custom:my-provider"
    );
  }

  #[test]
  fn test_keyring_manager_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = KeyringManager::new_with_path(temp_dir.path().join("api_keys.sqlite3"));
    assert!(manager.is_ok());
  }

  #[test]
  fn test_has_api_key_nonexistent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = KeyringManager::new_with_path(temp_dir.path().join("api_keys.sqlite3")).unwrap();
    // 检查一个不存在的提供商
    let result = manager.has_api_key(ModelProvider::Custom("nonexistent-test".to_string()));
    assert!(result.is_ok());
  }

  #[test]
  fn test_list_providers() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = KeyringManager::new_with_path(temp_dir.path().join("api_keys.sqlite3")).unwrap();
    let result = manager.list_providers();
    assert!(result.is_ok());
  }

  #[test]
  fn test_get_nonexistent_api_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = KeyringManager::new_with_path(temp_dir.path().join("api_keys.sqlite3")).unwrap();
    let result = manager.get_api_key(ModelProvider::Custom("nonexistent-get".to_string()));
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
  }

  #[test]
  fn test_delete_nonexistent_api_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = KeyringManager::new_with_path(temp_dir.path().join("api_keys.sqlite3")).unwrap();
    let result = manager.delete_api_key(ModelProvider::Custom("nonexistent-delete".to_string()));
    assert!(result.is_ok());
  }

  #[test]
  fn test_store_get_delete_api_key_sqlite() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = KeyringManager::new_with_path(temp_dir.path().join("api_keys.sqlite3")).unwrap();
    let provider = ModelProvider::DeepSeek;

    manager
      .store_api_key(ApiKey::new(
        provider.clone(),
        "sk-test",
        Some("https://api.deepseek.com".to_string()),
      ))
      .unwrap();

    let stored = manager.get_api_key(provider.clone()).unwrap().unwrap();
    assert_eq!(stored.key().expose_secret(), "sk-test");
    assert_eq!(stored.endpoint(), Some("https://api.deepseek.com"));
    assert!(manager.has_api_key(provider.clone()).unwrap());

    let providers = manager.list_providers().unwrap();
    assert_eq!(providers, vec![provider.clone()]);

    manager.delete_api_key(provider.clone()).unwrap();
    assert!(!manager.has_api_key(provider).unwrap());
  }
}
