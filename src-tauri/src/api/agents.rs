//! Local Agent group persistence. This module stores frontend-owned camelCase
//! JSON verbatim apart from server-owned `revision` and `updatedAt` fields.
//! It never opens a project directory, launches an agent, or makes network calls.

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::State;

const MAX_JSON_BYTES: usize = 4 * 1024 * 1024;
const MAX_MEMBERS: usize = 16;
const MAX_MESSAGES: usize = 10_000;
const MAX_RUNS: usize = 1_000;
const MAX_ID_BYTES: usize = 256;
const MAX_NAME_BYTES: usize = 256;
const MAX_PROJECT_PATH_BYTES: usize = 4_096;
// Sequences and revisions must remain exactly representable by JS numbers.
const MAX_SAFE_INTEGER: u64 = (1 << 53) - 1;

fn has_text(value: &str) -> bool {
  !value.trim().is_empty() && !value.contains('\0')
}

fn required_string<'a>(
  object: &'a Map<String, Value>,
  field: &str,
  max_bytes: usize,
) -> Result<&'a str, String> {
  let value = string_field(object, field)?;
  if !has_text(value) || value.len() > max_bytes {
    return Err(format!(
      "Agent 群组的 {} 不能为空、包含空字符或超过 {} 字节",
      field, max_bytes
    ));
  }
  Ok(value)
}

fn string_field<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str, String> {
  object
    .get(field)
    .and_then(Value::as_str)
    .ok_or_else(|| format!("Agent 群组的 {} 必须是字符串", field))
}

fn optional_string<'a>(
  object: &'a Map<String, Value>,
  field: &str,
) -> Result<Option<&'a str>, String> {
  match object.get(field) {
    None | Some(Value::Null) => Ok(None),
    Some(Value::String(value)) => Ok(Some(value)),
    _ => Err(format!("Agent 群组的 {} 必须是字符串或 null", field)),
  }
}

fn safe_integer(object: &Map<String, Value>, field: &str, min: u64) -> Result<u64, String> {
  object
    .get(field)
    .and_then(Value::as_u64)
    .filter(|value| (min..=MAX_SAFE_INTEGER).contains(value))
    .ok_or_else(|| format!("Agent 群组的 {} 必须是大于或等于 {} 的安全整数", field, min))
}

fn enum_field(object: &Map<String, Value>, field: &str, choices: &[&str]) -> Result<(), String> {
  if !choices.contains(&string_field(object, field)?) {
    return Err(format!("Agent 群组的 {} 无效", field));
  }
  Ok(())
}

fn array_field<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a [Value], String> {
  object
    .get(field)
    .and_then(Value::as_array)
    .map(Vec::as_slice)
    .ok_or_else(|| format!("Agent 群组的 {} 必须是数组", field))
}

// Borrow entries from the snapshot. The indices support reference checks
// without rewriting the arrays or losing unknown object fields.
type ObjectsById<'a> = HashMap<&'a str, &'a Map<String, Value>>;

fn index_collection<'a>(
  object: &'a Map<String, Value>,
  field: &str,
  limit: usize,
) -> Result<ObjectsById<'a>, String> {
  let items = array_field(object, field)?;
  if items.len() > limit {
    return Err(format!("Agent 群组的 {} 不能超过 {} 条", field, limit));
  }
  let mut by_id = HashMap::with_capacity(items.len());
  for item in items {
    let entry = item
      .as_object()
      .ok_or_else(|| format!("Agent 群组的 {} 中的条目必须是对象", field))?;
    let id = required_string(entry, "id", MAX_ID_BYTES)?;
    if by_id.insert(id, entry).is_some() {
      return Err(format!("Agent 群组的 {} 中存在重复 ID", field));
    }
  }
  Ok(by_id)
}

fn require_reference<'a>(
  entries: &ObjectsById<'a>,
  id: &str,
  field: &str,
) -> Result<&'a Map<String, Value>, String> {
  entries
    .get(id)
    .copied()
    .ok_or_else(|| format!("Agent 群组的 {} 引用了不存在的条目", field))
}

fn validate_messages(
  messages: &ObjectsById<'_>,
  members: &ObjectsById<'_>,
  runs: &ObjectsById<'_>,
) -> Result<u64, String> {
  let mut sequences = HashSet::with_capacity(messages.len());
  let mut max_seq = 0;
  for (&id, &message) in messages {
    let seq = safe_integer(message, "seq", 1)?;
    if !sequences.insert(seq) {
      return Err("Agent 群组 messages 中存在重复 seq".to_string());
    }
    max_seq = max_seq.max(seq);
    enum_field(message, "role", &["user", "assistant"])?;
    string_field(message, "authorName")?;
    string_field(message, "content")?; // Queued and streaming replies can be empty.
    required_string(message, "createdAt", 64)?;
    if let Some(author_id) = optional_string(message, "authorId")? {
      require_reference(members, author_id, "authorId")?;
    }
    for recipient in array_field(message, "recipients")? {
      let recipient = recipient
        .as_str()
        .ok_or_else(|| "Agent 群组消息的 recipients 必须是字符串数组".to_string())?;
      require_reference(members, recipient, "recipients")?;
    }
    if let Some(run_id) = optional_string(message, "runId")? {
      if !has_text(run_id) || run_id.len() > MAX_ID_BYTES {
        return Err("Agent 群组消息的 runId 无效".to_string());
      }
      // Historical messages can outlive pruned runs. Retained runs must point
      // back to the same reply.
      if let Some(run) = runs.get(run_id) {
        if string_field(run, "messageId")? != id {
          return Err("Agent 群组消息的 runId 与运行回复不匹配".to_string());
        }
      }
    }
  }
  Ok(max_seq)
}

fn validate_tools(run: &Map<String, Value>) -> Result<(), String> {
  // Tool identifiers are opaque CLI strings. The payload cap bounds their size
  // and the number of tools in both current and older snapshots.
  let tools = array_field(run, "tools")?;
  let mut ids = HashSet::with_capacity(tools.len());
  for tool in tools {
    let tool = tool
      .as_object()
      .ok_or_else(|| "Agent 群组的 tools 中的条目必须是对象".to_string())?;
    if !ids.insert(string_field(tool, "id")?) {
      return Err("Agent 群组的 tools 中存在重复 ID".to_string());
    }
    string_field(tool, "name")?;
    enum_field(tool, "status", &["running", "completed", "failed"])?;
    optional_string(tool, "detail")?;
  }
  Ok(())
}

fn validate_runs(
  runs: &ObjectsById<'_>,
  members: &ObjectsById<'_>,
  messages: &ObjectsById<'_>,
  max_seq: u64,
) -> Result<(), String> {
  let mut reply_ids = HashSet::with_capacity(runs.len());
  for (&id, &run) in runs {
    let member_id = string_field(run, "memberId")?;
    require_reference(members, member_id, "memberId")?;
    let trigger_id = string_field(run, "triggerMessageId")?;
    require_reference(messages, trigger_id, "triggerMessageId")?;
    let reply_id = string_field(run, "messageId")?;
    let reply = require_reference(messages, reply_id, "messageId")?;
    if reply_id == trigger_id
      || !reply_ids.insert(reply_id)
      || string_field(reply, "role")? != "assistant"
    {
      return Err("Agent 群组运行的 messageId 必须指向独立的助手回复".to_string());
    }
    if optional_string(reply, "authorId")?.is_some_and(|author_id| author_id != member_id)
      || optional_string(reply, "runId")?.is_some_and(|run_id| run_id != id)
    {
      return Err("Agent 群组运行与回复的 authorId 或 runId 不匹配".to_string());
    }
    enum_field(
      run,
      "status",
      &[
        "queued",
        "running",
        "awaiting_approval",
        "completed",
        "failed",
        "stopped",
        "interrupted",
      ],
    )?;
    if safe_integer(run, "contextSeq", 0)? > max_seq {
      return Err("Agent 群组运行的 contextSeq 超出消息历史".to_string());
    }
    for field in ["model", "error", "startedAt", "finishedAt"] {
      optional_string(run, field)?;
    }
    validate_tools(run)?;
  }
  Ok(())
}

/// Validate the camelCase snapshot without normalizing its contents. The UI
/// checks project path syntax, and a WSL/remote path need not exist on this host.
fn validate_group(group: &Value) -> Result<&str, String> {
  let object = group
    .as_object()
    .ok_or_else(|| "Agent 群组必须是对象".to_string())?;
  let id = required_string(object, "id", MAX_ID_BYTES)?;
  required_string(object, "name", MAX_NAME_BYTES)?;
  required_string(object, "projectPath", MAX_PROJECT_PATH_BYTES)?;
  enum_field(object, "executionMode", &["discussion", "approval"])?;
  required_string(object, "createdAt", 64)?;
  required_string(object, "updatedAt", 64)?;
  // New/migrated groups may omit revision. It never controls an upsert.
  if object.contains_key("revision") {
    safe_integer(object, "revision", 0)?;
  }

  let members = index_collection(object, "members", MAX_MEMBERS)?;
  let messages = index_collection(object, "messages", MAX_MESSAGES)?;
  let runs = index_collection(object, "runs", MAX_RUNS)?;
  let max_seq = validate_messages(&messages, &members, &runs)?;
  for member in members.values() {
    if !has_text(string_field(member, "engineId")?) {
      return Err("Agent 群组成员的 engineId 无效".to_string());
    }
    string_field(member, "name")?;
    string_field(member, "role")?; // The store accepts an empty role.
    if member.get("enabled").and_then(Value::as_bool).is_none() {
      return Err("Agent 群组成员的 enabled 必须是布尔值".to_string());
    }
    optional_string(member, "nativeSessionId")?;
    if safe_integer(member, "lastDeliveredSeq", 0)? > max_seq {
      return Err("Agent 群组成员的 lastDeliveredSeq 超出消息历史".to_string());
    }
  }
  validate_runs(&runs, &members, &messages, max_seq)?;
  Ok(id)
}

fn validate_payload_size(payload: &str) -> Result<(), String> {
  if payload.len() > MAX_JSON_BYTES {
    return Err("Agent 群组 JSON 不能超过 4 MiB".to_string());
  }
  Ok(())
}

fn encode_group(group: &Value) -> Result<String, String> {
  let payload =
    serde_json::to_string(group).map_err(|e| format!("序列化 Agent 群组失败: {}", e))?;
  validate_payload_size(&payload)?;
  Ok(payload)
}

struct AgentGroupManager {
  connection: Connection,
}

impl AgentGroupManager {
  fn new() -> Result<Self, String> {
    let base = dirs::data_dir().ok_or_else(|| "无法获取应用数据目录".to_string())?;
    Self::new_with_path(base.join("Jedi").join("agent_groups.sqlite3"))
  }

  fn new_with_path(db_path: impl AsRef<Path>) -> Result<Self, String> {
    let db_path = db_path.as_ref();
    if let Some(parent) = db_path.parent().filter(|path| !path.as_os_str().is_empty()) {
      std::fs::create_dir_all(parent)
        .map_err(|e| format!("创建 Agent 群组数据库目录失败: {}", e))?;
    }
    let connection =
      Connection::open(db_path).map_err(|e| format!("打开 Agent 群组数据库失败: {}", e))?;
    connection
      .busy_timeout(Duration::from_secs(5))
      .map_err(|e| format!("设置 Agent 群组数据库等待时间失败: {}", e))?;
    // FULL sync makes a successful commit durable, including its WAL contents.
    connection
      .execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = FULL;
         CREATE TABLE IF NOT EXISTS agent_groups (
           id TEXT PRIMARY KEY NOT NULL,
           updated_at TEXT NOT NULL,
           revision INTEGER NOT NULL CHECK (revision > 0),
           payload TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_agent_groups_updated_at
           ON agent_groups(updated_at DESC, id ASC);",
      )
      .map_err(|e| format!("初始化 Agent 群组数据库失败: {}", e))?;
    Ok(Self { connection })
  }

  fn list(&self) -> Result<Vec<Value>, String> {
    let mut statement = self
      .connection
      .prepare("SELECT payload FROM agent_groups ORDER BY updated_at DESC, id ASC")
      .map_err(|e| format!("准备 Agent 群组查询失败: {}", e))?;
    let rows = statement
      .query_map([], |row| row.get::<_, String>(0))
      .map_err(|e| format!("查询 Agent 群组失败: {}", e))?;
    let mut groups = Vec::new();
    for row in rows {
      let payload = row.map_err(|e| format!("读取 Agent 群组失败: {}", e))?;
      validate_payload_size(&payload)?;
      let group =
        serde_json::from_str(&payload).map_err(|e| format!("解析 Agent 群组数据失败: {}", e))?;
      // Do not silently turn a damaged database into an empty history.
      validate_group(&group)?;
      groups.push(group);
    }
    Ok(groups)
  }

  fn save(&mut self, mut group: Value) -> Result<Value, String> {
    encode_group(&group)?;
    let id = validate_group(&group)?.to_owned();
    // The write lock covers reading and incrementing the revision, so even
    // separate connections cannot assign the same revision to concurrent saves.
    let transaction = self
      .connection
      .transaction_with_behavior(TransactionBehavior::Immediate)
      .map_err(|e| format!("开始 Agent 群组保存事务失败: {}", e))?;
    let previous: Option<i64> = transaction
      .query_row(
        "SELECT revision FROM agent_groups WHERE id = ?1",
        params![id],
        |row| row.get(0),
      )
      .optional()
      .map_err(|e| format!("读取 Agent 群组版本失败: {}", e))?;
    let revision = previous
      .unwrap_or(0)
      .checked_add(1)
      .filter(|value| *value <= MAX_SAFE_INTEGER as i64)
      .ok_or_else(|| "Agent 群组 revision 已达到上限".to_string())?;
    let updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    group["revision"] = Value::from(revision);
    group["updatedAt"] = Value::from(updated_at.clone());
    // Also check the normalized payload: a longer revision can cross the limit.
    let payload = encode_group(&group)?;
    transaction
      .execute(
        "INSERT INTO agent_groups (id, updated_at, revision, payload)
           VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET
           updated_at = excluded.updated_at,
           revision = excluded.revision,
           payload = excluded.payload",
        params![id, updated_at, revision, payload],
      )
      .map_err(|e| format!("保存 Agent 群组失败: {}", e))?;
    transaction
      .commit()
      .map_err(|e| format!("提交 Agent 群组保存失败: {}", e))?;
    Ok(group)
  }

  fn delete(&self, group_id: &str) -> Result<(), String> {
    if group_id.trim().is_empty() || group_id.contains('\0') || group_id.len() > MAX_ID_BYTES {
      return Err("Agent 群组 ID 不能为空、包含空字符或超过 256 字节".to_string());
    }
    // A single DELETE is an atomic durable SQLite autocommit transaction.
    self
      .connection
      .execute("DELETE FROM agent_groups WHERE id = ?1", params![group_id])
      .map_err(|e| format!("删除 Agent 群组失败: {}", e))?;
    Ok(())
  }
}

/// Shared by all Tauri command workers; no blocking SQLite work runs on the UI.
pub struct AgentGroupManagerState {
  manager: Arc<Mutex<AgentGroupManager>>,
}

impl AgentGroupManagerState {
  pub fn new() -> Result<Self, String> {
    Ok(Self {
      manager: Arc::new(Mutex::new(AgentGroupManager::new()?)),
    })
  }

  async fn run<T: Send + 'static>(
    &self,
    operation: impl FnOnce(&mut AgentGroupManager) -> Result<T, String> + Send + 'static,
  ) -> Result<T, String> {
    let manager = Arc::clone(&self.manager);
    tauri::async_runtime::spawn_blocking(move || {
      let mut manager = manager
        .lock()
        .map_err(|_| "Agent 群组存储锁定失败".to_string())?;
      operation(&mut manager)
    })
    .await
    .map_err(|e| format!("Agent 群组存储任务失败: {}", e))?
  }
}

#[tauri::command]
pub async fn list_agent_groups(
  state: State<'_, AgentGroupManagerState>,
) -> Result<Vec<Value>, String> {
  state.run(|manager| manager.list()).await
}

#[tauri::command]
pub async fn save_agent_group(
  state: State<'_, AgentGroupManagerState>,
  group: Value,
) -> Result<Value, String> {
  state.run(move |manager| manager.save(group)).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_agent_group(
  state: State<'_, AgentGroupManagerState>,
  group_id: String,
) -> Result<(), String> {
  state.run(move |manager| manager.delete(&group_id)).await
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;
  use tempfile::tempdir;

  fn group(id: &str) -> Value {
    json!({
      "id": id,
      "name": "Local group",
      "projectPath": "/mnt/work/project",
      "executionMode": "discussion",
      "members": [{
        "id": "member-b", "engineId": "local-cli", "name": "Reviewer",
        "role": "Review changes", "enabled": true,
        "nativeSessionId": "native-session-1", "lastDeliveredSeq": 1,
        "futureField": {"memberMetadata": true}
      }],
      "messages": [
        {"id": "message-z", "seq": 1, "role": "user", "authorName": "You",
         "recipients": ["member-b"], "content": "Review this", "createdAt": "2026-01-01T00:00:00.000Z"},
        {"id": "message-a", "seq": 2, "role": "assistant", "authorId": "member-b",
         "authorName": "Reviewer", "recipients": ["member-b"], "content": "Reviewed",
         "createdAt": "2026-01-01T00:00:00.000Z", "runId": "run-z"},
        {"id": "message-y", "seq": 3, "role": "user", "authorName": "You",
         "recipients": ["member-b"], "content": "Any follow-up?", "createdAt": "2026-01-01T00:00:02.000Z"},
        {"id": "message-next", "seq": 4, "role": "assistant", "authorId": "member-b",
         "authorName": "Reviewer", "recipients": ["member-b"], "content": "",
         "createdAt": "2026-01-01T00:00:02.000Z", "runId": "run-a"}
      ],
      "runs": [
        {"id": "run-z", "memberId": "member-b", "triggerMessageId": "message-z",
         "messageId": "message-a", "status": "completed", "contextSeq": 1,
         "model": "local-model", "startedAt": "2026-01-01T00:00:00.000Z",
         "finishedAt": "2026-01-01T00:00:01.000Z",
         "tools": [{"id": "tool-1", "name": "read", "status": "completed", "detail": "file", "futureField": [1]}]},
        {"id": "run-a", "memberId": "member-b", "triggerMessageId": "message-y",
         "messageId": "message-next", "status": "queued", "contextSeq": 3, "tools": []}
      ],
      "createdAt": "2026-01-01T00:00:00.000Z",
      "updatedAt": "2026-01-01T00:00:00.000Z",
      "revision": 0,
      "futureField": {"keep": [1, true, "value"]}
    })
  }

  #[test]
  fn schema_initializes_and_reopens() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("Jedi/agent_groups.sqlite3");
    let manager = AgentGroupManager::new_with_path(&path).unwrap();
    let mut statement = manager
      .connection
      .prepare("PRAGMA table_info(agent_groups)")
      .unwrap();
    let columns: Vec<String> = statement
      .query_map([], |row| row.get(1))
      .unwrap()
      .map(Result::unwrap)
      .collect();
    assert_eq!(columns, ["id", "updated_at", "revision", "payload"]);
    let journal_mode: String = manager
      .connection
      .query_row("PRAGMA journal_mode", [], |row| row.get(0))
      .unwrap();
    let synchronous: i64 = manager
      .connection
      .query_row("PRAGMA synchronous", [], |row| row.get(0))
      .unwrap();
    assert_eq!(journal_mode, "wal");
    assert_eq!(synchronous, 2);
    assert!(manager.list().unwrap().is_empty());
    drop(statement);
    drop(manager);
    assert!(AgentGroupManager::new_with_path(&path)
      .unwrap()
      .list()
      .unwrap()
      .is_empty());
  }

  #[test]
  fn upsert_preserves_complete_payload_and_assigns_revision() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("agent_groups.sqlite3");
    let mut manager = AgentGroupManager::new_with_path(&path).unwrap();
    let mut original = group("group-1");
    original["revision"] = json!(50);
    let first = manager.save(original.clone()).unwrap();
    assert_eq!(first["revision"], 1);
    assert_ne!(first["updatedAt"], original["updatedAt"]);
    assert!(chrono::DateTime::parse_from_rfc3339(first["updatedAt"].as_str().unwrap()).is_ok());
    original["revision"] = first["revision"].clone();
    original["updatedAt"] = first["updatedAt"].clone();
    assert_eq!(first, original);

    let mut update = first.clone();
    update["name"] = json!("Renamed");
    update["executionMode"] = json!("approval");
    update["members"][0]["nativeSessionId"] = json!("native-session-2");
    update["members"][0]["lastDeliveredSeq"] = json!(3);
    update["runs"][1]["status"] = json!("interrupted");
    update["runs"][1]["error"] = json!("Restarted");
    let mut message = update["messages"][0].clone();
    message["id"] = json!("message-appended");
    message["seq"] = json!(5);
    update["messages"].as_array_mut().unwrap().push(message);
    let second = manager.save(update.clone()).unwrap();
    assert_eq!(second["revision"], 2);
    update["revision"] = second["revision"].clone();
    update["updatedAt"] = second["updatedAt"].clone();
    assert_eq!(second, update);
    assert_eq!(manager.list().unwrap(), vec![second.clone()]);

    // Incoming revisions are not an optimistic-lock precondition.
    let third = manager.save(first).unwrap();
    assert_eq!(third["revision"], 3);
    let (revision, updated_at, payload): (i64, String, String) = manager
      .connection
      .query_row(
        "SELECT revision, updated_at, payload FROM agent_groups WHERE id = ?1",
        ["group-1"],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
      )
      .unwrap();
    assert_eq!(revision, 3);
    assert_eq!(third["updatedAt"], updated_at);
    assert_eq!(serde_json::from_str::<Value>(&payload).unwrap(), third);
    drop(manager);
    assert_eq!(
      AgentGroupManager::new_with_path(&path)
        .unwrap()
        .list()
        .unwrap(),
      vec![third]
    );
  }

  #[test]
  fn lists_by_update_time_with_stable_id_order() {
    let dir = tempdir().unwrap();
    let manager = AgentGroupManager::new_with_path(dir.path().join("groups.sqlite3")).unwrap();
    for (id, updated_at) in [
      ("old", "2026-01-01T00:00:00.000Z"),
      ("b", "2026-01-02T00:00:00.000Z"),
      ("a", "2026-01-02T00:00:00.000Z"),
    ] {
      let mut value = group(id);
      value["revision"] = json!(1);
      value["updatedAt"] = json!(updated_at);
      manager
        .connection
        .execute(
          "INSERT INTO agent_groups (id, updated_at, revision, payload) VALUES (?1, ?2, 1, ?3)",
          params![id, updated_at, encode_group(&value).unwrap()],
        )
        .unwrap();
    }
    let ids: Vec<String> = manager
      .list()
      .unwrap()
      .into_iter()
      .map(|value| value["id"].as_str().unwrap().to_string())
      .collect();
    assert_eq!(ids, ["a", "b", "old"]);
  }

  #[test]
  fn delete_is_durable_idempotent_and_validated() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("groups.sqlite3");
    let mut manager = AgentGroupManager::new_with_path(&path).unwrap();
    manager.save(group("remove")).unwrap();
    let keep = manager.save(group("keep")).unwrap();
    for invalid in [
      String::new(),
      "  ".to_string(),
      "bad\0id".to_string(),
      "x".repeat(MAX_ID_BYTES + 1),
    ] {
      assert!(manager.delete(&invalid).is_err());
    }
    manager.delete("remove").unwrap();
    manager.delete("remove").unwrap();
    manager.delete("missing").unwrap();
    drop(manager);
    assert_eq!(
      AgentGroupManager::new_with_path(&path)
        .unwrap()
        .list()
        .unwrap(),
      vec![keep]
    );
  }

  #[test]
  fn rejects_invalid_envelopes_without_overwriting() {
    let dir = tempdir().unwrap();
    let mut manager = AgentGroupManager::new_with_path(dir.path().join("groups.sqlite3")).unwrap();
    let saved = manager.save(group("group-1")).unwrap();
    for invalid in [Value::Null, json!([]), json!("group")] {
      assert!(manager.save(invalid).is_err());
    }
    for field in [
      "id",
      "name",
      "projectPath",
      "executionMode",
      "createdAt",
      "updatedAt",
      "members",
      "messages",
      "runs",
    ] {
      let mut invalid = group("group-1");
      invalid.as_object_mut().unwrap().remove(field);
      assert!(manager.save(invalid).is_err(), "missing {}", field);
    }
    for field in ["id", "name", "projectPath"] {
      for invalid_value in [Value::Null, json!(42), json!("  "), json!("bad\0value")] {
        let mut invalid = group("group-1");
        invalid[field] = invalid_value;
        assert!(manager.save(invalid).is_err(), "invalid {}", field);
      }
    }
    for (field, limit) in [
      ("id", MAX_ID_BYTES),
      ("name", MAX_NAME_BYTES),
      ("projectPath", MAX_PROJECT_PATH_BYTES),
    ] {
      let mut invalid = group("group-1");
      invalid[field] = json!("x".repeat(limit + 1));
      assert!(manager.save(invalid).is_err(), "long {}", field);
    }
    for field in ["members", "messages", "runs"] {
      for invalid_value in [json!({}), json!([null]), json!(["entry"])] {
        let mut invalid = group("group-1");
        invalid[field] = invalid_value;
        assert!(manager.save(invalid).is_err(), "invalid {}", field);
      }
    }
    for (field, invalid_value) in [
      ("executionMode", json!("sandbox")),
      ("revision", json!(-1)),
      ("revision", json!(1.5)),
      ("revision", json!("1")),
      ("revision", json!(MAX_SAFE_INTEGER + 1)),
    ] {
      let mut invalid = group("group-1");
      invalid[field] = invalid_value;
      assert!(manager.save(invalid).is_err(), "invalid {}", field);
    }
    assert_eq!(manager.list().unwrap(), vec![saved]);
  }

  #[test]
  fn rejects_malformed_nested_fields_without_overwriting() {
    let dir = tempdir().unwrap();
    let mut manager = AgentGroupManager::new_with_path(dir.path().join("groups.sqlite3")).unwrap();
    let saved = manager.save(group("group-1")).unwrap();
    for (path, value) in [
      ("/members/0/id", json!("")),
      ("/members/0/engineId", json!(false)),
      ("/members/0/name", Value::Null),
      ("/members/0/role", json!([])),
      ("/members/0/enabled", json!(1)),
      ("/members/0/nativeSessionId", json!({})),
      ("/members/0/lastDeliveredSeq", json!(-1)),
      ("/members/0/lastDeliveredSeq", json!(5)),
      ("/messages/0/id", json!("message-a")),
      ("/messages/0/seq", json!(0)),
      ("/messages/0/seq", json!(1.5)),
      ("/messages/0/seq", json!(2)),
      ("/messages/0/seq", json!(MAX_SAFE_INTEGER + 1)),
      ("/messages/0/role", json!("system")),
      ("/messages/0/authorName", json!(false)),
      ("/messages/0/recipients", json!("member-b")),
      ("/messages/0/recipients/0", Value::Null),
      ("/messages/0/recipients/0", json!("missing-member")),
      ("/messages/0/content", json!([])),
      ("/messages/0/createdAt", json!(123)),
      ("/messages/1/authorId", json!(42)),
      ("/messages/1/authorId", json!("missing-member")),
      ("/messages/1/runId", json!(true)),
      ("/messages/1/runId", json!("run-a")),
      ("/runs/1/id", json!("run-z")),
      ("/runs/0/memberId", json!("missing-member")),
      ("/runs/0/triggerMessageId", json!("missing-trigger")),
      ("/runs/0/messageId", json!("missing-reply")),
      ("/runs/0/messageId", json!("message-y")),
      ("/runs/0/status", json!("unknown")),
      ("/runs/0/contextSeq", json!("1")),
      ("/runs/0/contextSeq", json!(5)),
      ("/runs/0/model", json!([])),
      ("/runs/0/startedAt", json!(false)),
      ("/runs/0/tools", Value::Null),
      ("/runs/0/tools/0", Value::Null),
      ("/runs/0/tools/0/id", json!(1)),
      ("/runs/0/tools/0/name", json!({})),
      ("/runs/0/tools/0/status", json!("queued")),
      ("/runs/0/tools/0/detail", json!(false)),
    ] {
      let mut invalid = group("group-1");
      *invalid.pointer_mut(path).unwrap() = value;
      assert!(manager.save(invalid).is_err(), "accepted invalid {path}");
    }
    let mut duplicate_member = group("group-1");
    let member = duplicate_member["members"][0].clone();
    duplicate_member["members"]
      .as_array_mut()
      .unwrap()
      .push(member);
    assert!(manager
      .save(duplicate_member)
      .unwrap_err()
      .contains("members"));
    let mut duplicate_tool = group("group-1");
    let tools = duplicate_tool["runs"][0]["tools"].as_array_mut().unwrap();
    tools.push(tools[0].clone());
    assert!(manager.save(duplicate_tool).unwrap_err().contains("tools"));
    let mut missing_content = group("group-1");
    missing_content["messages"][3]
      .as_object_mut()
      .unwrap()
      .remove("content");
    assert!(manager
      .save(missing_content)
      .unwrap_err()
      .contains("content"));
    assert_eq!(manager.list().unwrap(), vec![saved]);
  }

  #[test]
  fn accepts_nullable_optional_fields_and_pruned_run_history() {
    let dir = tempdir().unwrap();
    let mut manager = AgentGroupManager::new_with_path(dir.path().join("groups.sqlite3")).unwrap();
    let mut original = group("group-1");
    original["members"][0]["role"] = json!("");
    original["members"][0]["nativeSessionId"] = Value::Null;
    original["members"][0]["lastDeliveredSeq"] = json!(0);
    original["messages"][0]["runId"] = Value::Null;
    original["messages"][3]["authorId"] = Value::Null;
    original["runs"][0]["tools"][0]["detail"] = Value::Null;
    original["runs"][1]["error"] = Value::Null;
    original["runs"][1]["finishedAt"] = Value::Null;
    let saved = manager.save(original).unwrap();
    assert_eq!(saved["messages"][3]["content"], "");
    assert_eq!(saved["runs"][1]["status"], "queued");
    assert_eq!(saved["runs"][0]["tools"][0]["detail"], Value::Null);
    assert_eq!(manager.list().unwrap(), vec![saved.clone()]);

    let mut pruned = saved;
    pruned["runs"].as_array_mut().unwrap().remove(0);
    let saved = manager.save(pruned).unwrap();
    assert_eq!(saved["messages"][1]["runId"], "run-z");
    assert_eq!(manager.list().unwrap(), vec![saved]);
  }

  #[test]
  fn list_rejects_corrupt_nested_rows_and_oversized_payload() {
    let dir = tempdir().unwrap();
    let mut manager = AgentGroupManager::new_with_path(dir.path().join("groups.sqlite3")).unwrap();
    let mut corrupt = manager.save(group("group-1")).unwrap();
    corrupt["messages"][3]["content"] = json!({"invalid": true});
    manager
      .connection
      .execute(
        "UPDATE agent_groups SET payload = ?1 WHERE id = 'group-1'",
        [serde_json::to_string(&corrupt).unwrap()],
      )
      .unwrap();
    assert!(manager.list().unwrap_err().contains("content"));
    manager
      .connection
      .execute(
        "UPDATE agent_groups SET payload = ?1 WHERE id = 'group-1'",
        ["x".repeat(MAX_JSON_BYTES + 1)],
      )
      .unwrap();
    assert!(manager.list().unwrap_err().contains("4 MiB"));
  }

  #[test]
  fn collection_limits_are_inclusive() {
    let dir = tempdir().unwrap();
    let mut manager = AgentGroupManager::new_with_path(dir.path().join("groups.sqlite3")).unwrap();
    for (field, limit) in [
      ("members", MAX_MEMBERS),
      ("messages", MAX_MESSAGES),
      ("runs", MAX_RUNS),
    ] {
      let mut at_limit = group(field);
      let sample = at_limit[field][0].clone();
      for index in at_limit[field].as_array().unwrap().len()..limit {
        let mut entry = sample.clone();
        let id = format!("{field}-{index}");
        entry["id"] = json!(id);
        if field == "messages" {
          entry["seq"] = json!(index + 1);
        } else if field == "runs" {
          let mut reply = at_limit["messages"][1].clone();
          reply["id"] = json!(format!("reply-{index}"));
          reply["seq"] = json!(at_limit["messages"].as_array().unwrap().len() + 1);
          reply["runId"] = json!(id);
          entry["messageId"] = reply["id"].clone();
          at_limit["messages"].as_array_mut().unwrap().push(reply);
        }
        at_limit[field].as_array_mut().unwrap().push(entry);
      }
      let saved = manager.save(at_limit.clone()).unwrap();
      at_limit[field].as_array_mut().unwrap().push(sample);
      assert!(manager.save(at_limit).unwrap_err().contains(field));
      let stored = manager
        .list()
        .unwrap()
        .into_iter()
        .find(|value| value["id"] == field)
        .unwrap();
      assert_eq!(stored, saved);
    }
  }

  #[test]
  fn json_size_limit_counts_serialized_bytes_and_rolls_back() {
    let dir = tempdir().unwrap();
    let mut manager = AgentGroupManager::new_with_path(dir.path().join("groups.sqlite3")).unwrap();
    let mut at_limit = group("large");
    at_limit["padding"] = json!("");
    let overhead = serde_json::to_string(&at_limit).unwrap().len();
    at_limit["padding"] = json!("x".repeat(MAX_JSON_BYTES - overhead));
    assert_eq!(encode_group(&at_limit).unwrap().len(), MAX_JSON_BYTES);
    let saved = manager.save(at_limit.clone()).unwrap();
    assert_eq!(encode_group(&saved).unwrap().len(), MAX_JSON_BYTES);
    at_limit["padding"] = json!("x".repeat(MAX_JSON_BYTES - overhead + 1));
    assert!(manager
      .save(at_limit.clone())
      .unwrap_err()
      .contains("4 MiB"));
    // Escaping can exceed the cap even when raw string bytes do not.
    at_limit["padding"] = json!("\n".repeat(MAX_JSON_BYTES / 2));
    assert!(manager.save(at_limit).unwrap_err().contains("4 MiB"));

    manager
      .connection
      .execute(
        "UPDATE agent_groups SET revision = 9 WHERE id = 'large'",
        [],
      )
      .unwrap();
    let mut normalized_over_limit = group("large");
    normalized_over_limit["padding"] = saved["padding"].clone();
    assert_eq!(
      encode_group(&normalized_over_limit).unwrap().len(),
      MAX_JSON_BYTES
    );
    assert!(manager
      .save(normalized_over_limit)
      .unwrap_err()
      .contains("4 MiB"));
    let revision: i64 = manager
      .connection
      .query_row(
        "SELECT revision FROM agent_groups WHERE id = 'large'",
        [],
        |row| row.get(0),
      )
      .unwrap();
    assert_eq!(revision, 9);
    assert_eq!(manager.list().unwrap(), vec![saved]);
  }

  #[test]
  fn concurrent_connections_assign_distinct_revisions() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("groups.sqlite3");
    let managers: Vec<_> = (0..4)
      .map(|_| AgentGroupManager::new_with_path(&path).unwrap())
      .collect();
    let barrier = Arc::new(std::sync::Barrier::new(managers.len()));
    let handles: Vec<_> = managers
      .into_iter()
      .map(|mut manager| {
        let barrier = Arc::clone(&barrier);
        std::thread::spawn(move || {
          barrier.wait();
          let saved = manager.save(group("shared")).unwrap();
          saved["revision"].as_i64().unwrap()
        })
      })
      .collect();
    let mut revisions: Vec<_> = handles
      .into_iter()
      .map(|handle| handle.join().unwrap())
      .collect();
    revisions.sort_unstable();
    assert_eq!(revisions, [1, 2, 3, 4]);
    let groups = AgentGroupManager::new_with_path(&path)
      .unwrap()
      .list()
      .unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["revision"], 4);
  }

  #[test]
  fn revision_overflow_and_corrupt_payload_return_errors() {
    let dir = tempdir().unwrap();
    let mut manager = AgentGroupManager::new_with_path(dir.path().join("groups.sqlite3")).unwrap();
    let saved = manager.save(group("group-1")).unwrap();
    manager
      .connection
      .execute(
        "UPDATE agent_groups SET revision = ?1 WHERE id = 'group-1'",
        [MAX_SAFE_INTEGER as i64],
      )
      .unwrap();
    assert!(manager
      .save(group("group-1"))
      .unwrap_err()
      .contains("revision"));
    assert_eq!(manager.list().unwrap(), vec![saved]);
    manager
      .connection
      .execute(
        "UPDATE agent_groups SET payload = 'not json' WHERE id = 'group-1'",
        [],
      )
      .unwrap();
    assert!(manager.list().is_err());
  }
}
