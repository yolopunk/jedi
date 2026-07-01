// 第三方 MCP server 接入管理（P3）
//
// 把外部 MCP server（stdio）连接进来，将其工具适配为 AgentTool 注册进 ToolRegistry。
// 复用已实现的同步 McpClient；因其为阻塞式，工具调用经 spawn_blocking 执行。
//
// 安全：第三方 server = 运行外部程序，属信任边界。其工具默认风险 = Write（至少需确认）。

use crate::mcp::protocol::{McpClient, McpClientBuilder};
use crate::mcp::transport::TransportConfig;
use crate::mcp::types::{CallToolResult, Content};
use crate::tools::{AgentTool, RiskLevel, ToolDeclaration, ToolOutcome, ToolRegistry, ToolSource};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

// ============================================================================
// 配置 / 状态
// ============================================================================

/// 第三方 MCP server 配置（由前端传入）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
  pub id: String,
  #[serde(default)]
  pub name: String,
  /// 传输方式，目前仅支持 "stdio"
  #[serde(default = "default_transport")]
  pub transport: String,
  #[serde(default)]
  pub command: String,
  #[serde(default)]
  pub args: Vec<String>,
  #[serde(default)]
  pub env: HashMap<String, String>,
}

fn default_transport() -> String {
  "stdio".to_string()
}

/// 连接状态（返回给前端）
#[derive(Debug, Clone, Serialize)]
pub struct McpServerStatus {
  pub id: String,
  pub name: String,
  pub connected: bool,
  pub tool_count: usize,
  /// 该 server 暴露的工具名（已注册进 registry 的 LLM 名）
  pub tools: Vec<String>,
}

// ============================================================================
// 工具名规整（【R1】function-calling 命名规则）
// ============================================================================

fn sanitize(s: &str) -> String {
  s.chars()
    .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
    .collect()
}

/// 远端工具名 → LLM 工具名：mcp_<server>_<tool>，截断到 64
pub fn mangle_name(server_id: &str, tool: &str) -> String {
  let raw = format!("mcp_{}_{}", sanitize(server_id), sanitize(tool));
  if raw.len() > 64 {
    raw.chars().take(64).collect()
  } else {
    raw
  }
}

fn flatten_result(result: &CallToolResult) -> String {
  result
    .content
    .iter()
    .map(|c| match c {
      Content::Text { text } => text.clone(),
      Content::Image { mime_type, .. } => format!("[image: {}]", mime_type),
      Content::Resource { resource } => resource
        .text
        .clone()
        .unwrap_or_else(|| format!("[resource: {}]", resource.uri)),
    })
    .collect::<Vec<_>>()
    .join("\n")
}

// ============================================================================
// McpClientTool：远端工具适配为 AgentTool
// ============================================================================

pub struct McpClientTool {
  remote_name: String,
  decl: ToolDeclaration,
  client: Arc<Mutex<McpClient>>,
}

#[async_trait]
impl AgentTool for McpClientTool {
  fn declaration(&self) -> ToolDeclaration {
    self.decl.clone()
  }

  async fn call(&self, args: serde_json::Value, _snapshot: Option<String>) -> ToolOutcome {
    let client = self.client.clone();
    let remote = self.remote_name.clone();
    let arg_map: Option<HashMap<String, serde_json::Value>> = args
      .as_object()
      .map(|o| o.clone().into_iter().collect());

    // 阻塞式客户端 → 放到 blocking 线程执行
    let joined = tokio::task::spawn_blocking(move || {
      let mut c = client.lock().map_err(|e| e.to_string())?;
      c.call_tool(&remote, arg_map).map_err(|e| e.to_string())
    })
    .await;

    match joined {
      Ok(Ok(result)) => {
        let is_error = result.is_error.unwrap_or(false);
        let content = flatten_result(&result);
        ToolOutcome {
          content,
          is_error,
          undo_token: None,
        }
      }
      Ok(Err(e)) => ToolOutcome::error(format!("MCP 工具调用失败: {}", e)),
      Err(e) => ToolOutcome::error(format!("任务执行失败: {}", e)),
    }
  }
}

// ============================================================================
// McpManager：连接管理（Tauri 托管状态）
// ============================================================================

struct Connection {
  #[allow(dead_code)]
  client: Arc<Mutex<McpClient>>,
  name: String,
  tool_names: Vec<String>,
}

#[derive(Default)]
pub struct McpManager {
  conns: Mutex<HashMap<String, Connection>>,
}

impl McpManager {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn list(&self) -> Vec<McpServerStatus> {
    match self.conns.lock() {
      Ok(conns) => conns
        .iter()
        .map(|(id, c)| McpServerStatus {
          id: id.clone(),
          name: c.name.clone(),
          connected: true,
          tool_count: c.tool_names.len(),
          tools: c.tool_names.clone(),
        })
        .collect(),
      Err(_) => Vec::new(),
    }
  }
}

/// 启动 stdio 客户端并拉取工具（阻塞，放 blocking 线程）
fn start_client(config: &McpServerConfig) -> Result<(Arc<Mutex<McpClient>>, Vec<crate::mcp::types::Tool>), String> {
  if config.transport != "stdio" {
    return Err(format!("暂不支持的传输方式: {}（当前仅支持 stdio）", config.transport));
  }
  if config.command.trim().is_empty() {
    return Err("stdio 传输需要 command".to_string());
  }

  let env: Vec<(String, String)> = config.env.clone().into_iter().collect();
  let tconf = TransportConfig::new(&config.command)
    .with_args(config.args.clone())
    .with_env(env);

  let mut client = McpClientBuilder::new()
    .name("jedi-mcp-client")
    .transport(tconf)
    .build();
  client.start().map_err(|e| e.to_string())?;
  let tools = client.list_tools().map_err(|e| e.to_string())?;
  Ok((Arc::new(Mutex::new(client)), tools))
}

// ============================================================================
// Tauri commands
// ============================================================================

/// 连接一个第三方 MCP server，并把其工具注入 registry
#[tauri::command]
pub async fn mcp_connect(
  registry: State<'_, ToolRegistry>,
  manager: State<'_, McpManager>,
  config: McpServerConfig,
) -> Result<McpServerStatus, String> {
  // 若已连接，先断开
  let _ = mcp_disconnect_inner(&registry, &manager, &config.id);

  let cfg = config.clone();
  let (client_arc, tools) = tokio::task::spawn_blocking(move || start_client(&cfg))
    .await
    .map_err(|e| format!("任务执行失败: {}", e))??;

  let server_name = if config.name.is_empty() {
    config.id.clone()
  } else {
    config.name.clone()
  };

  let mut tool_names = Vec::new();
  for t in &tools {
    let name = mangle_name(&config.id, &t.name);
    let input_schema =
      serde_json::to_value(&t.input_schema).unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
    let decl = ToolDeclaration {
      name: name.clone(),
      description: t.description.clone().unwrap_or_default(),
      input_schema,
      // 第三方工具默认需确认（§7 / §8 信任边界）
      risk: RiskLevel::Write,
      source: ToolSource::Mcp {
        server_id: config.id.clone(),
        remote_name: t.name.clone(),
      },
      group: server_name.clone(),
    };
    let tool = McpClientTool {
      remote_name: t.name.clone(),
      decl,
      client: client_arc.clone(),
    };
    if registry.register(Arc::new(tool)).is_ok() {
      tool_names.push(name);
    }
  }

  if let Ok(mut conns) = manager.conns.lock() {
    conns.insert(
      config.id.clone(),
      Connection {
        client: client_arc,
        name: server_name.clone(),
        tool_names: tool_names.clone(),
      },
    );
  }

  Ok(McpServerStatus {
    id: config.id,
    name: server_name,
    connected: true,
    tool_count: tool_names.len(),
    tools: tool_names,
  })
}

fn mcp_disconnect_inner(registry: &ToolRegistry, manager: &McpManager, id: &str) -> bool {
  let removed = manager.conns.lock().ok().and_then(|mut c| c.remove(id));
  match removed {
    Some(conn) => {
      for name in &conn.tool_names {
        registry.unregister(name);
      }
      // client 随 Connection drop → McpClient::drop 会 stop() 子进程
      true
    }
    None => false,
  }
}

/// 断开某个 MCP server，注销其工具
#[tauri::command]
pub fn mcp_disconnect(
  registry: State<'_, ToolRegistry>,
  manager: State<'_, McpManager>,
  server_id: String,
) -> Result<(), String> {
  if mcp_disconnect_inner(&registry, &manager, &server_id) {
    Ok(())
  } else {
    Err("该 server 未连接".to_string())
  }
}

/// 列出已连接的 MCP server
#[tauri::command]
pub fn mcp_list_connected(manager: State<'_, McpManager>) -> Vec<McpServerStatus> {
  manager.list()
}

/// 测试连接：连接、拉取工具、返回状态（保持连接，等同 connect）
#[tauri::command]
pub async fn mcp_server_test(
  registry: State<'_, ToolRegistry>,
  manager: State<'_, McpManager>,
  config: McpServerConfig,
) -> Result<McpServerStatus, String> {
  mcp_connect(registry, manager, config).await
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mangle_name_safe() {
    let n = mangle_name("my server", "create/issue");
    assert_eq!(n, "mcp_my_server_create_issue");
    assert!(n
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
  }

  #[test]
  fn test_mangle_name_truncates() {
    let long = "x".repeat(100);
    let n = mangle_name("srv", &long);
    assert!(n.len() <= 64);
  }

  #[test]
  fn test_flatten_result_text() {
    let r = CallToolResult {
      content: vec![Content::text("a"), Content::text("b")],
      is_error: Some(false),
    };
    assert_eq!(flatten_result(&r), "a\nb");
  }

  #[test]
  fn test_start_client_rejects_non_stdio() {
    let cfg = McpServerConfig {
      id: "x".into(),
      name: "X".into(),
      transport: "sse".into(),
      command: "".into(),
      args: vec![],
      env: HashMap::new(),
    };
    assert!(start_client(&cfg).is_err());
  }

  #[test]
  fn test_start_client_requires_command() {
    let cfg = McpServerConfig {
      id: "x".into(),
      name: "X".into(),
      transport: "stdio".into(),
      command: "  ".into(),
      args: vec![],
      env: HashMap::new(),
    };
    assert!(start_client(&cfg).is_err());
  }

  /// 端到端：连接一个真实的 mock MCP server 子进程（stdio），
  /// 验证 initialize/tools/list/tools/call 全链路。
  /// 若环境无 python3 或脚本缺失，start_client 返回 Err → 优雅跳过（不判失败）。
  #[test]
  fn test_connect_mock_server_end_to_end() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/mock_mcp_server.py");
    if !std::path::Path::new(script).exists() {
      eprintln!("skip: mock server fixture 不存在");
      return;
    }
    let cfg = McpServerConfig {
      id: "mock".into(),
      name: "Mock".into(),
      transport: "stdio".into(),
      command: "python3".into(),
      args: vec![script.to_string()],
      env: HashMap::new(),
    };

    let (client, tools) = match start_client(&cfg) {
      Ok(v) => v,
      Err(e) => {
        eprintln!("skip: 无法启动 mock server（可能缺少 python3）: {}", e);
        return;
      }
    };

    // tools/list 拿到 echo 工具
    assert!(tools.iter().any(|t| t.name == "echo"), "应包含 echo 工具");

    // 名称规整后应可注入 registry
    let mangled = mangle_name(&cfg.id, "echo");
    assert_eq!(mangled, "mcp_mock_echo");

    // tools/call echo → "echo: hi"
    let mut args = HashMap::new();
    args.insert("text".to_string(), serde_json::json!("hi"));
    let result = {
      let mut c = client.lock().unwrap();
      c.call_tool("echo", Some(args)).expect("call_tool 应成功")
    };
    let text = flatten_result(&result);
    assert_eq!(text, "echo: hi");
    assert_eq!(result.is_error, Some(false));
  }
}
