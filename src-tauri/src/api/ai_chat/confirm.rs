// Agent 确认与回滚运行时（【R2】可挂起回路 + 【Y4】per-turn 回滚）
//
// PendingConfirmations：回路遇到 Write/System 工具时挂起，等待前端 tool_confirm 唤醒。
// UndoStacks：按 request_id 记录本回合的可撤销工具，支持整回合/单步回滚。

use crate::tools::{RiskLevel, ToolRegistry};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tauri::State;
use tokio::sync::oneshot;

// ============================================================================
// 确认模式
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConfirmMode {
  /// Read+Write 自动执行，仅 System 确认
  Auto,
  /// Read 自动，Write+System 确认（默认）
  Normal,
}

impl ConfirmMode {
  pub fn parse(s: Option<&str>) -> Self {
    match s {
      Some("auto") => ConfirmMode::Auto,
      _ => ConfirmMode::Normal,
    }
  }
}

/// 给定风险与模式，是否需要人机确认
pub fn should_confirm(
  risk: RiskLevel,
  mode: ConfirmMode,
  auto_approve: &[String],
  name: &str,
) -> bool {
  if auto_approve.iter().any(|n| n == name) {
    return false;
  }
  match risk {
    RiskLevel::Read => false,
    RiskLevel::Write => mode != ConfirmMode::Auto,
    RiskLevel::System => true,
  }
}

// ============================================================================
// 确认决定
// ============================================================================

pub enum ConfirmDecision {
  Approve { edited_args: Option<serde_json::Value> },
  Reject,
}

// ============================================================================
// PendingConfirmations（Tauri 托管状态）
// ============================================================================

fn wkey(request_id: &str, call_id: &str) -> String {
  format!("{}::{}", request_id, call_id)
}

#[derive(Default)]
pub struct PendingConfirmations {
  waiters: Mutex<HashMap<String, oneshot::Sender<ConfirmDecision>>>,
  cancelled: Mutex<HashSet<String>>,
}

impl PendingConfirmations {
  pub fn new() -> Self {
    Self::default()
  }

  /// 登记一个等待者，返回接收端（回路 await 它）
  pub fn register(&self, request_id: &str, call_id: &str) -> oneshot::Receiver<ConfirmDecision> {
    let (tx, rx) = oneshot::channel();
    if let Ok(mut w) = self.waiters.lock() {
      w.insert(wkey(request_id, call_id), tx);
    }
    rx
  }

  /// 前端确认结果送达
  pub fn resolve(&self, request_id: &str, call_id: &str, decision: ConfirmDecision) -> bool {
    let sender = self
      .waiters
      .lock()
      .ok()
      .and_then(|mut w| w.remove(&wkey(request_id, call_id)));
    match sender {
      Some(tx) => tx.send(decision).is_ok(),
      None => false,
    }
  }

  /// 取消整个 request：标记取消 + 拒绝其所有挂起项
  pub fn cancel(&self, request_id: &str) {
    if let Ok(mut c) = self.cancelled.lock() {
      c.insert(request_id.to_string());
    }
    let prefix = format!("{}::", request_id);
    if let Ok(mut w) = self.waiters.lock() {
      let keys: Vec<String> = w.keys().filter(|k| k.starts_with(&prefix)).cloned().collect();
      for k in keys {
        if let Some(tx) = w.remove(&k) {
          let _ = tx.send(ConfirmDecision::Reject);
        }
      }
    }
  }

  pub fn is_cancelled(&self, request_id: &str) -> bool {
    self
      .cancelled
      .lock()
      .map(|c| c.contains(request_id))
      .unwrap_or(false)
  }

  /// 回路结束时清理该 request 的残留
  pub fn clear(&self, request_id: &str) {
    if let Ok(mut c) = self.cancelled.lock() {
      c.remove(request_id);
    }
    let prefix = format!("{}::", request_id);
    if let Ok(mut w) = self.waiters.lock() {
      let keys: Vec<String> = w.keys().filter(|k| k.starts_with(&prefix)).cloned().collect();
      for k in keys {
        w.remove(&k);
      }
    }
  }
}

// ============================================================================
// UndoStacks（Tauri 托管状态）
// ============================================================================

#[derive(Clone, Serialize)]
pub struct UndoEntry {
  pub tool: String,
  pub token: String,
  pub label: String,
}

#[derive(Default)]
pub struct UndoStacks {
  stacks: Mutex<HashMap<String, Vec<UndoEntry>>>,
}

impl UndoStacks {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn push(&self, request_id: &str, entry: UndoEntry) {
    if let Ok(mut m) = self.stacks.lock() {
      m.entry(request_id.to_string()).or_default().push(entry);
    }
  }

  pub fn take(&self, request_id: &str) -> Vec<UndoEntry> {
    self
      .stacks
      .lock()
      .ok()
      .and_then(|mut m| m.remove(request_id))
      .unwrap_or_default()
  }

  pub fn remove_token(&self, request_id: &str, token: &str) -> Option<UndoEntry> {
    let mut m = self.stacks.lock().ok()?;
    let v = m.get_mut(request_id)?;
    let pos = v.iter().position(|e| e.token == token)?;
    Some(v.remove(pos))
  }
}

// ============================================================================
// Tauri commands
// ============================================================================

/// 前端对某个挂起的工具调用做出确认
#[tauri::command]
pub fn tool_confirm(
  pending: State<'_, PendingConfirmations>,
  request_id: String,
  call_id: String,
  approve: bool,
  edited_args: Option<serde_json::Value>,
) -> Result<(), String> {
  let decision = if approve {
    ConfirmDecision::Approve { edited_args }
  } else {
    ConfirmDecision::Reject
  };
  if pending.resolve(&request_id, &call_id, decision) {
    Ok(())
  } else {
    Err("确认请求不存在或已超时".to_string())
  }
}

/// 取消整个 Agent 回路
#[tauri::command]
pub fn agent_cancel(pending: State<'_, PendingConfirmations>, request_id: String) {
  pending.cancel(&request_id);
}

/// 整回合逆序回滚
#[tauri::command]
pub async fn turn_undo(
  registry: State<'_, ToolRegistry>,
  undo: State<'_, UndoStacks>,
  request_id: String,
) -> Result<Vec<String>, String> {
  let entries = undo.take(&request_id);
  let mut results = Vec::new();
  for entry in entries.into_iter().rev() {
    match registry.get(&entry.tool) {
      Some(tool) => match tool.undo(&entry.token).await {
        Ok(()) => results.push(format!("已撤销: {}", entry.label)),
        Err(e) => results.push(format!("撤销失败 {}: {}", entry.label, e)),
      },
      None => results.push(format!("工具不存在: {}", entry.tool)),
    }
  }
  Ok(results)
}

/// 单步回滚指定的 undo_token
#[tauri::command]
pub async fn tool_undo(
  registry: State<'_, ToolRegistry>,
  undo: State<'_, UndoStacks>,
  request_id: String,
  undo_token: String,
) -> Result<String, String> {
  let entry = undo
    .remove_token(&request_id, &undo_token)
    .ok_or_else(|| "回滚项不存在".to_string())?;
  let tool = registry
    .get(&entry.tool)
    .ok_or_else(|| "工具不存在".to_string())?;
  tool
    .undo(&entry.token)
    .await
    .map(|_| format!("已撤销: {}", entry.label))
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_should_confirm_tiers() {
    let none: Vec<String> = vec![];
    // Read 从不确认
    assert!(!should_confirm(RiskLevel::Read, ConfirmMode::Normal, &none, "x"));
    // Write 在 Normal 确认，Auto 不确认
    assert!(should_confirm(RiskLevel::Write, ConfirmMode::Normal, &none, "x"));
    assert!(!should_confirm(RiskLevel::Write, ConfirmMode::Auto, &none, "x"));
    // System 始终确认
    assert!(should_confirm(RiskLevel::System, ConfirmMode::Auto, &none, "x"));
    // autoApprove 白名单免确认
    assert!(!should_confirm(
      RiskLevel::System,
      ConfirmMode::Normal,
      &["x".into()],
      "x"
    ));
  }

  #[test]
  fn test_mode_parse() {
    assert!(ConfirmMode::parse(Some("auto")) == ConfirmMode::Auto);
    assert!(ConfirmMode::parse(Some("normal")) == ConfirmMode::Normal);
    assert!(ConfirmMode::parse(None) == ConfirmMode::Normal);
  }

  #[test]
  fn test_pending_resolve_and_missing() {
    let p = PendingConfirmations::new();
    let _rx = p.register("req1", "callA");
    assert!(p.resolve("req1", "callA", ConfirmDecision::Reject));
    // 再次 resolve 应失败（已移除）
    assert!(!p.resolve("req1", "callA", ConfirmDecision::Reject));
    assert!(!p.resolve("req1", "missing", ConfirmDecision::Reject));
  }

  #[test]
  fn test_cancel_marks_and_rejects() {
    let p = PendingConfirmations::new();
    let _rx = p.register("req2", "c1");
    assert!(!p.is_cancelled("req2"));
    p.cancel("req2");
    assert!(p.is_cancelled("req2"));
    // 挂起项已被拒绝移除
    assert!(!p.resolve("req2", "c1", ConfirmDecision::Reject));
    p.clear("req2");
    assert!(!p.is_cancelled("req2"));
  }

  #[test]
  fn test_undo_stack_order() {
    let u = UndoStacks::new();
    u.push("r", UndoEntry { tool: "hosts_add".into(), token: "t1".into(), label: "a".into() });
    u.push("r", UndoEntry { tool: "hosts_add".into(), token: "t2".into(), label: "b".into() });
    assert!(u.remove_token("r", "t1").is_some());
    let rest = u.take("r");
    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0].token, "t2");
    assert!(u.take("r").is_empty());
  }
}
