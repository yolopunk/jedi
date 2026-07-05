// AI Chat 聊天会话管理
// Phase 2: 会话创建、列表、删除、消息追加

use crate::utils::security::{AuditLogger, OperationResult, SecurityEvent, SecurityEventType};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
  /// 附加元数据（JSON 字符串，保存前端的 agent trace / usage / reasoning）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<String>,
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

  /// 添加消息（仅用于内存场景与测试）
  pub fn add_message(&mut self, role: String, content: String) {
    let message = ChatMessage {
      id: uuid::Uuid::new_v4().to_string(),
      role,
      content,
      metadata: None,
      created_at: Utc::now(),
    };
    self.messages.push(message);
    self.updated_at = Utc::now();
  }
}

// ========== 会话管理器 ==========

/// 会话管理器（SQLite 持久化）
///
/// 会话与消息持久化到与 API Key 相同的 SQLite 数据库
/// （`<data_dir>/Jedi/ai_chat.sqlite3`），应用重启后历史可恢复。
pub struct ChatSessionManager {
  /// 数据库文件路径
  db_path: PathBuf,
  /// 审计日志记录器
  audit_logger: AuditLogger,
}

impl ChatSessionManager {
  /// 创建新的会话管理器（默认数据库位置）
  pub fn new(audit_logger: AuditLogger) -> Self {
    let db_path = Self::default_db_path();
    let manager = Self {
      db_path,
      audit_logger,
    };
    if let Err(e) = manager.init_schema() {
      log::error!("Failed to initialize chat session schema: {}", e);
    }
    manager
  }

  /// 使用自定义数据库路径创建（用于测试）
  pub fn new_with_path(db_path: impl Into<PathBuf>, audit_logger: AuditLogger) -> Self {
    let manager = Self {
      db_path: db_path.into(),
      audit_logger,
    };
    let _ = manager.init_schema();
    manager
  }

  /// 默认数据库路径，与 KeyringManager 复用同一个文件
  fn default_db_path() -> PathBuf {
    let base_dir = dirs::data_dir().unwrap_or_else(std::env::temp_dir);
    base_dir.join("Jedi").join("ai_chat.sqlite3")
  }

  /// 打开数据库连接
  fn open(&self) -> Result<Connection, String> {
    if let Some(parent) = self.db_path.parent() {
      std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create session database directory: {}", e))?;
    }
    let conn = Connection::open(&self.db_path)
      .map_err(|e| format!("Failed to open session database: {}", e))?;
    conn
      .pragma_update(None, "foreign_keys", "ON")
      .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;
    Ok(conn)
  }

  /// 初始化表结构
  fn init_schema(&self) -> Result<(), String> {
    let conn = self.open()?;
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS chat_sessions (
           id TEXT PRIMARY KEY NOT NULL,
           title TEXT NOT NULL,
           provider TEXT NOT NULL,
           model TEXT NOT NULL,
           created_at TEXT NOT NULL,
           updated_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS chat_messages (
           id TEXT PRIMARY KEY NOT NULL,
           session_id TEXT NOT NULL,
           role TEXT NOT NULL,
           content TEXT NOT NULL,
           metadata TEXT,
           created_at TEXT NOT NULL,
           seq INTEGER NOT NULL,
           FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
         );
         CREATE INDEX IF NOT EXISTS idx_chat_messages_session
           ON chat_messages(session_id, seq);",
      )
      .map_err(|e| format!("Failed to initialize session schema: {}", e))?;
    Ok(())
  }

  /// 读取一个会话的所有消息（按写入顺序）
  fn load_messages(conn: &Connection, session_id: &str) -> Result<Vec<ChatMessage>, String> {
    let mut stmt = conn
      .prepare(
        "SELECT id, role, content, metadata, created_at
           FROM chat_messages WHERE session_id = ?1 ORDER BY seq ASC",
      )
      .map_err(|e| format!("Failed to prepare message query: {}", e))?;
    let rows = stmt
      .query_map(params![session_id], |row| {
        let created_at: String = row.get(4)?;
        Ok(ChatMessage {
          id: row.get(0)?,
          role: row.get(1)?,
          content: row.get(2)?,
          metadata: row.get(3)?,
          created_at: DateTime::parse_from_rfc3339(&created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        })
      })
      .map_err(|e| format!("Failed to query messages: {}", e))?;
    let mut messages = Vec::new();
    for row in rows {
      messages.push(row.map_err(|e| format!("Failed to read message row: {}", e))?);
    }
    Ok(messages)
  }

  /// 创建会话
  pub fn create_session(
    &mut self,
    title: String,
    provider: String,
    model: String,
  ) -> Result<ChatSession, String> {
    let session = ChatSession::new(title, provider, model);
    let conn = self.open()?;
    conn
      .execute(
        "INSERT INTO chat_sessions (id, title, provider, model, created_at, updated_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
          session.id,
          session.title,
          session.provider,
          session.model,
          session.created_at.to_rfc3339(),
          session.updated_at.to_rfc3339(),
        ],
      )
      .map_err(|e| format!("Failed to insert session: {}", e))?;

    let mut event = SecurityEvent::new(SecurityEventType::DataAccess, OperationResult::Success);
    event = event.with_resource(format!("session:{}", session.id));
    event = event.with_action("create");
    let _ = self.audit_logger.log_event(event);

    Ok(session)
  }

  /// 获取会话列表（按更新时间倒序，含消息）
  pub fn list_sessions(&self) -> Vec<ChatSession> {
    let conn = match self.open() {
      Ok(conn) => conn,
      Err(e) => {
        log::error!("Failed to open session database: {}", e);
        return Vec::new();
      }
    };
    let mut stmt = match conn.prepare(
      "SELECT id, title, provider, model, created_at, updated_at
         FROM chat_sessions ORDER BY updated_at DESC",
    ) {
      Ok(stmt) => stmt,
      Err(e) => {
        log::error!("Failed to prepare session query: {}", e);
        return Vec::new();
      }
    };
    let rows = stmt.query_map([], |row| {
      let created_at: String = row.get(4)?;
      let updated_at: String = row.get(5)?;
      Ok((
        row.get::<_, String>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        row.get::<_, String>(3)?,
        created_at,
        updated_at,
      ))
    });
    let rows = match rows {
      Ok(rows) => rows,
      Err(e) => {
        log::error!("Failed to query sessions: {}", e);
        return Vec::new();
      }
    };
    let mut sessions = Vec::new();
    for row in rows.flatten() {
      let (id, title, provider, model, created_at, updated_at) = row;
      let messages = Self::load_messages(&conn, &id).unwrap_or_default();
      sessions.push(ChatSession {
        id,
        title,
        provider,
        model,
        created_at: DateTime::parse_from_rfc3339(&created_at)
          .map(|dt| dt.with_timezone(&Utc))
          .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_at)
          .map(|dt| dt.with_timezone(&Utc))
          .unwrap_or_else(|_| Utc::now()),
        messages,
      });
    }
    sessions
  }

  /// 获取会话
  pub fn get_session(&self, session_id: &str) -> Option<ChatSession> {
    let conn = self.open().ok()?;
    let mut session = conn
      .query_row(
        "SELECT id, title, provider, model, created_at, updated_at
           FROM chat_sessions WHERE id = ?1",
        params![session_id],
        |row| {
          let created_at: String = row.get(4)?;
          let updated_at: String = row.get(5)?;
          Ok(ChatSession {
            id: row.get(0)?,
            title: row.get(1)?,
            provider: row.get(2)?,
            model: row.get(3)?,
            created_at: DateTime::parse_from_rfc3339(&created_at)
              .map(|dt| dt.with_timezone(&Utc))
              .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at)
              .map(|dt| dt.with_timezone(&Utc))
              .unwrap_or_else(|_| Utc::now()),
            messages: Vec::new(),
          })
        },
      )
      .optional()
      .ok()??;
    session.messages = Self::load_messages(&conn, session_id).unwrap_or_default();
    Some(session)
  }

  /// 更新会话标题
  pub fn update_session_title(&mut self, session_id: &str, title: &str) -> Result<(), String> {
    let conn = self.open()?;
    let affected = conn
      .execute(
        "UPDATE chat_sessions SET title = ?1 WHERE id = ?2",
        params![title, session_id],
      )
      .map_err(|e| format!("Failed to update session title: {}", e))?;
    if affected == 0 {
      return Err(format!("Session not found: {}", session_id));
    }

    let mut event = SecurityEvent::new(SecurityEventType::DataAccess, OperationResult::Success);
    event = event.with_resource(format!("session:{}", session_id));
    event = event.with_action("update_title");
    let _ = self.audit_logger.log_event(event);
    Ok(())
  }

  /// 删除会话（级联删除消息）
  pub fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
    let conn = self.open()?;
    let affected = conn
      .execute("DELETE FROM chat_sessions WHERE id = ?1", params![session_id])
      .map_err(|e| format!("Failed to delete session: {}", e))?;
    if affected == 0 {
      return Err(format!("Session not found: {}", session_id));
    }

    let mut event = SecurityEvent::new(SecurityEventType::DataAccess, OperationResult::Success);
    event = event.with_resource(format!("session:{}", session_id));
    event = event.with_action("delete");
    let _ = self.audit_logger.log_event(event);
    Ok(())
  }

  /// 追加消息到会话
  pub fn append_message(
    &mut self,
    session_id: &str,
    role: String,
    content: String,
    metadata: Option<String>,
  ) -> Result<ChatMessage, String> {
    let conn = self.open()?;

    // 确认会话存在
    let exists: bool = conn
      .query_row(
        "SELECT 1 FROM chat_sessions WHERE id = ?1",
        params![session_id],
        |_| Ok(true),
      )
      .optional()
      .map_err(|e| format!("Failed to check session: {}", e))?
      .unwrap_or(false);
    if !exists {
      return Err(format!("Session not found: {}", session_id));
    }

    let next_seq: i64 = conn
      .query_row(
        "SELECT COALESCE(MAX(seq), -1) + 1 FROM chat_messages WHERE session_id = ?1",
        params![session_id],
        |row| row.get(0),
      )
      .map_err(|e| format!("Failed to compute message sequence: {}", e))?;

    let message = ChatMessage {
      id: uuid::Uuid::new_v4().to_string(),
      role,
      content,
      metadata,
      created_at: Utc::now(),
    };

    conn
      .execute(
        "INSERT INTO chat_messages (id, session_id, role, content, metadata, created_at, seq)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
          message.id,
          session_id,
          message.role,
          message.content,
          message.metadata,
          message.created_at.to_rfc3339(),
          next_seq,
        ],
      )
      .map_err(|e| format!("Failed to insert message: {}", e))?;

    conn
      .execute(
        "UPDATE chat_sessions SET updated_at = ?1 WHERE id = ?2",
        params![Utc::now().to_rfc3339(), session_id],
      )
      .map_err(|e| format!("Failed to update session timestamp: {}", e))?;

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
  /// 可选元数据（JSON 字符串，保存 agent trace / usage / reasoning）
  #[serde(default)]
  pub metadata: Option<String>,
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

/// 更新会话标题请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSessionTitleRequest {
  /// 会话 ID
  pub session_id: String,
  /// 新标题
  pub title: String,
}

/// Tauri command: 更新会话标题
#[tauri::command]
pub async fn update_session_title(
  state: State<'_, ChatSessionManagerState>,
  request: UpdateSessionTitleRequest,
) -> Result<(), String> {
  let mut manager = state
    .manager
    .lock()
    .map_err(|e| format!("Lock error: {}", e))?;
  manager.update_session_title(&request.session_id, &request.title)
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
  manager.append_message(
    &request.session_id,
    request.role,
    request.content,
    request.metadata,
  )
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
    let mut manager =
      ChatSessionManager::new_with_path(temp_dir.path().join("sessions.sqlite3"), audit_logger);

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
      .append_message(&session.id, "user".to_string(), "Hello".to_string(), None)
      .unwrap();

    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello");

    // 删除会话
    manager.delete_session(&session.id).unwrap();
    let sessions = manager.list_sessions();
    assert!(sessions.is_empty());
  }
}
