// 统一工具抽象层（Unified Tool Architecture）
// 见 docs/ai-chat/05-unified-tool-architecture.md
//
// Agent 回路只依赖这里的 AgentTool 抽象；MCP 只是它的一个来源/出口，不是唯一入口。
// - 主力：内置工具（native/）进程内直调
// - 可选：MCP 客户端（后续 P3）把第三方工具适配为 AgentTool 注册进来
// - 战略：MCP 服务端（后续 P4）把 registry 中可导出的工具对外暴露

pub mod native;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ============================================================================
// 核心类型
// ============================================================================

/// 工具风险等级 → 驱动确认策略（P2 使用）
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
  /// 只读：默认自动执行
  Read,
  /// 写入：默认 diff 预览后确认
  Write,
  /// 系统级/危险：强制二次确认
  System,
}

/// 工具来源
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ToolSource {
  /// 内置工具
  Native,
  /// 第三方 MCP server（携带 server_id 与远端原始工具名）
  Mcp {
    server_id: String,
    remote_name: String,
  },
}

/// 工具声明：喂给 LLM（name/description/input_schema），并用于 UI 展示
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDeclaration {
  /// 【R1】必须匹配 ^[a-zA-Z0-9_-]{1,64}$，一律下划线命名
  pub name: String,
  pub description: String,
  /// 标准 JSON Schema（object）
  pub input_schema: Value,
  pub risk: RiskLevel,
  pub source: ToolSource,
  /// UI 分组标签（也用作 ToolFilter 的分组匹配键），如 "hosts"
  pub group: String,
}

/// 改动预览（P2 确认 UI 使用）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolPreview {
  /// 人类可读 diff
  pub diff: String,
  /// 一致性快照 token：dry_run 打快照，call 执行前校验资源未被外部改动
  pub snapshot_token: String,
}

/// 工具执行结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolOutcome {
  /// 回填给 LLM 的文本
  pub content: String,
  pub is_error: bool,
  /// 回滚句柄（写前快照 id），仅可逆工具提供
  pub undo_token: Option<String>,
}

impl ToolOutcome {
  pub fn text(content: impl Into<String>) -> Self {
    Self {
      content: content.into(),
      is_error: false,
      undo_token: None,
    }
  }

  pub fn error(content: impl Into<String>) -> Self {
    Self {
      content: content.into(),
      is_error: true,
      undo_token: None,
    }
  }

  #[allow(dead_code)]
  pub fn with_undo(mut self, token: impl Into<String>) -> Self {
    self.undo_token = Some(token.into());
    self
  }
}

// ============================================================================
// AgentTool trait
// ============================================================================

#[async_trait]
pub trait AgentTool: Send + Sync {
  /// 工具声明
  fn declaration(&self) -> ToolDeclaration;

  /// 【Y2】按实际参数动态升级风险。默认返回声明里的静态风险。（P2 使用）
  #[allow(dead_code)]
  fn dynamic_risk(&self, _args: &Value) -> RiskLevel {
    self.declaration().risk
  }

  /// 【Y1】干跑：只计算改动预览 + 打一致性快照，不落地。Read 工具无需实现。（P2 使用）
  #[allow(dead_code)]
  async fn dry_run(&self, _args: &Value) -> Result<Option<ToolPreview>, String> {
    Ok(None)
  }

  /// 执行工具。expected_snapshot 为确认时锁定的快照（P2 使用）。
  async fn call(&self, args: Value, expected_snapshot: Option<String>) -> ToolOutcome;

  /// 回滚（仅可逆工具实现）。（P2 使用）
  #[allow(dead_code)]
  async fn undo(&self, _undo_token: &str) -> Result<(), String> {
    Err("此工具不支持回滚".into())
  }
}

// ============================================================================
// 工具过滤（§6.1 工具子集注入）
// ============================================================================

/// 按启用的来源过滤要注入给 LLM 的工具
#[derive(Debug, Clone, Default)]
pub struct ToolFilter {
  /// 启用的内置分组（如 ["hosts"]）
  pub enabled_groups: Vec<String>,
  /// 启用的 MCP server id
  pub enabled_servers: Vec<String>,
}

impl ToolFilter {
  fn accepts(&self, decl: &ToolDeclaration) -> bool {
    match &decl.source {
      ToolSource::Native => self
        .enabled_groups
        .iter()
        .any(|g| g.eq_ignore_ascii_case(&decl.group)),
      ToolSource::Mcp { server_id, .. } => self.enabled_servers.iter().any(|s| s == server_id),
    }
  }
}

// ============================================================================
// ToolRegistry
// ============================================================================

/// 工具注册表（作为 Tauri 托管状态，供 agent_chat 与工具命令共享）
pub struct ToolRegistry {
  tools: RwLock<HashMap<String, Arc<dyn AgentTool>>>,
}

impl Default for ToolRegistry {
  fn default() -> Self {
    Self::new()
  }
}

impl ToolRegistry {
  pub fn new() -> Self {
    Self {
      tools: RwLock::new(HashMap::new()),
    }
  }

  /// 创建并注册全部内置工具
  pub fn with_builtins() -> Self {
    let reg = Self::new();
    native::register_all(&reg);
    reg
  }

  /// 注册工具，重名拒绝
  pub fn register(&self, tool: Arc<dyn AgentTool>) -> Result<(), String> {
    let name = tool.declaration().name;
    let mut map = self.tools.write().map_err(|e| e.to_string())?;
    if map.contains_key(&name) {
      return Err(format!("工具重名: {}", name));
    }
    map.insert(name, tool);
    Ok(())
  }

  /// 注销工具（断开某 MCP server 时批量移除，P3 使用）
  #[allow(dead_code)]
  pub fn unregister(&self, name: &str) {
    if let Ok(mut map) = self.tools.write() {
      map.remove(name);
    }
  }

  /// 取工具（短暂持锁后 clone Arc 释放，避免跨 await 持锁）
  pub fn get(&self, name: &str) -> Option<Arc<dyn AgentTool>> {
    self.tools.read().ok()?.get(name).cloned()
  }

  /// 按过滤条件列出工具声明（注入给 LLM）
  pub fn declarations(&self, filter: &ToolFilter) -> Vec<ToolDeclaration> {
    match self.tools.read() {
      Ok(map) => map
        .values()
        .map(|t| t.declaration())
        .filter(|d| filter.accepts(d))
        .collect(),
      Err(_) => Vec::new(),
    }
  }

  /// 列出全部工具声明（供工具浏览器）
  pub fn all_declarations(&self) -> Vec<ToolDeclaration> {
    match self.tools.read() {
      Ok(map) => map.values().map(|t| t.declaration()).collect(),
      Err(_) => Vec::new(),
    }
  }

  /// 派发调用
  pub async fn call(&self, name: &str, args: Value, snapshot: Option<String>) -> ToolOutcome {
    match self.get(name) {
      Some(tool) => tool.call(args, snapshot).await,
      None => ToolOutcome::error(format!("未知工具: {}", name)),
    }
  }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_builtins_registered() {
    let reg = ToolRegistry::with_builtins();
    let names: Vec<String> = reg.all_declarations().into_iter().map(|d| d.name).collect();
    assert!(names.contains(&"hosts_read".to_string()));
    assert!(names.contains(&"hosts_add".to_string()));
  }

  #[test]
  fn test_names_are_function_call_safe() {
    // 【R1】所有工具名必须匹配 ^[a-zA-Z0-9_-]{1,64}$
    let reg = ToolRegistry::with_builtins();
    for decl in reg.all_declarations() {
      assert!(
        decl.name.len() <= 64
          && decl
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
        "非法工具名: {}",
        decl.name
      );
    }
  }

  #[test]
  fn test_register_rejects_duplicate() {
    let reg = ToolRegistry::with_builtins();
    let dup = native::hosts::tools().into_iter().next().unwrap();
    assert!(reg.register(dup).is_err());
  }

  #[test]
  fn test_filter_by_group() {
    let reg = ToolRegistry::with_builtins();
    let empty = reg.declarations(&ToolFilter::default());
    assert!(empty.is_empty(), "空过滤器不应注入任何工具");

    let hosts = reg.declarations(&ToolFilter {
      enabled_groups: vec!["hosts".into()],
      enabled_servers: vec![],
    });
    assert!(!hosts.is_empty());
    assert!(hosts.iter().all(|d| d.group == "hosts"));
  }
}
