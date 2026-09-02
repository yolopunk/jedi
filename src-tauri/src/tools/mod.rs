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
  /// 注入上限：候选超过该数量时，按与 query 的相关性取 top-K
  pub max_tools: Option<usize>,
  /// 相关性检索用的查询（通常是用户最后一条消息）
  pub query: Option<String>,
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

fn is_cjk(c: char) -> bool {
  matches!(c as u32, 0x4E00..=0x9FFF)
}

/// 从查询抽取检索词：ASCII 单词（>=2 字符）+ 每段连续 CJK 的双字组合
fn query_terms(query: &str) -> Vec<String> {
  let lower = query.to_lowercase();
  let mut terms: Vec<String> = Vec::new();
  let push = |t: String, terms: &mut Vec<String>| {
    if !terms.iter().any(|x| *x == t) {
      terms.push(t);
    }
  };

  for w in lower.split(|c: char| !c.is_ascii_alphanumeric()) {
    if w.len() >= 2 {
      push(w.to_string(), &mut terms);
    }
  }

  // 逐段连续 CJK 取二元组，避免跨词误配
  let mut run: Vec<char> = Vec::new();
  for c in lower.chars().chain(std::iter::once(' ')) {
    if is_cjk(c) {
      run.push(c);
    } else {
      for pair in run.windows(2) {
        push(pair.iter().collect::<String>(), &mut terms);
      }
      run.clear();
    }
  }

  terms
}

/// 工具与检索词的相关度：命中名称权重最高，其次分组，再次描述
fn relevance(decl: &ToolDeclaration, terms: &[String]) -> u32 {
  if terms.is_empty() {
    return 0;
  }
  let name = decl.name.to_lowercase();
  let group = decl.group.to_lowercase();
  let desc = decl.description.to_lowercase();
  let mut score = 0;
  for t in terms {
    if name.contains(t.as_str()) {
      score += 3;
    }
    if group.contains(t.as_str()) {
      score += 2;
    }
    if desc.contains(t.as_str()) {
      score += 1;
    }
  }
  score
}

/// 按 (相关性降序, 名称升序) 排序并按上限截断；无 query 时退化为稳定的名称序
fn select_relevant(
  mut decls: Vec<ToolDeclaration>,
  query: Option<&str>,
  max: Option<usize>,
) -> Vec<ToolDeclaration> {
  let terms = query.map(query_terms).unwrap_or_default();
  decls.sort_by(|a, b| {
    relevance(b, &terms)
      .cmp(&relevance(a, &terms))
      .then_with(|| a.name.cmp(&b.name))
  });
  if let Some(m) = max {
    decls.truncate(m);
  }
  decls
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

  /// 按过滤条件列出工具声明（注入给 LLM）。
  /// 候选超过 max_tools 时按与 query 的相关性取 top-K（§6.1）。
  pub fn declarations(&self, filter: &ToolFilter) -> Vec<ToolDeclaration> {
    let matched: Vec<ToolDeclaration> = match self.tools.read() {
      Ok(map) => map
        .values()
        .map(|t| t.declaration())
        .filter(|d| filter.accepts(d))
        .collect(),
      Err(_) => Vec::new(),
    };
    select_relevant(matched, filter.query.as_deref(), filter.max_tools)
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

  fn decl_of(name: &str, group: &str, desc: &str) -> ToolDeclaration {
    ToolDeclaration {
      name: name.into(),
      description: desc.into(),
      input_schema: serde_json::json!({ "type": "object" }),
      risk: RiskLevel::Read,
      source: ToolSource::Native,
      group: group.into(),
    }
  }

  #[test]
  fn test_query_terms_ascii_and_cjk() {
    let t = query_terms("add a Hosts entry");
    assert!(t.contains(&"add".to_string()));
    assert!(t.contains(&"hosts".to_string()));
    assert!(!t.contains(&"a".to_string())); // 单字符忽略

    let z = query_terms("设置壁纸");
    assert!(z.contains(&"壁纸".to_string()));
    // 不跨非 CJK 边界组词
    let split = query_terms("壁纸 播客");
    assert!(!split.contains(&"纸播".to_string()));
  }

  #[test]
  fn test_relevance_ranks_matching_first() {
    let decls = vec![
      decl_of("podcast_list", "podcast", "列出播客"),
      decl_of("hosts_add", "hosts", "添加一条 Hosts 记录"),
      decl_of("wallpaper_set", "wallpaper", "设置壁纸"),
    ];
    let out = select_relevant(decls, Some("帮我加一条 hosts"), None);
    assert_eq!(out[0].name, "hosts_add");
  }

  #[test]
  fn test_relevance_ranks_cjk_query() {
    let decls = vec![
      decl_of("hosts_add", "hosts", "添加一条 Hosts 记录"),
      decl_of("wallpaper_set", "wallpaper", "设置壁纸"),
    ];
    let out = select_relevant(decls, Some("换个壁纸"), None);
    assert_eq!(out[0].name, "wallpaper_set");
  }

  #[test]
  fn test_max_tools_truncates_and_is_deterministic() {
    let decls = vec![
      decl_of("b_tool", "g", ""),
      decl_of("a_tool", "g", ""),
      decl_of("c_tool", "g", ""),
    ];
    // 无 query → 名称序，截断到 2
    let out = select_relevant(decls.clone(), None, Some(2));
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].name, "a_tool");
    assert_eq!(out[1].name, "b_tool");
    // 未超上限则全保留
    assert_eq!(select_relevant(decls, None, Some(10)).len(), 3);
  }

  #[test]
  fn test_registry_declarations_respect_max_tools() {
    let reg = ToolRegistry::with_builtins();
    let base = ToolFilter {
      enabled_groups: vec!["hosts".into()],
      ..Default::default()
    };
    let all = reg.declarations(&base);
    assert!(all.len() > 2);

    let capped = reg.declarations(&ToolFilter {
      max_tools: Some(2),
      query: Some("添加 hosts".into()),
      ..base
    });
    assert_eq!(capped.len(), 2);
    // 相关性最高的应被保留
    assert!(capped.iter().any(|d| d.name == "hosts_add"));
  }

  #[test]
  fn test_filter_by_group() {
    let reg = ToolRegistry::with_builtins();
    let empty = reg.declarations(&ToolFilter::default());
    assert!(empty.is_empty(), "空过滤器不应注入任何工具");

    let hosts = reg.declarations(&ToolFilter {
      enabled_groups: vec!["hosts".into()],
      ..Default::default()
    });
    assert!(!hosts.is_empty());
    assert!(hosts.iter().all(|d| d.group == "hosts"));
  }
}
