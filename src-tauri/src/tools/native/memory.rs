// 记忆内置工具（⑤）：让 Agent 跨会话记住用户的长期偏好与常用配置。
//
// 持久化到 ~/.jedi/agent_memory.json（键值对）。记忆只触及 Jedi 私有存储、不涉及系统，
// 因此归为 Read 风险（无需确认），保证"记住/回忆"是无摩擦的常规行为。

use crate::tools::{AgentTool, RiskLevel, ToolDeclaration, ToolOutcome, ToolSource};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const GROUP: &str = "memory";

pub fn tools() -> Vec<Arc<dyn AgentTool>> {
  vec![
    Arc::new(MemorySave),
    Arc::new(MemoryRecall),
    Arc::new(MemoryList),
    Arc::new(MemoryDelete),
  ]
}

fn decl(name: &str, description: &str, schema: Value) -> ToolDeclaration {
  ToolDeclaration {
    name: name.into(),
    description: description.into(),
    input_schema: schema,
    risk: RiskLevel::Read,
    source: ToolSource::Native,
    group: GROUP.into(),
  }
}

fn get_str(args: &Value, key: &str) -> Result<String, String> {
  args
    .get(key)
    .and_then(|v| v.as_str())
    .map(|s| s.to_string())
    .ok_or_else(|| format!("缺少参数: {}", key))
}

/// 记忆文件路径（~/.jedi/agent_memory.json）
fn memory_path() -> Result<PathBuf, String> {
  let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
  let dir = home.join(".jedi");
  if !dir.exists() {
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 .jedi 目录失败: {}", e))?;
  }
  Ok(dir.join("agent_memory.json"))
}

fn load_at(path: &Path) -> BTreeMap<String, String> {
  std::fs::read_to_string(path)
    .ok()
    .and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or_default()
}

fn save_at(path: &Path, map: &BTreeMap<String, String>) -> Result<(), String> {
  let s = serde_json::to_string_pretty(map).map_err(|e| e.to_string())?;
  std::fs::write(path, s).map_err(|e| format!("写入记忆失败: {}", e))
}

// ---- 路径参数化的核心操作（便于单测）----

fn op_save(path: &Path, key: &str, value: &str) -> Result<String, String> {
  let mut map = load_at(path);
  map.insert(key.to_string(), value.to_string());
  save_at(path, &map)?;
  Ok(format!("已记住「{}」", key))
}

fn op_recall(path: &Path, key: &str) -> Result<String, String> {
  let map = load_at(path);
  match map.get(key) {
    Some(v) => Ok(v.clone()),
    None => Ok(format!("未找到记忆「{}」", key)),
  }
}

fn op_list(path: &Path) -> Result<String, String> {
  let map = load_at(path);
  if map.is_empty() {
    return Ok("（暂无记忆）".to_string());
  }
  serde_json::to_string_pretty(&map).map_err(|e| e.to_string())
}

fn op_delete(path: &Path, key: &str) -> Result<String, String> {
  let mut map = load_at(path);
  if map.remove(key).is_some() {
    save_at(path, &map)?;
    Ok(format!("已删除记忆「{}」", key))
  } else {
    Ok(format!("未找到记忆「{}」", key))
  }
}

// ============================================================================
// 工具
// ============================================================================

pub struct MemorySave;

#[async_trait]
impl AgentTool for MemorySave {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "memory_save",
      "记住一条长期信息（用户偏好、常用配置等），跨会话保留",
      json!({
        "type": "object",
        "properties": {
          "key": { "type": "string", "description": "记忆的键，如 preferred_wallpaper" },
          "value": { "type": "string", "description": "记忆的内容" }
        },
        "required": ["key", "value"]
      }),
    )
  }

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let path = match memory_path() {
      Ok(p) => p,
      Err(e) => return ToolOutcome::error(e),
    };
    let key = match get_str(&args, "key") {
      Ok(v) => v,
      Err(e) => return ToolOutcome::error(e),
    };
    let value = match get_str(&args, "value") {
      Ok(v) => v,
      Err(e) => return ToolOutcome::error(e),
    };
    match op_save(&path, &key, &value) {
      Ok(m) => ToolOutcome::text(m),
      Err(e) => ToolOutcome::error(e),
    }
  }
}

pub struct MemoryRecall;

#[async_trait]
impl AgentTool for MemoryRecall {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "memory_recall",
      "按键回忆一条此前记住的信息",
      json!({
        "type": "object",
        "properties": {
          "key": { "type": "string", "description": "要回忆的键" }
        },
        "required": ["key"]
      }),
    )
  }

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let path = match memory_path() {
      Ok(p) => p,
      Err(e) => return ToolOutcome::error(e),
    };
    let key = match get_str(&args, "key") {
      Ok(v) => v,
      Err(e) => return ToolOutcome::error(e),
    };
    match op_recall(&path, &key) {
      Ok(m) => ToolOutcome::text(m),
      Err(e) => ToolOutcome::error(e),
    }
  }
}

pub struct MemoryList;

#[async_trait]
impl AgentTool for MemoryList {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "memory_list",
      "列出所有已记住的信息",
      json!({ "type": "object", "properties": {} }),
    )
  }

  async fn call(&self, _args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let path = match memory_path() {
      Ok(p) => p,
      Err(e) => return ToolOutcome::error(e),
    };
    match op_list(&path) {
      Ok(m) => ToolOutcome::text(m),
      Err(e) => ToolOutcome::error(e),
    }
  }
}

pub struct MemoryDelete;

#[async_trait]
impl AgentTool for MemoryDelete {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "memory_delete",
      "删除一条记忆",
      json!({
        "type": "object",
        "properties": {
          "key": { "type": "string", "description": "要删除的键" }
        },
        "required": ["key"]
      }),
    )
  }

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let path = match memory_path() {
      Ok(p) => p,
      Err(e) => return ToolOutcome::error(e),
    };
    let key = match get_str(&args, "key") {
      Ok(v) => v,
      Err(e) => return ToolOutcome::error(e),
    };
    match op_delete(&path, &key) {
      Ok(m) => ToolOutcome::text(m),
      Err(e) => ToolOutcome::error(e),
    }
  }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicU64, Ordering};

  static SEQ: AtomicU64 = AtomicU64::new(1);

  fn temp_path() -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("jedi_memory_test_{}.json", n))
  }

  #[test]
  fn test_save_recall_roundtrip() {
    let p = temp_path();
    let _ = std::fs::remove_file(&p);
    assert!(op_recall(&p, "k").unwrap().contains("未找到"));
    assert!(op_save(&p, "k", "v1").unwrap().contains("已记住"));
    assert_eq!(op_recall(&p, "k").unwrap(), "v1");
    // 覆盖
    op_save(&p, "k", "v2").unwrap();
    assert_eq!(op_recall(&p, "k").unwrap(), "v2");
    let _ = std::fs::remove_file(&p);
  }

  #[test]
  fn test_list_and_delete() {
    let p = temp_path();
    let _ = std::fs::remove_file(&p);
    assert!(op_list(&p).unwrap().contains("暂无"));
    op_save(&p, "a", "1").unwrap();
    op_save(&p, "b", "2").unwrap();
    let listed = op_list(&p).unwrap();
    assert!(listed.contains("\"a\""));
    assert!(listed.contains("\"b\""));
    assert!(op_delete(&p, "a").unwrap().contains("已删除"));
    assert!(op_recall(&p, "a").unwrap().contains("未找到"));
    assert!(op_delete(&p, "missing").unwrap().contains("未找到"));
    let _ = std::fs::remove_file(&p);
  }

  #[test]
  fn test_all_read_risk() {
    for t in tools() {
      assert_eq!(t.declaration().risk, RiskLevel::Read, "{}", t.declaration().name);
    }
  }
}
