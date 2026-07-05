// Third-party MCP client manager.
//
// Wires Jedi's existing (previously unwired) `McpClient` into the app: it lets
// the frontend connect to external MCP servers over stdio, discover their tools,
// and call them. Each connection keeps a live `McpClient` (a spawned child
// process) keyed by a caller-chosen server id.
//
// The underlying `McpClient` API is synchronous/blocking, so every command runs
// its work on a blocking thread via `spawn_blocking` to avoid stalling the async
// runtime / UI.

use crate::mcp::{McpClient, McpClientBuilder, TransportConfig};
use crate::mcp::types::Content;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Connection config for one third-party MCP server (stdio transport).
#[derive(Debug, Clone, Deserialize)]
pub struct McpServerConfig {
  pub id: String,
  pub name: String,
  pub command: String,
  #[serde(default)]
  pub args: Vec<String>,
  /// Extra environment variables as [name, value] pairs.
  #[serde(default)]
  pub env: Vec<(String, String)>,
}

/// A tool discovered on a connected MCP server, shaped for the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct McpToolInfo {
  pub name: String,
  pub description: String,
  pub input_schema: serde_json::Value,
}

/// Snapshot of a connected server (id + name + its tools).
#[derive(Debug, Clone, Serialize)]
pub struct McpConnectedServer {
  pub id: String,
  pub name: String,
  pub tools: Vec<McpToolInfo>,
}

struct Connection {
  name: String,
  client: McpClient,
  tools: Vec<McpToolInfo>,
}

/// Tauri-managed state holding every live MCP connection.
#[derive(Default)]
pub struct McpManager {
  conns: Arc<Mutex<HashMap<String, Connection>>>,
}

impl McpManager {
  pub fn new() -> Self {
    Self::default()
  }
}

fn tool_infos(client: &mut McpClient) -> Result<Vec<McpToolInfo>, String> {
  let tools = client.list_tools().map_err(|e| e.to_string())?;
  Ok(
    tools
      .into_iter()
      .map(|t| McpToolInfo {
        name: t.name,
        description: t.description.unwrap_or_default(),
        input_schema: serde_json::to_value(&t.input_schema).unwrap_or(serde_json::Value::Null),
      })
      .collect(),
  )
}

/// Connect to a third-party MCP server, initialize it, and return its tools.
/// Replaces any existing connection with the same id.
#[tauri::command]
pub async fn mcp_connect(
  manager: State<'_, McpManager>,
  config: McpServerConfig,
) -> Result<McpConnectedServer, String> {
  let conns = manager.conns.clone();
  tauri::async_runtime::spawn_blocking(move || {
    let transport = TransportConfig::new(config.command.clone())
      .with_args(config.args.clone())
      .with_env(config.env.clone());
    let mut client = McpClientBuilder::new()
      .name("jedi")
      .version(env!("CARGO_PKG_VERSION"))
      .transport(transport)
      .build();
    client.start().map_err(|e| e.to_string())?;
    let tools = tool_infos(&mut client)?;

    let snapshot = McpConnectedServer {
      id: config.id.clone(),
      name: config.name.clone(),
      tools: tools.clone(),
    };

    let mut map = conns.lock().map_err(|e| e.to_string())?;
    // Drop any prior connection with this id (stops its child process).
    if let Some(mut old) = map.remove(&config.id) {
      let _ = old.client.stop();
    }
    map.insert(
      config.id.clone(),
      Connection {
        name: config.name,
        client,
        tools,
      },
    );
    Ok(snapshot)
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Disconnect a server and stop its child process.
#[tauri::command]
pub async fn mcp_disconnect(manager: State<'_, McpManager>, id: String) -> Result<(), String> {
  let conns = manager.conns.clone();
  tauri::async_runtime::spawn_blocking(move || {
    let mut map = conns.lock().map_err(|e| e.to_string())?;
    if let Some(mut conn) = map.remove(&id) {
      let _ = conn.client.stop();
    }
    Ok(())
  })
  .await
  .map_err(|e| e.to_string())?
}

/// List currently connected servers and their tools.
#[tauri::command]
pub async fn mcp_list_connected(
  manager: State<'_, McpManager>,
) -> Result<Vec<McpConnectedServer>, String> {
  let conns = manager.conns.clone();
  tauri::async_runtime::spawn_blocking(move || {
    let map = conns.lock().map_err(|e| e.to_string())?;
    Ok(
      map
        .iter()
        .map(|(id, conn)| McpConnectedServer {
          id: id.clone(),
          name: conn.name.clone(),
          tools: conn.tools.clone(),
        })
        .collect(),
    )
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Call a tool on a connected server, returning its text output.
#[tauri::command]
pub async fn mcp_call_tool(
  manager: State<'_, McpManager>,
  id: String,
  tool: String,
  args: Option<serde_json::Value>,
) -> Result<String, String> {
  let conns = manager.conns.clone();
  tauri::async_runtime::spawn_blocking(move || {
    let arguments = match args {
      Some(serde_json::Value::Object(map)) => Some(map.into_iter().collect()),
      _ => None,
    };
    let mut guard = conns.lock().map_err(|e| e.to_string())?;
    let conn = guard
      .get_mut(&id)
      .ok_or_else(|| format!("MCP server 未连接: {}", id))?;
    let result = conn
      .client
      .call_tool(&tool, arguments)
      .map_err(|e| e.to_string())?;

    let text = result
      .content
      .iter()
      .filter_map(|c| match c {
        Content::Text { text } => Some(text.clone()),
        Content::Image { .. } => Some("[image]".to_string()),
        Content::Resource { .. } => Some("[resource]".to_string()),
      })
      .collect::<Vec<_>>()
      .join("\n");

    if result.is_error.unwrap_or(false) {
      Err(if text.is_empty() { "工具返回错误".into() } else { text })
    } else {
      Ok(text)
    }
  })
  .await
  .map_err(|e| e.to_string())?
}
