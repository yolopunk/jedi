// Hosts 内置工具：复用 api::hosts 的读取逻辑，写入走本地共享 helper
// 由 mcp/servers/hosts.rs 的伪 MCP 实现迁移而来（P1）

use crate::api::hosts::{read_system_hosts, GroupHosts, HostEntry, HOSTS_PATH};
use crate::tools::{AgentTool, RiskLevel, ToolDeclaration, ToolOutcome, ToolSource};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

const GROUP: &str = "hosts";

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

/// 写入完整 hosts 配置（保留非 Jedi 管理区，重写 Jedi 区）
/// 迁移自 mcp/servers/hosts.rs 的 write_hosts。
fn write_groups(groups: &[GroupHosts]) -> Result<(), String> {
  let hosts_content =
    std::fs::read_to_string(HOSTS_PATH).map_err(|e| format!("读取 hosts 失败: {}", e))?;

  let mut new_lines: Vec<String> = Vec::new();
  let mut in_jedi = false;

  for line in hosts_content.lines() {
    let trimmed = line.trim_start();
    if trimmed.starts_with("# === JEDI HOSTS MANAGER ===") {
      in_jedi = true;
      continue;
    }
    if trimmed.starts_with("# === END JEDI HOSTS MANAGER ===") {
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

  let content = new_lines.join("\n") + "\n";
  std::fs::write(HOSTS_PATH, content).map_err(|e| format!("写入 hosts 失败: {}", e))?;
  Ok(())
}

// ============================================================================
// hosts_read
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
// hosts_list
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
// hosts_add
// ============================================================================

pub struct HostsAdd;

fn add_impl(args: Value) -> Result<String, String> {
  let ip = get_str(&args, "ip")?;
  let domain = get_str(&args, "domain")?;
  let group = get_str(&args, "group")?;
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

  write_groups(&groups)?;
  Ok(format!(
    "已添加: {} {} → {}（禁用: {}）",
    ip, domain, group, disabled
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

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    match add_impl(args) {
      Ok(msg) => ToolOutcome::text(msg),
      Err(e) => ToolOutcome::error(e),
    }
  }
}

// ============================================================================
// hosts_remove
// ============================================================================

pub struct HostsRemove;

fn remove_impl(args: Value) -> Result<String, String> {
  let domain = get_str(&args, "domain")?;
  let group_filter = args.get("group").and_then(|v| v.as_str());

  let mut groups = read_system_hosts().map_err(|e| format!("读取 hosts 失败: {}", e))?;

  let mut removed_from = Vec::new();
  for group in groups.iter_mut() {
    if let Some(filter) = group_filter {
      if group.name != filter {
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

  write_groups(&groups)?;
  Ok(format!(
    "已从分组 [{}] 删除 '{}'",
    removed_from.join(", "),
    domain
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

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    match remove_impl(args) {
      Ok(msg) => ToolOutcome::text(msg),
      Err(e) => ToolOutcome::error(e),
    }
  }
}

// ============================================================================
// hosts_toggle
// ============================================================================

pub struct HostsToggle;

fn toggle_impl(args: Value) -> Result<String, String> {
  let domain = get_str(&args, "domain")?;
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

  write_groups(&groups)?;
  Ok(format!("已切换 '{}':\n{}", domain, changes.join("\n")))
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

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    match toggle_impl(args) {
      Ok(msg) => ToolOutcome::text(msg),
      Err(e) => ToolOutcome::error(e),
    }
  }
}

// ============================================================================
// hosts_write
// ============================================================================

pub struct HostsWrite;

fn write_impl(args: Value) -> Result<String, String> {
  let groups_value = args
    .get("groups")
    .ok_or_else(|| "缺少参数: groups".to_string())?;
  let groups: Vec<GroupHosts> =
    serde_json::from_value(groups_value.clone()).map_err(|e| format!("groups 格式错误: {}", e))?;

  write_groups(&groups)?;
  let total: usize = groups.iter().map(|g| g.hosts.len()).sum();
  Ok(format!("已写入 {} 个分组，共 {} 条记录", groups.len(), total))
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

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    match write_impl(args) {
      Ok(msg) => ToolOutcome::text(msg),
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

  #[test]
  fn test_tools_count_and_names() {
    let names: Vec<String> = tools()
      .iter()
      .map(|t| t.declaration().name)
      .collect();
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
    assert!(required.iter().any(|x| x == "domain"));
    assert!(required.iter().any(|x| x == "group"));
    assert_eq!(d.risk, RiskLevel::Write);
  }

  #[test]
  fn test_read_is_read_risk() {
    assert_eq!(HostsRead.declaration().risk, RiskLevel::Read);
    assert_eq!(HostsList.declaration().risk, RiskLevel::Read);
  }

  #[test]
  fn test_add_impl_missing_param() {
    let err = add_impl(json!({ "ip": "127.0.0.1" })).unwrap_err();
    assert!(err.contains("缺少参数"));
  }
}
