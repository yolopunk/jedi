// Hosts MCP Server 实现
// Phase 3: Hosts MCP Server - 提供 Hosts 文件管理工具

use crate::api::hosts::{read_system_hosts, GroupHosts, HostEntry, HOSTS_PATH};
use crate::mcp::types::*;
use std::collections::HashMap;
use tracing::{debug, info};

// ============================================================================
// Hosts MCP Server 定义
// ============================================================================

/// Hosts MCP Server
///
/// 提供 Hosts 文件管理的 MCP 工具
#[allow(dead_code)]
pub struct HostsMcpServer {
  /// 服务器状态
  state: ServerState,
}

/// 服务器状态
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServerState {
  /// 未初始化
  Uninitialized,
  /// 已初始化
  Initialized,
}

impl Default for HostsMcpServer {
  fn default() -> Self {
    Self::new()
  }
}

#[allow(dead_code)]
impl HostsMcpServer {
  /// 创建新的 Hosts MCP Server
  pub fn new() -> Self {
    Self {
      state: ServerState::Uninitialized,
    }
  }

  /// 初始化服务器
  pub fn initialize(&mut self) -> Result<(), McpError> {
    info!("Initializing Hosts MCP Server");
    self.state = ServerState::Initialized;
    Ok(())
  }

  /// 获取工具列表
  pub fn list_tools(&self) -> Result<Vec<Tool>, McpError> {
    self.ensure_initialized()?;

    debug!("Listing Hosts MCP tools");

    let tools = vec![
      // hosts_read - 读取 Hosts 文件
      Tool {
        name: "hosts_read".to_string(),
        description: Some("读取系统 Hosts 文件内容，返回所有分组和条目".to_string()),
        input_schema: ToolInputSchema::new(),
      },
      // hosts_list - 列出所有条目
      Tool {
        name: "hosts_list".to_string(),
        description: Some("列出所有 Hosts 条目，可以按分组筛选".to_string()),
        input_schema: ToolInputSchema::new().with_properties(HashMap::from([(
          "group".to_string(),
          serde_json::json!({
              "type": "string",
              "description": "分组名称（可选，不指定则返回所有分组）"
          }),
        )])),
      },
      // hosts_add - 添加条目
      Tool {
        name: "hosts_add".to_string(),
        description: Some("添加新的 Hosts 条目到指定分组".to_string()),
        input_schema: ToolInputSchema::new()
          .with_properties(HashMap::from([
            (
              "ip".to_string(),
              serde_json::json!({
                  "type": "string",
                  "description": "IP 地址，例如: 127.0.0.1"
              }),
            ),
            (
              "domain".to_string(),
              serde_json::json!({
                  "type": "string",
                  "description": "域名，例如: example.com"
              }),
            ),
            (
              "group".to_string(),
              serde_json::json!({
                  "type": "string",
                  "description": "分组名称，如果不存在则创建新分组"
              }),
            ),
            (
              "disabled".to_string(),
              serde_json::json!({
                  "type": "boolean",
                  "description": "是否禁用该条目，默认为 false"
              }),
            ),
          ]))
          .with_required(vec![
            "ip".to_string(),
            "domain".to_string(),
            "group".to_string(),
          ]),
      },
      // hosts_remove - 删除条目
      Tool {
        name: "hosts_remove".to_string(),
        description: Some("删除指定的 Hosts 条目".to_string()),
        input_schema: ToolInputSchema::new()
          .with_properties(HashMap::from([
            (
              "domain".to_string(),
              serde_json::json!({
                  "type": "string",
                  "description": "要删除的域名"
              }),
            ),
            (
              "group".to_string(),
              serde_json::json!({
                  "type": "string",
                  "description": "分组名称（可选，不指定则搜索所有分组）"
              }),
            ),
          ]))
          .with_required(vec!["domain".to_string()]),
      },
      // hosts_toggle - 启用/禁用条目
      Tool {
        name: "hosts_toggle".to_string(),
        description: Some("切换 Hosts 条目的启用/禁用状态".to_string()),
        input_schema: ToolInputSchema::new()
          .with_properties(HashMap::from([
            (
              "domain".to_string(),
              serde_json::json!({
                  "type": "string",
                  "description": "要切换的域名"
              }),
            ),
            (
              "disabled".to_string(),
              serde_json::json!({
                  "type": "boolean",
                  "description": "目标状态：true=禁用，false=启用（可选，不指定则切换当前状态）"
              }),
            ),
          ]))
          .with_required(vec!["domain".to_string()]),
      },
      // hosts_write - 写入完整 Hosts 配置
      Tool {
        name: "hosts_write".to_string(),
        description: Some("写入完整的 Hosts 配置（替换所有分组）".to_string()),
        input_schema: ToolInputSchema::new()
          .with_properties(HashMap::from([(
            "groups".to_string(),
            serde_json::json!({
                "type": "array",
                "description": "分组列表，每个分组包含名称和 hosts 条目",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "分组名称" },
                        "hosts": {
                            "type": "array",
                            "description": "Hosts 条目列表",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "ip": { "type": "string", "description": "IP 地址" },
                                    "domain": { "type": "string", "description": "域名" },
                                    "disabled": { "type": "boolean", "description": "是否禁用" }
                                },
                                "required": ["ip", "domain"]
                            }
                        }
                    },
                    "required": ["name", "hosts"]
                }
            }),
          )]))
          .with_required(vec!["groups".to_string()]),
      },
    ];

    Ok(tools)
  }

  /// 调用工具
  pub fn call_tool(
    &self,
    name: &str,
    arguments: Option<HashMap<String, serde_json::Value>>,
  ) -> Result<CallToolResult, McpError> {
    self.ensure_initialized()?;

    info!(tool = %name, "Calling Hosts MCP tool");

    let args = arguments.unwrap_or_default();

    match name {
      "hosts_read" => self.handle_hosts_read(),
      "hosts_list" => self.handle_hosts_list(&args),
      "hosts_add" => self.handle_hosts_add(&args),
      "hosts_remove" => self.handle_hosts_remove(&args),
      "hosts_toggle" => self.handle_hosts_toggle(&args),
      "hosts_write" => self.handle_hosts_write(&args),
      _ => Err(McpError::MethodNotFound(name.to_string())),
    }
  }

  // ========================================================================
  // 工具处理函数
  // ========================================================================

  /// 处理 hosts_read 工具
  fn handle_hosts_read(&self) -> Result<CallToolResult, McpError> {
    let groups = read_system_hosts()
      .map_err(|e| McpError::Protocol(format!("Failed to read hosts: {}", e)))?;

    let result = serde_json::to_string_pretty(&groups)
      .map_err(|e| McpError::Serialization(format!("Failed to serialize hosts: {}", e)))?;

    Ok(CallToolResult {
      content: vec![Content::text(result)],
      is_error: Some(false),
    })
  }

  /// 处理 hosts_list 工具
  fn handle_hosts_list(
    &self,
    args: &HashMap<String, serde_json::Value>,
  ) -> Result<CallToolResult, McpError> {
    let groups = read_system_hosts()
      .map_err(|e| McpError::Protocol(format!("Failed to read hosts: {}", e)))?;

    let filtered_groups = if let Some(group_name) = args.get("group").and_then(|v| v.as_str()) {
      groups
        .into_iter()
        .filter(|g| g.name == group_name)
        .collect::<Vec<_>>()
    } else {
      groups
    };

    let result = serde_json::to_string_pretty(&filtered_groups)
      .map_err(|e| McpError::Serialization(format!("Failed to serialize hosts: {}", e)))?;

    Ok(CallToolResult {
      content: vec![Content::text(result)],
      is_error: Some(false),
    })
  }

  /// 处理 hosts_add 工具
  fn handle_hosts_add(
    &self,
    args: &HashMap<String, serde_json::Value>,
  ) -> Result<CallToolResult, McpError> {
    let ip = args
      .get("ip")
      .and_then(|v| v.as_str())
      .ok_or_else(|| McpError::InvalidParams("Missing ip parameter".to_string()))?;

    let domain = args
      .get("domain")
      .and_then(|v| v.as_str())
      .ok_or_else(|| McpError::InvalidParams("Missing domain parameter".to_string()))?;

    let group = args
      .get("group")
      .and_then(|v| v.as_str())
      .ok_or_else(|| McpError::InvalidParams("Missing group parameter".to_string()))?;

    let disabled = args
      .get("disabled")
      .and_then(|v| v.as_bool())
      .unwrap_or(false);

    // 读取当前 hosts
    let mut groups = read_system_hosts()
      .map_err(|e| McpError::Protocol(format!("Failed to read hosts: {}", e)))?;

    // 查找或创建分组
    let target_index = groups.iter().position(|g| g.name == group);
    let target_group = if let Some(index) = target_index {
      &mut groups[index]
    } else {
      groups.push(GroupHosts {
        name: group.to_string(),
        hosts: Vec::new(),
      });
      groups
        .last_mut()
        .ok_or_else(|| McpError::Protocol("Failed to create hosts group".to_string()))?
    };

    // 检查是否已存在相同域名
    if target_group.hosts.iter().any(|h| h.domain == domain) {
      return Ok(CallToolResult {
        content: vec![Content::text(format!(
          "Domain '{}' already exists in group '{}'",
          domain, group
        ))],
        is_error: Some(true),
      });
    }

    // 添加新条目
    target_group.hosts.push(HostEntry {
      ip: ip.to_string(),
      domain: domain.to_string(),
      disabled,
    });

    // 写回 hosts 文件
    self.write_hosts(groups)?;

    Ok(CallToolResult {
      content: vec![Content::text(format!(
        "Added: {} {} → {} (disabled: {})",
        ip, domain, group, disabled
      ))],
      is_error: Some(false),
    })
  }

  /// 处理 hosts_remove 工具
  fn handle_hosts_remove(
    &self,
    args: &HashMap<String, serde_json::Value>,
  ) -> Result<CallToolResult, McpError> {
    let domain = args
      .get("domain")
      .and_then(|v| v.as_str())
      .ok_or_else(|| McpError::InvalidParams("Missing domain parameter".to_string()))?;

    let group_filter = args.get("group").and_then(|v| v.as_str());

    // 读取当前 hosts
    let mut groups = read_system_hosts()
      .map_err(|e| McpError::Protocol(format!("Failed to read hosts: {}", e)))?;

    let mut removed = false;
    let mut removed_from = Vec::new();

    for group in groups.iter_mut() {
      if let Some(filter) = group_filter {
        if group.name != filter {
          continue;
        }
      }

      let original_len = group.hosts.len();
      group.hosts.retain(|h| h.domain != domain);
      if group.hosts.len() < original_len {
        removed = true;
        removed_from.push(group.name.clone());
      }
    }

    if !removed {
      return Ok(CallToolResult {
        content: vec![Content::text(format!("Domain '{}' not found", domain))],
        is_error: Some(true),
      });
    }

    // 写回 hosts 文件
    self.write_hosts(groups)?;

    Ok(CallToolResult {
      content: vec![Content::text(format!(
        "Removed '{}' from groups: {}",
        domain,
        removed_from.join(", ")
      ))],
      is_error: Some(false),
    })
  }

  /// 处理 hosts_toggle 工具
  fn handle_hosts_toggle(
    &self,
    args: &HashMap<String, serde_json::Value>,
  ) -> Result<CallToolResult, McpError> {
    let domain = args
      .get("domain")
      .and_then(|v| v.as_str())
      .ok_or_else(|| McpError::InvalidParams("Missing domain parameter".to_string()))?;

    let target_disabled = args.get("disabled").and_then(|v| v.as_bool());

    // 读取当前 hosts
    let mut groups = read_system_hosts()
      .map_err(|e| McpError::Protocol(format!("Failed to read hosts: {}", e)))?;

    let mut found = false;
    let mut changes = Vec::new();

    for group in groups.iter_mut() {
      for host in group.hosts.iter_mut() {
        if host.domain == domain {
          found = true;
          let old_disabled = host.disabled;
          let new_disabled = target_disabled.unwrap_or(!old_disabled);
          host.disabled = new_disabled;
          changes.push((group.name.clone(), old_disabled, new_disabled));
        }
      }
    }

    if !found {
      return Ok(CallToolResult {
        content: vec![Content::text(format!("Domain '{}' not found", domain))],
        is_error: Some(true),
      });
    }

    // 写回 hosts 文件
    self.write_hosts(groups)?;

    let change_desc = changes
      .into_iter()
      .map(|(group, old, new)| format!("{}: {} → {}", group, old, new))
      .collect::<Vec<_>>()
      .join("\n");

    Ok(CallToolResult {
      content: vec![Content::text(format!(
        "Toggled '{}':\n{}",
        domain, change_desc
      ))],
      is_error: Some(false),
    })
  }

  /// 处理 hosts_write 工具
  fn handle_hosts_write(
    &self,
    args: &HashMap<String, serde_json::Value>,
  ) -> Result<CallToolResult, McpError> {
    let groups_value = args
      .get("groups")
      .ok_or_else(|| McpError::InvalidParams("Missing groups parameter".to_string()))?;

    let groups: Vec<GroupHosts> = serde_json::from_value(groups_value.clone())
      .map_err(|e| McpError::InvalidParams(format!("Invalid groups format: {}", e)))?;

    // 写回 hosts 文件
    self.write_hosts(groups.clone())?;

    let total_entries: usize = groups.iter().map(|g| g.hosts.len()).sum();

    Ok(CallToolResult {
      content: vec![Content::text(format!(
        "Wrote {} groups with {} total entries",
        groups.len(),
        total_entries
      ))],
      is_error: Some(false),
    })
  }

  // ========================================================================
  // 辅助函数
  // ========================================================================

  /// 写入 Hosts 文件（直接操作文件系统）
  fn write_hosts(&self, groups: Vec<GroupHosts>) -> Result<(), McpError> {
    // 尝试读取现有的 hosts 文件
    let hosts_content = match std::fs::read_to_string(HOSTS_PATH) {
      Ok(content) => content,
      Err(e) => {
        return Err(McpError::Io(format!("Failed to read hosts file: {}", e)));
      }
    };

    // 分析原始 hosts 文件，提取非 Jedi 管理部分
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_jedi_section = false;

    for line in hosts_content.lines() {
      let trimmed = line.trim_start();

      // 检查是否进入 Jedi 管理部分
      if trimmed.starts_with("# === JEDI HOSTS MANAGER ===") {
        in_jedi_section = true;
        continue;
      }

      // 检查是否离开 Jedi 管理部分
      if trimmed.starts_with("# === END JEDI HOSTS MANAGER ===") {
        in_jedi_section = false;
        continue;
      }

      // 如果不在 Jedi 管理部分中，添加到非 Jedi 行
      if !in_jedi_section {
        new_lines.push(line.to_string());
      }
    }

    // 添加 Jedi 管理部分
    new_lines.push("# === JEDI HOSTS MANAGER ===".to_string());

    for g in &groups {
      // 创建分组标题
      let group_name = g.name.clone();
      new_lines.push(format!("# +{}+", group_name));

      // 按域名排序显示
      let mut sorted_hosts: Vec<HostEntry> = g.hosts.clone();
      sorted_hosts.sort_by(|a, b| a.domain.cmp(&b.domain));

      for host in sorted_hosts {
        if host.disabled {
          // 如果禁用，添加注释符号
          new_lines.push(format!("# {} {}", host.ip, host.domain));
        } else {
          // 如果启用，正常显示
          new_lines.push(format!("{} {}", host.ip, host.domain));
        }
      }
    }

    new_lines.push("# === END JEDI HOSTS MANAGER ===".to_string());

    let new_content = new_lines.join("\n") + "\n";

    // 写入文件
    std::fs::write(HOSTS_PATH, new_content)
      .map_err(|e| McpError::Io(format!("Failed to write hosts file: {}", e)))?;

    Ok(())
  }

  /// 确保已初始化
  fn ensure_initialized(&self) -> Result<(), McpError> {
    if self.state != ServerState::Initialized {
      return Err(McpError::NotInitialized);
    }
    Ok(())
  }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_hosts_mcp_server_new() {
    let server = HostsMcpServer::new();
    assert_eq!(server.state, ServerState::Uninitialized);
  }

  #[test]
  fn test_hosts_mcp_server_initialize() {
    let mut server = HostsMcpServer::new();
    assert!(server.initialize().is_ok());
    assert_eq!(server.state, ServerState::Initialized);
  }

  #[test]
  fn test_list_tools() {
    let mut server = HostsMcpServer::new();
    server.initialize().unwrap();

    let tools = server.list_tools().unwrap();
    assert!(!tools.is_empty());

    // 验证工具名称
    let tool_names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"hosts_read"));
    assert!(tool_names.contains(&"hosts_list"));
    assert!(tool_names.contains(&"hosts_add"));
    assert!(tool_names.contains(&"hosts_remove"));
    assert!(tool_names.contains(&"hosts_toggle"));
    assert!(tool_names.contains(&"hosts_write"));
  }
}
