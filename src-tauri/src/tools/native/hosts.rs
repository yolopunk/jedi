// Hosts 内置工具：复用 api::hosts 读取逻辑，写入走本地共享 helper
// P1 迁移自伪 MCP；P2 增加 dynamic_risk / dry_run+快照 / undo

use crate::api::hosts::{read_system_hosts, GroupHosts, HostEntry, HOSTS_PATH};
use crate::tools::{AgentTool, RiskLevel, ToolDeclaration, ToolOutcome, ToolPreview, ToolSource};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

const GROUP: &str = "hosts";

/// 命中这些域名（或其子域）的写操作升级为 System 风险（强制二次确认）
const SENSITIVE_SUFFIXES: &[&str] = &[
  "microsoft.com",
  "windowsupdate.com",
  "apple.com",
  "icloud.com",
  "google.com",
  "googleapis.com",
  "github.com",
  "amazonaws.com",
];

/// 回滚快照：token → 写入前的 hosts 完整内容
static UNDO_SNAPSHOTS: LazyLock<Mutex<HashMap<String, String>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));
static UNDO_COUNTER: AtomicU64 = AtomicU64::new(1);

/// 全部 hosts 工具
pub fn tools() -> Vec<Arc<dyn AgentTool>> {
  vec![
    Arc::new(HostsRead),
    Arc::new(HostsList),
    Arc::new(HostsAdd),
    Arc::new(HostsRemove),
    Arc::new(HostsToggle),
    Arc::new(HostsWrite),
  ]
}

fn decl(name: &str, description: &str, schema: Value, risk: RiskLevel) -> ToolDeclaration {
  ToolDeclaration {
    name: name.into(),
    description: description.into(),
    input_schema: schema,
    risk,
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

fn is_sensitive(domain: &str) -> bool {
  let d = domain.trim().to_ascii_lowercase();
  SENSITIVE_SUFFIXES
    .iter()
    .any(|s| d == *s || d.ends_with(&format!(".{}", s)))
}

// ============================================================================
// 快照 / 渲染 / 写入 / 回滚
// ============================================================================

fn hash_str(s: &str) -> String {
  let mut h = DefaultHasher::new();
  s.hash(&mut h);
  format!("{:016x}", h.finish())
}

/// 当前 hosts 文件的一致性快照 token（【Y1】）
fn hosts_snapshot() -> String {
  hash_str(&std::fs::read_to_string(HOSTS_PATH).unwrap_or_default())
}

/// 用 groups 渲染完整 hosts 内容（保留 base 中的非 Jedi 区）
fn render_hosts(base: &str, groups: &[GroupHosts]) -> String {
  let mut new_lines: Vec<String> = Vec::new();
  let mut in_jedi = false;
  for line in base.lines() {
    let t = line.trim_start();
    if t.starts_with("# === JEDI HOSTS MANAGER ===") {
      in_jedi = true;
      continue;
    }
    if t.starts_with("# === END JEDI HOSTS MANAGER ===") {
      in_jedi = false;
      continue;
    }
    if !in_jedi {
      new_lines.push(line.to_string());
    }
  }
  new_lines.push("# === JEDI HOSTS MANAGER ===".to_string());
  for g in groups {
    new_lines.push(format!("# +{}+", g.name));
    let mut sorted = g.hosts.clone();
    sorted.sort_by(|a, b| a.domain.cmp(&b.domain));
    for host in sorted {
      if host.disabled {
        new_lines.push(format!("# {} {}", host.ip, host.domain));
      } else {
        new_lines.push(format!("{} {}", host.ip, host.domain));
      }
    }
  }
  new_lines.push("# === END JEDI HOSTS MANAGER ===".to_string());
  new_lines.join("\n") + "\n"
}

/// 写入 groups。expected 为确认时锁定的快照：若与当前不一致则拒绝（【Y1】）。
/// 成功返回 undo_token（写入前内容已入回滚栈）。
fn write_groups(groups: &[GroupHosts], expected: Option<String>) -> Result<String, String> {
  let base = std::fs::read_to_string(HOSTS_PATH).map_err(|e| format!("读取 hosts 失败: {}", e))?;
  if let Some(exp) = &expected {
    if *exp != hash_str(&base) {
      return Err("目标 hosts 已被外部修改，请重新确认后再执行".to_string());
    }
  }
  let content = render_hosts(&base, groups);
  std::fs::write(HOSTS_PATH, &content).map_err(|e| format!("写入 hosts 失败: {}", e))?;

  let token = format!("hosts-{}", UNDO_COUNTER.fetch_add(1, Ordering::Relaxed));
  if let Ok(mut map) = UNDO_SNAPSHOTS.lock() {
    map.insert(token.clone(), base);
  }
  Ok(token)
}

/// 回滚到某个 undo_token 对应的写入前内容（【Y4】）
fn restore_hosts(token: &str) -> Result<(), String> {
  let content = {
    let map = UNDO_SNAPSHOTS.lock().map_err(|e| e.to_string())?;
    map.get(token).cloned()
  }
  .ok_or_else(|| "快照不存在或已过期".to_string())?;
  std::fs::write(HOSTS_PATH, content).map_err(|e| format!("写入 hosts 失败: {}", e))?;
  Ok(())
}

fn preview(diff: String) -> Result<Option<ToolPreview>, String> {
  Ok(Some(ToolPreview {
    diff,
    snapshot_token: hosts_snapshot(),
  }))
}

// ============================================================================
// hosts_read（Read）
// ============================================================================

pub struct HostsRead;

#[async_trait]
impl AgentTool for HostsRead {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "hosts_read",
      "读取系统 Hosts 文件内容，返回所有分组和条目",
      json!({ "type": "object", "properties": {} }),
      RiskLevel::Read,
    )
  }

  async fn call(&self, _args: Value, _snapshot: Option<String>) -> ToolOutcome {
    match read_system_hosts() {
      Ok(groups) => match serde_json::to_string_pretty(&groups) {
        Ok(s) => ToolOutcome::text(s),
        Err(e) => ToolOutcome::error(format!("序列化失败: {}", e)),
      },
      Err(e) => ToolOutcome::error(format!("读取 hosts 失败: {}", e)),
    }
  }
}

// ============================================================================
// hosts_list（Read）
// ============================================================================

pub struct HostsList;

#[async_trait]
impl AgentTool for HostsList {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "hosts_list",
      "列出所有 Hosts 条目，可按分组筛选",
      json!({
        "type": "object",
        "properties": {
          "group": { "type": "string", "description": "分组名称（可选，不指定返回全部）" }
        }
      }),
      RiskLevel::Read,
    )
  }

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let groups = match read_system_hosts() {
      Ok(g) => g,
      Err(e) => return ToolOutcome::error(format!("读取 hosts 失败: {}", e)),
    };
    let filtered: Vec<GroupHosts> = match args.get("group").and_then(|v| v.as_str()) {
      Some(name) => groups.into_iter().filter(|g| g.name == name).collect(),
      None => groups,
    };
    match serde_json::to_string_pretty(&filtered) {
      Ok(s) => ToolOutcome::text(s),
      Err(e) => ToolOutcome::error(format!("序列化失败: {}", e)),
    }
  }
}

// ============================================================================
// hosts_add（Write / 命中敏感域名→System）
// ============================================================================

pub struct HostsAdd;

fn add_apply(args: &Value, expected: Option<String>) -> Result<(String, String), String> {
  let ip = get_str(args, "ip")?;
  let domain = get_str(args, "domain")?;
  let group = get_str(args, "group")?;
  let disabled = args.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);

  let mut groups = read_system_hosts().map_err(|e| format!("读取 hosts 失败: {}", e))?;
  let target = match groups.iter().position(|g| g.name == group) {
    Some(idx) => &mut groups[idx],
    None => {
      groups.push(GroupHosts {
        name: group.clone(),
        hosts: Vec::new(),
      });
      groups.last_mut().unwrap()
    }
  };
  if target.hosts.iter().any(|h| h.domain == domain) {
    return Err(format!("域名 '{}' 已存在于分组 '{}'", domain, group));
  }
  target.hosts.push(HostEntry {
    ip: ip.clone(),
    domain: domain.clone(),
    disabled,
  });

  let token = write_groups(&groups, expected)?;
  Ok((
    format!("已添加: {} {} → {}（禁用: {}）", ip, domain, group, disabled),
    token,
  ))
}

#[async_trait]
impl AgentTool for HostsAdd {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "hosts_add",
      "添加一条 Hosts 记录到指定分组",
      json!({
        "type": "object",
        "properties": {
          "ip": { "type": "string", "description": "IP 地址，如 127.0.0.1" },
          "domain": { "type": "string", "description": "域名，如 example.com" },
          "group": { "type": "string", "description": "分组名称，不存在则创建" },
          "disabled": { "type": "boolean", "description": "是否禁用，默认 false" }
        },
        "required": ["ip", "domain", "group"]
      }),
      RiskLevel::Write,
    )
  }

  fn dynamic_risk(&self, args: &Value) -> RiskLevel {
    match args.get("domain").and_then(|v| v.as_str()) {
      Some(d) if is_sensitive(d) => RiskLevel::System,
      _ => RiskLevel::Write,
    }
  }

  async fn dry_run(&self, args: &Value) -> Result<Option<ToolPreview>, String> {
    let ip = get_str(args, "ip")?;
    let domain = get_str(args, "domain")?;
    let group = get_str(args, "group")?;
    let disabled = args.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let groups = read_system_hosts().map_err(|e| format!("读取 hosts 失败: {}", e))?;
    let exists = groups
      .iter()
      .find(|g| g.name == group)
      .map(|g| g.hosts.iter().any(|h| h.domain == domain))
      .unwrap_or(false);
    let diff = if exists {
      format!("（无变化）{} 已存在于分组「{}」", domain, group)
    } else {
      format!(
        "+ {} {}  →  分组「{}」{}",
        ip,
        domain,
        group,
        if disabled { "  [禁用]" } else { "" }
      )
    };
    preview(diff)
  }

  async fn call(&self, args: Value, snapshot: Option<String>) -> ToolOutcome {
    match add_apply(&args, snapshot) {
      Ok((msg, token)) => ToolOutcome::text(msg).with_undo(token),
      Err(e) => ToolOutcome::error(e),
    }
  }

  async fn undo(&self, token: &str) -> Result<(), String> {
    restore_hosts(token)
  }
}

// ============================================================================
// hosts_remove（Write）
// ============================================================================

pub struct HostsRemove;

fn remove_apply(args: &Value, expected: Option<String>) -> Result<(String, String), String> {
  let domain = get_str(args, "domain")?;
  let group_filter = args.get("group").and_then(|v| v.as_str());
  let mut groups = read_system_hosts().map_err(|e| format!("读取 hosts 失败: {}", e))?;
  let mut removed_from = Vec::new();
  for group in groups.iter_mut() {
    if let Some(f) = group_filter {
      if group.name != f {
        continue;
      }
    }
    let before = group.hosts.len();
    group.hosts.retain(|h| h.domain != domain);
    if group.hosts.len() < before {
      removed_from.push(group.name.clone());
    }
  }
  if removed_from.is_empty() {
    return Err(format!("未找到域名 '{}'", domain));
  }
  let token = write_groups(&groups, expected)?;
  Ok((
    format!("已从分组 [{}] 删除 '{}'", removed_from.join(", "), domain),
    token,
  ))
}

#[async_trait]
impl AgentTool for HostsRemove {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "hosts_remove",
      "删除指定的 Hosts 条目",
      json!({
        "type": "object",
        "properties": {
          "domain": { "type": "string", "description": "要删除的域名" },
          "group": { "type": "string", "description": "分组名称（可选，不指定搜索全部）" }
        },
        "required": ["domain"]
      }),
      RiskLevel::Write,
    )
  }

  async fn dry_run(&self, args: &Value) -> Result<Option<ToolPreview>, String> {
    let domain = get_str(args, "domain")?;
    let groups = read_system_hosts().map_err(|e| format!("读取 hosts 失败: {}", e))?;
    let hits: Vec<String> = groups
      .iter()
      .filter(|g| g.hosts.iter().any(|h| h.domain == domain))
      .map(|g| g.name.clone())
      .collect();
    let diff = if hits.is_empty() {
      format!("（无变化）未找到 {}", domain)
    } else {
      format!("- {}  （来自分组：{}）", domain, hits.join(", "))
    };
    preview(diff)
  }

  async fn call(&self, args: Value, snapshot: Option<String>) -> ToolOutcome {
    match remove_apply(&args, snapshot) {
      Ok((msg, token)) => ToolOutcome::text(msg).with_undo(token),
      Err(e) => ToolOutcome::error(e),
    }
  }

  async fn undo(&self, token: &str) -> Result<(), String> {
    restore_hosts(token)
  }
}

// ============================================================================
// hosts_toggle（Write / 命中敏感域名→System）
// ============================================================================

pub struct HostsToggle;

fn toggle_apply(args: &Value, expected: Option<String>) -> Result<(String, String), String> {
  let domain = get_str(args, "domain")?;
  let target_disabled = args.get("disabled").and_then(|v| v.as_bool());
  let mut groups = read_system_hosts().map_err(|e| format!("读取 hosts 失败: {}", e))?;
  let mut changes = Vec::new();
  for group in groups.iter_mut() {
    for host in group.hosts.iter_mut() {
      if host.domain == domain {
        let old = host.disabled;
        let new = target_disabled.unwrap_or(!old);
        host.disabled = new;
        changes.push(format!("{}: {} → {}", group.name, old, new));
      }
    }
  }
  if changes.is_empty() {
    return Err(format!("未找到域名 '{}'", domain));
  }
  let token = write_groups(&groups, expected)?;
  Ok((format!("已切换 '{}':\n{}", domain, changes.join("\n")), token))
}

#[async_trait]
impl AgentTool for HostsToggle {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "hosts_toggle",
      "切换 Hosts 条目的启用/禁用状态",
      json!({
        "type": "object",
        "properties": {
          "domain": { "type": "string", "description": "要切换的域名" },
          "disabled": { "type": "boolean", "description": "目标状态：true=禁用/false=启用（可选，不指定则翻转）" }
        },
        "required": ["domain"]
      }),
      RiskLevel::Write,
    )
  }

  fn dynamic_risk(&self, args: &Value) -> RiskLevel {
    match args.get("domain").and_then(|v| v.as_str()) {
      Some(d) if is_sensitive(d) => RiskLevel::System,
      _ => RiskLevel::Write,
    }
  }

  async fn dry_run(&self, args: &Value) -> Result<Option<ToolPreview>, String> {
    let domain = get_str(args, "domain")?;
    preview(format!("~ 切换 {} 的启用/禁用状态", domain))
  }

  async fn call(&self, args: Value, snapshot: Option<String>) -> ToolOutcome {
    match toggle_apply(&args, snapshot) {
      Ok((msg, token)) => ToolOutcome::text(msg).with_undo(token),
      Err(e) => ToolOutcome::error(e),
    }
  }

  async fn undo(&self, token: &str) -> Result<(), String> {
    restore_hosts(token)
  }
}

// ============================================================================
// hosts_write（Write / 命中敏感域名→System）
// ============================================================================

pub struct HostsWrite;

fn write_apply(args: &Value, expected: Option<String>) -> Result<(String, String), String> {
  let groups_value = args
    .get("groups")
    .ok_or_else(|| "缺少参数: groups".to_string())?;
  let groups: Vec<GroupHosts> =
    serde_json::from_value(groups_value.clone()).map_err(|e| format!("groups 格式错误: {}", e))?;
  let token = write_groups(&groups, expected)?;
  let total: usize = groups.iter().map(|g| g.hosts.len()).sum();
  Ok((
    format!("已写入 {} 个分组，共 {} 条记录", groups.len(), total),
    token,
  ))
}

#[async_trait]
impl AgentTool for HostsWrite {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "hosts_write",
      "写入完整的 Hosts 配置（替换所有分组）",
      json!({
        "type": "object",
        "properties": {
          "groups": {
            "type": "array",
            "description": "分组列表",
            "items": {
              "type": "object",
              "properties": {
                "name": { "type": "string" },
                "hosts": {
                  "type": "array",
                  "items": {
                    "type": "object",
                    "properties": {
                      "ip": { "type": "string" },
                      "domain": { "type": "string" },
                      "disabled": { "type": "boolean" }
                    },
                    "required": ["ip", "domain"]
                  }
                }
              },
              "required": ["name", "hosts"]
            }
          }
        },
        "required": ["groups"]
      }),
      RiskLevel::Write,
    )
  }

  fn dynamic_risk(&self, args: &Value) -> RiskLevel {
    let sensitive = args
      .get("groups")
      .and_then(|v| v.as_array())
      .map(|groups| {
        groups.iter().any(|g| {
          g.get("hosts")
            .and_then(|h| h.as_array())
            .map(|hosts| {
              hosts.iter().any(|h| {
                h.get("domain")
                  .and_then(|d| d.as_str())
                  .map(is_sensitive)
                  .unwrap_or(false)
              })
            })
            .unwrap_or(false)
        })
      })
      .unwrap_or(false);
    if sensitive {
      RiskLevel::System
    } else {
      RiskLevel::Write
    }
  }

  async fn dry_run(&self, args: &Value) -> Result<Option<ToolPreview>, String> {
    let groups_value = args
      .get("groups")
      .ok_or_else(|| "缺少参数: groups".to_string())?;
    let groups: Vec<GroupHosts> = serde_json::from_value(groups_value.clone())
      .map_err(|e| format!("groups 格式错误: {}", e))?;
    let total: usize = groups.iter().map(|g| g.hosts.len()).sum();
    preview(format!(
      "! 覆盖全部 Jedi 分组 → {} 个分组、{} 条记录",
      groups.len(),
      total
    ))
  }

  async fn call(&self, args: Value, snapshot: Option<String>) -> ToolOutcome {
    match write_apply(&args, snapshot) {
      Ok((msg, token)) => ToolOutcome::text(msg).with_undo(token),
      Err(e) => ToolOutcome::error(e),
    }
  }

  async fn undo(&self, token: &str) -> Result<(), String> {
    restore_hosts(token)
  }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tools_count_and_names() {
    let names: Vec<String> = tools().iter().map(|t| t.declaration().name).collect();
    assert_eq!(names.len(), 6);
    for expected in [
      "hosts_read",
      "hosts_list",
      "hosts_add",
      "hosts_remove",
      "hosts_toggle",
      "hosts_write",
    ] {
      assert!(names.contains(&expected.to_string()), "缺少 {}", expected);
    }
  }

  #[test]
  fn test_add_schema_required() {
    let d = HostsAdd.declaration();
    let required = d.input_schema["required"].as_array().unwrap();
    assert!(required.iter().any(|x| x == "ip"));
    assert_eq!(d.risk, RiskLevel::Write);
  }

  #[test]
  fn test_sensitive_domain_escalates_risk() {
    assert!(is_sensitive("update.microsoft.com"));
    assert!(is_sensitive("MICROSOFT.COM"));
    assert!(!is_sensitive("example.com"));

    let esc = HostsAdd.dynamic_risk(&json!({ "domain": "foo.microsoft.com" }));
    assert_eq!(esc, RiskLevel::System);
    let normal = HostsAdd.dynamic_risk(&json!({ "domain": "test.local" }));
    assert_eq!(normal, RiskLevel::Write);
  }

  #[test]
  fn test_render_hosts_preserves_non_jedi() {
    let base = "127.0.0.1 keep.local\n# === JEDI HOSTS MANAGER ===\n# +old+\n1.1.1.1 old.test\n# === END JEDI HOSTS MANAGER ===\n";
    let groups = vec![GroupHosts {
      name: "dev".into(),
      hosts: vec![HostEntry {
        ip: "127.0.0.1".into(),
        domain: "new.test".into(),
        disabled: false,
      }],
    }];
    let out = render_hosts(base, &groups);
    assert!(out.contains("127.0.0.1 keep.local"));
    assert!(out.contains("# +dev+"));
    assert!(out.contains("127.0.0.1 new.test"));
    assert!(!out.contains("old.test"));
  }

  #[test]
  fn test_hash_changes_with_content() {
    assert_ne!(hash_str("a"), hash_str("b"));
    assert_eq!(hash_str("a"), hash_str("a"));
  }

  #[test]
  fn test_restore_missing_token() {
    assert!(restore_hosts("nonexistent-token").is_err());
  }
}
