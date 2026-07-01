// Jedi 作为 MCP server 对外暴露（P4，战略层）
//
// 以 `jedi --mcp-server` 子命令启动一个纯 stdio MCP server，把 Jedi 的**只读**内置工具
// 暴露给 Claude Desktop / Cursor 等外部 Agent 调用。
//
// 安全（§14-1 已决）：仅导出只读工具；写操作须显式白名单，默认拒绝——因此这里只导出
// 不依赖 Tauri 运行时、且风险为 Read 的原生工具（当前为 hosts_read / hosts_list）。

use crate::mcp::types::*;
use crate::tools::{RiskLevel, ToolDeclaration, ToolRegistry};
use serde_json::json;
use std::io::{BufRead, Write};

/// 构建仅含可导出工具的注册表（只读、无需 AppHandle）
fn build_export_registry() -> ToolRegistry {
  let reg = ToolRegistry::new();
  for tool in crate::tools::native::hosts::tools() {
    if tool.declaration().risk == RiskLevel::Read {
      let _ = reg.register(tool);
    }
  }
  reg
}

fn decl_to_tool(d: &ToolDeclaration) -> Tool {
  Tool {
    name: d.name.clone(),
    description: Some(d.description.clone()),
    input_schema: serde_json::from_value(d.input_schema.clone()).unwrap_or_else(|_| ToolInputSchema::new()),
  }
}

/// 可导出工具的 MCP Tool 列表
fn export_tools(registry: &ToolRegistry) -> Vec<Tool> {
  registry.all_declarations().iter().map(decl_to_tool).collect()
}

fn initialize_result() -> InitializeResult {
  InitializeResult {
    protocol_version: MCP_VERSION.to_string(),
    capabilities: ServerCapabilities {
      tools: Some(ToolsCapability { list_changed: None }),
      ..Default::default()
    },
    server_info: Implementation::new("jedi", env!("CARGO_PKG_VERSION")),
    instructions: Some("Jedi 工具箱对外暴露的只读工具（hosts 查询）".to_string()),
  }
}

/// 处理单条请求，返回响应（tools/call 使用给定 runtime 执行异步工具）
fn handle_request(
  registry: &ToolRegistry,
  rt: &tokio::runtime::Runtime,
  req: &JsonRpcRequest,
) -> JsonRpcResponse {
  let id = req.id.clone();
  match req.method.as_str() {
    "initialize" => match serde_json::to_value(initialize_result()) {
      Ok(v) => JsonRpcResponse::success(id, v),
      Err(e) => JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string())),
    },
    "tools/list" => {
      let result = ListToolsResult {
        tools: export_tools(registry),
        next_cursor: None,
      };
      match serde_json::to_value(result) {
        Ok(v) => JsonRpcResponse::success(id, v),
        Err(e) => JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string())),
      }
    }
    "tools/call" => {
      let params = req
        .params
        .clone()
        .and_then(|p| serde_json::from_value::<CallToolParams>(p).ok());
      let params = match params {
        Some(p) => p,
        None => {
          return JsonRpcResponse::error(id, JsonRpcError::invalid_params("缺少 tools/call 参数"))
        }
      };
      let args = params
        .arguments
        .map(|m| json!(m))
        .unwrap_or_else(|| json!({}));
      let outcome = rt.block_on(registry.call(&params.name, args, None));
      let result = CallToolResult {
        content: vec![Content::text(outcome.content)],
        is_error: Some(outcome.is_error),
      };
      match serde_json::to_value(result) {
        Ok(v) => JsonRpcResponse::success(id, v),
        Err(e) => JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string())),
      }
    }
    other => JsonRpcResponse::error(id, JsonRpcError::method_not_found(other)),
  }
}

/// 运行 stdio MCP server（阻塞，直到 stdin 关闭）
pub fn run_stdio_server() {
  let registry = build_export_registry();
  let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

  let stdin = std::io::stdin();
  let stdout = std::io::stdout();

  for line in stdin.lock().lines() {
    let line = match line {
      Ok(l) => l,
      Err(_) => break,
    };
    if line.trim().is_empty() {
      continue;
    }
    // 通知（无 id，如 notifications/initialized）会解析失败 → 忽略
    let req: JsonRpcRequest = match serde_json::from_str(&line) {
      Ok(r) => r,
      Err(_) => continue,
    };
    let resp = handle_request(&registry, &rt, &req);
    if let Ok(s) = serde_json::to_string(&resp) {
      let mut out = stdout.lock();
      let _ = writeln!(out, "{}", s);
      let _ = out.flush();
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
  fn test_export_registry_only_read() {
    let reg = build_export_registry();
    let decls = reg.all_declarations();
    assert!(!decls.is_empty());
    // 仅只读工具
    assert!(decls.iter().all(|d| d.risk == RiskLevel::Read));
    let names: Vec<_> = decls.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"hosts_read"));
    assert!(names.contains(&"hosts_list"));
    // 写工具不导出
    assert!(!names.contains(&"hosts_add"));
  }

  #[test]
  fn test_export_tools_convert() {
    let reg = build_export_registry();
    let tools = export_tools(&reg);
    assert!(tools.iter().any(|t| t.name == "hosts_read"));
    let read = tools.iter().find(|t| t.name == "hosts_read").unwrap();
    assert_eq!(read.input_schema.schema_type, "object");
  }

  #[test]
  fn test_initialize_result_shape() {
    let r = initialize_result();
    assert_eq!(r.protocol_version, MCP_VERSION);
    assert_eq!(r.server_info.name, "jedi");
    assert!(r.capabilities.tools.is_some());
  }

  #[test]
  fn test_handle_tools_list() {
    let reg = build_export_registry();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let req = JsonRpcRequest::new(RequestId::Number(1), "tools/list");
    let resp = handle_request(&reg, &rt, &req);
    assert!(resp.is_success());
  }

  #[test]
  fn test_handle_unknown_method() {
    let reg = build_export_registry();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let req = JsonRpcRequest::new(RequestId::Number(2), "nope/nope");
    let resp = handle_request(&reg, &rt, &req);
    assert!(!resp.is_success());
    assert!(resp.error.is_some());
  }
}
