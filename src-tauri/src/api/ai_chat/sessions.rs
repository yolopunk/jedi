// AI Chat 聊天会话管理
// Phase 2: 会话创建、列表、删除、消息追加

use crate::utils::security::{AuditLogger, OperationResult, SecurityEvent, SecurityEventType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

// ========== 会话数据结构 ==========

/// 聊天会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
  /// 会话 ID
  pub id: String,
  /// 会话标题
  pub title: String,
  /// 创建时间
  pub created_at: DateTime<Utc>,
  /// 更新时间
  pub updated_at: DateTime<Utc>,
  /// 消息列表
  pub messages: Vec<ChatMessage>,
  /// 使用的模型提供商
  pub provider: String,
  /// 使用的模型
  pub model: String,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
  /// 消息 ID
  pub id: String,
  /// 消息角色
  pub role: String,
  /// 消息内容
  pub content: String,
  /// 创建时间
  pub created_at: DateTime<Utc>,
}

impl ChatSession {
  /// 创建新的会话
  pub fn new(title: String, provider: String, model: String) -> Self {
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();

    Self {
      id,
      title,
      created_at: now,
      updated_at: now,
      messages: Vec::new(),
      provider,
      model,
    }
  }

  /// 添加消息
  pub fn add_message(&mut self, role: String, content: String) {
    let message = ChatMessage {
      id: uuid::Uuid::new_v4().to_string(),
      role,
      content,
      created_at: Utc::now(),
    };
    self.messages.push(message);
    self.updated_at = Utc::now();
  }
}

// ========== 会话管理器 ==========

/// 会话管理器
pub struct ChatSessionManager {
  /// 会话存储（内存存储，MVP 版本）
  sessions: HashMap<String, ChatSession>,
  /// 审计日志记录器
  audit_logger: AuditLogger,
}

impl ChatSessionManager {
  /// 创建新的会话管理器
  pub fn new(audit_logger: AuditLogger) -> Self {
    Self {
      sessions: HashMap::new(),
      audit_logger,
    }
  }

  /// 创建会话
  pub fn create_session(
    &mut self,
    title: String,
    provider: String,
    model: String,
  ) -> Result<ChatSession, String> {
    let session = ChatSession::new(title.clone(), provider.clone(), model.clone());
    let session_id = session.id.clone();
    self.sessions.insert(session_id.clone(), session.clone());

    // 记录审计日志
    let mut event = SecurityEvent::new(SecurityEventType::DataAccess, OperationResult::Success);
    event = event.with_resource(format!("session:{}", session_id));
    event = event.with_action("create");
    let _ = self.audit_logger.log_event(event);

    Ok(session)
  }

  /// 获取会话列表
  pub fn list_sessions(&self) -> Vec<ChatSession> {
    let mut sessions: Vec<_> = self.sessions.values().cloned().collect();
    // 按更新时间倒序排列
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    sessions
  }

  /// 获取会话
  pub fn get_session(&self, session_id: &str) -> Option<ChatSession> {
    self.sessions.get(session_id).cloned()
  }

  /// 删除会话
  pub fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
    if self.sessions.remove(session_id).is_some() {
      // 记录审计日志
      let mut event = SecurityEvent::new(SecurityEventType::DataAccess, OperationResult::Success);
      event = event.with_resource(format!("session:{}", session_id));
      event = event.with_action("delete");
      let _ = self.audit_logger.log_event(event);
      Ok(())
    } else {
      Err(format!("Session not found: {}", session_id))
    }
  }

  /// 追加消息到会话
  pub fn append_message(
    &mut self,
    session_id: &str,
    role: String,
    content: String,
  ) -> Result<ChatMessage, String> {
    let session = self
      .sessions
      .get_mut(session_id)
      .ok_or_else(|| format!("Session not found: {}", session_id))?;

    session.add_message(role.clone(), content.clone());
    let message = session.messages.last().unwrap().clone();

    // 记录审计日志
    let mut event = SecurityEvent::new(SecurityEventType::DataAccess, OperationResult::Success);
    event = event.with_resource(format!("session:{}", session_id));
    event = event.with_action("append_message");
    let _ = self.audit_logger.log_event(event);

    Ok(message)
  }
}

// ========== Tauri commands ==========

/// 会话管理器状态
pub struct ChatSessionManagerState {
  manager: Mutex<ChatSessionManager>,
}

impl ChatSessionManagerState {
  pub fn new(manager: ChatSessionManager) -> Self {
    Self {
      manager: Mutex::new(manager),
    }
  }
}

/// 创建会话请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
  /// 会话标题
  pub title: String,
  /// 模型提供商
  pub provider: String,
  /// 模型
  pub model: String,
}

/// 追加消息请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendMessageRequest {
  /// 会话 ID
  pub session_id: String,
  /// 消息角色
  pub role: String,
  /// 消息内容
  pub content: String,
}

/// Tauri command: 创建会话
#[tauri::command]
pub async fn create_session(
  state: State<'_, ChatSessionManagerState>,
  request: CreateSessionRequest,
) -> Result<ChatSession, String> {
  let mut manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  manager.create_session(request.title, request.provider, request.model)
}

/// Tauri command: 列出会话
#[tauri::command]
pub async fn list_sessions(
  state: State<'_, ChatSessionManagerState>,
) -> Result<Vec<ChatSession>, String> {
  let manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  Ok(manager.list_sessions())
}

/// Tauri command: 获取会话
#[tauri::command]
pub async fn get_session(
  state: State<'_, ChatSessionManagerState>,
  session_id: String,
) -> Result<Option<ChatSession>, String> {
  let manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  Ok(manager.get_session(&session_id))
}

/// Tauri command: 删除会话
#[tauri::command]
pub async fn delete_session(
  state: State<'_, ChatSessionManagerState>,
  session_id: String,
) -> Result<(), String> {
  let mut manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  manager.delete_session(&session_id)
}

/// Tauri command: 追加消息
#[tauri::command]
pub async fn append_message(
  state: State<'_, ChatSessionManagerState>,
  request: AppendMessageRequest,
) -> Result<ChatMessage, String> {
  let mut manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  manager.append_message(&request.session_id, request.role, request.content)
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::tempdir;

  #[test]
  fn test_chat_session_creation() {
    let session = ChatSession::new(
      "Test Session".to_string(),
      "openai".to_string(),
      "gpt-4".to_string(),
    );

    assert_eq!(session.title, "Test Session");
    assert_eq!(session.provider, "openai");
    assert_eq!(session.model, "gpt-4");
    assert!(session.messages.is_empty());
  }

  #[test]
  fn test_add_message() {
    let mut session = ChatSession::new(
      "Test Session".to_string(),
      "openai".to_string(),
      "gpt-4".to_string(),
    );

    session.add_message("user".to_string(), "Hello".to_string());
    session.add_message("assistant".to_string(), "Hi there!".to_string());

    assert_eq!(session.messages.len(), 2);
    assert_eq!(session.messages[0].role, "user");
    assert_eq!(session.messages[0].content, "Hello");
    assert_eq!(session.messages[1].role, "assistant");
    assert_eq!(session.messages[1].content, "Hi there!");
  }

  #[test]
  fn test_session_manager() {
    let temp_dir = tempdir().unwrap();
    let audit_logger = AuditLogger::new_with_path(temp_dir.path()).unwrap();
    let mut manager = ChatSessionManager::new(audit_logger);

    // 创建会话
    let session = manager
      .create_session(
        "Test Session".to_string(),
        "openai".to_string(),
        "gpt-4".to_string(),
      )
      .unwrap();

    assert_eq!(session.title, "Test Session");

    // 列出会话
    let sessions = manager.list_sessions();
    assert_eq!(sessions.len(), 1);

    // 获取会话
    let retrieved = manager.get_session(&session.id).unwrap();
    assert_eq!(retrieved.id, session.id);

    // 追加消息
    let message = manager
      .append_message(&session.id, "user".to_string(), "Hello".to_string())
      .unwrap();

    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello");

    // 删除会话
    manager.delete_session(&session.id).unwrap();
    let sessions = manager.list_sessions();
    assert!(sessions.is_empty());
  }
}
