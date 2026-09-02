// Jedi 作为 MCP server 对外暴露（P4，战略层）
//
// 以 `jedi --mcp-server` 子命令启动一个纯 stdio MCP server，把 Jedi 的**只读**内置工具
// 暴露给 Claude Desktop / Cursor 等外部 Agent 调用。
//
// 安全（§14-1 已决）：默认仅导出只读工具；写操作必须经显式白名单逐个开启，否则不导出。
// 白名单来自 `--allow-write=a,b` 参数或 JEDI_MCP_ALLOW_WRITE 环境变量。
//
// 候选池只含**不依赖 Tauri 运行时**的原生工具（hosts / memory）——壁纸/播客/系统工具需要
// AppHandle，在无头模式下不可用，因此永远不导出。记忆工具涉及用户隐私，也需显式白名单。

use crate::mcp::types::*;
use crate::tools::{AgentTool, RiskLevel, ToolDeclaration, ToolRegistry};
use serde_json::json;
use std::io::{BufRead, Write};
use std::sync::Arc;

/// 无头模式下可用的候选工具池（不依赖 AppHandle）
fn exportable_candidates() -> Vec<Arc<dyn AgentTool>> {
  let mut v = crate::tools::native::hosts::tools();
  v.extend(crate::tools::native::memory::tools());
  v
}

/// 解析写工具白名单：`--allow-write=a,b`、`--allow-write a,b` 或 JEDI_MCP_ALLOW_WRITE
pub fn parse_allow_write(args: &[String]) -> Vec<String> {
  let mut raw: Vec<String> = Vec::new();

  for (i, a) in args.iter().enumerate() {
    if let Some(rest) = a.strip_prefix("--allow-write=") {
      raw.push(rest.to_string());
    } else if a == "--allow-write" {
      if let Some(next) = args.get(i + 1) {
        if !next.starts_with("--") {
          raw.push(next.clone());
        }
      }
    }
  }
  if let Ok(env) = std::env::var("JEDI_MCP_ALLOW_WRITE") {
    raw.push(env);
  }

  let mut out: Vec<String> = Vec::new();
  for chunk in raw {
    for name in chunk.split(',') {
      let name = name.trim();
      if !name.is_empty() && !out.iter().any(|x| x == name) {
        out.push(name.to_string());
      }
    }
  }
  out
}

/// 构建可导出工具的注册表。
/// 默认：候选池中的只读 hosts 工具；白名单：按名字显式追加（可含写操作 / 记忆）。
fn build_export_registry(allow_write: &[String]) -> ToolRegistry {
  let reg = ToolRegistry::new();

  // 默认导出：只读且属于 hosts 分组（记忆默认不导出，避免泄露用户偏好）
  for tool in crate::tools::native::hosts::tools() {
    if tool.declaration().risk == RiskLevel::Read {
      let _ = reg.register(tool);
    }
  }

  // 白名单：显式点名才导出（重复注册会失败，忽略即可）
  if !allow_write.is_empty() {
    for tool in exportable_candidates() {
      let name = tool.declaration().name;
      if allow_write.iter().any(|n| n == &name) {
        let _ = reg.register(tool);
      }
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
pub fn run_stdio_server(allow_write: Vec<String>) {
  let registry = build_export_registry(&allow_write);
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
    let reg = build_export_registry(&[]);
    let decls = reg.all_declarations();
    assert!(!decls.is_empty());
    // 默认仅只读工具
    assert!(decls.iter().all(|d| d.risk == RiskLevel::Read));
    let names: Vec<_> = decls.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"hosts_read"));
    assert!(names.contains(&"hosts_list"));
    // 写工具默认不导出
    assert!(!names.contains(&"hosts_add"));
    // 记忆工具（涉隐私）默认也不导出
    assert!(!names.contains(&"memory_list"));
  }

  #[test]
  fn test_export_registry_honors_whitelist() {
    let reg = build_export_registry(&["hosts_add".to_string(), "memory_list".to_string()]);
    let names: Vec<String> = reg.all_declarations().into_iter().map(|d| d.name).collect();
    // 白名单点名的才追加
    assert!(names.contains(&"hosts_add".to_string()));
    assert!(names.contains(&"memory_list".to_string()));
    // 未点名的写工具仍不导出
    assert!(!names.contains(&"hosts_remove".to_string()));
    // 需要 AppHandle 的工具永不导出
    assert!(!names.contains(&"wallpaper_set".to_string()));
    assert!(!names.contains(&"system_info".to_string()));
  }

  #[test]
  fn test_export_registry_ignores_unknown_whitelist_names() {
    let reg = build_export_registry(&["not_a_tool".to_string()]);
    let names: Vec<String> = reg.all_declarations().into_iter().map(|d| d.name).collect();
    assert!(!names.contains(&"not_a_tool".to_string()));
    // 默认只读工具仍在
    assert!(names.contains(&"hosts_read".to_string()));
  }

  #[test]
  fn test_parse_allow_write_forms() {
    let eq = parse_allow_write(&["--mcp-server".into(), "--allow-write=hosts_add,hosts_remove".into()]);
    assert_eq!(eq, vec!["hosts_add", "hosts_remove"]);

    let spaced = parse_allow_write(&["--allow-write".into(), "hosts_add".into()]);
    assert_eq!(spaced, vec!["hosts_add"]);

    // 后面跟另一个 flag 时不误吞
    let dangling = parse_allow_write(&["--allow-write".into(), "--mcp-server".into()]);
    assert!(dangling.is_empty());

    // 去重 + 去空白
    let dedup = parse_allow_write(&["--allow-write= hosts_add , hosts_add ,".into()]);
    assert_eq!(dedup, vec!["hosts_add"]);

    assert!(parse_allow_write(&["--mcp-server".into()]).is_empty());
  }

  #[test]
  fn test_export_tools_convert() {
    let reg = build_export_registry(&[]);
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
    let reg = build_export_registry(&[]);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let req = JsonRpcRequest::new(RequestId::Number(1), "tools/list");
    let resp = handle_request(&reg, &rt, &req);
    assert!(resp.is_success());
  }

  #[test]
  fn test_handle_unknown_method() {
    let reg = build_export_registry(&[]);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let req = JsonRpcRequest::new(RequestId::Number(2), "nope/nope");
    let resp = handle_request(&reg, &rt, &req);
    assert!(!resp.is_success());
    assert!(resp.error.is_some());
  }
}
