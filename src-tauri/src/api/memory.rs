// Cross-session memory for the AI agent.
//
// Lets the agent remember long-term user preferences and frequently-used
// configuration across chat sessions. Persisted as a flat key/value map in
// ~/.jedi/agent_memory.json. This only touches Jedi's own private storage
// (never the system), so it is a low-risk, friction-free "remember / recall".

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// 记忆文件路径（~/.jedi/agent_memory.json），必要时创建目录
fn memory_path() -> Result<PathBuf, String> {
  let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
  let dir = home.join(".jedi");
  if !dir.exists() {
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 .jedi 目录失败: {}", e))?;
  }
  Ok(dir.join("agent_memory.json"))
}

fn load() -> Result<BTreeMap<String, String>, String> {
  let path = memory_path()?;
  Ok(
    std::fs::read_to_string(&path)
      .ok()
      .and_then(|s| serde_json::from_str(&s).ok())
      .unwrap_or_default(),
  )
}

fn store(map: &BTreeMap<String, String>) -> Result<(), String> {
  let path = memory_path()?;
  let s = serde_json::to_string_pretty(map).map_err(|e| e.to_string())?;
  std::fs::write(&path, s).map_err(|e| format!("写入记忆失败: {}", e))
}

#[derive(Debug, Serialize)]
pub struct MemoryEntry {
  pub key: String,
  pub value: String,
}

/// 记住一条长期信息（用户偏好、常用配置等），跨会话保留
#[tauri::command]
pub fn memory_save(key: String, value: String) -> Result<String, String> {
  let mut map = load()?;
  map.insert(key.clone(), value);
  store(&map)?;
  Ok(format!("已记住「{}」", key))
}

/// 按键回忆一条此前记住的信息
#[tauri::command]
pub fn memory_recall(key: String) -> Result<String, String> {
  let map = load()?;
  Ok(match map.get(&key) {
    Some(v) => v.clone(),
    None => format!("未找到记忆「{}」", key),
  })
}

/// 列出所有已记住的信息
#[tauri::command]
pub fn memory_list() -> Result<Vec<MemoryEntry>, String> {
  let map = load()?;
  Ok(
    map
      .into_iter()
      .map(|(key, value)| MemoryEntry { key, value })
      .collect(),
  )
}

/// 删除一条记忆
#[tauri::command]
pub fn memory_delete(key: String) -> Result<String, String> {
  let mut map = load()?;
  if map.remove(&key).is_some() {
    store(&map)?;
    Ok(format!("已删除记忆「{}」", key))
  } else {
    Ok(format!("未找到记忆「{}」", key))
  }
}
