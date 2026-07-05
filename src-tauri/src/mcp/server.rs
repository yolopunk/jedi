// Jedi as an outbound MCP server (stdio).
//
// Running `jedi --mcp-server` starts a pure stdio JSON-RPC MCP server (protocol
// 2024-11-05) that exposes a curated set of Jedi's read-only backend tools —
// memory recall/list and web search/fetch — so other MCP clients (e.g. Claude
// Desktop) can call them by spawning Jedi with this flag. Read-only by design:
// no writes, no shell, no hosts mutation.

use crate::api::{memory, web};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

/// Tool declarations advertised to MCP clients.
fn tool_defs() -> Value {
  json!([
    {
      "name": "memory_recall",
      "description": "回忆一条此前记住的信息",
      "inputSchema": {
        "type": "object",
        "properties": { "key": { "type": "string", "description": "要回忆的键" } },
        "required": ["key"]
      }
    },
    {
      "name": "memory_list",
      "description": "列出所有已记住的信息",
      "inputSchema": { "type": "object", "properties": {} }
    },
    {
      "name": "web_search",
      "description": "搜索网页（DuckDuckGo），返回标题/URL/摘要列表",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": { "type": "string", "description": "搜索关键词" },
          "max_results": { "type": "number", "description": "结果数量上限（1-10）" }
        },
        "required": ["query"]
      }
    },
    {
      "name": "web_fetch",
      "description": "抓取单个 URL 并返回其正文文本",
      "inputSchema": {
        "type": "object",
        "properties": {
          "url": { "type": "string", "description": "要抓取的绝对 URL" },
          "max_chars": { "type": "number", "description": "返回正文的最大字符数" }
        },
        "required": ["url"]
      }
    }
  ])
}

fn call_tool(name: &str, args: &Value, rt: &tokio::runtime::Runtime) -> Result<String, String> {
  match name {
    "memory_recall" => {
      let key = args
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or("缺少参数: key")?;
      memory::memory_recall(key.to_string())
    }
    "memory_list" => {
      let entries = memory::memory_list()?;
      if entries.is_empty() {
        Ok("（暂无记忆）".into())
      } else {
        Ok(
          entries
            .iter()
            .map(|e| format!("{}: {}", e.key, e.value))
            .collect::<Vec<_>>()
            .join("\n"),
        )
      }
    }
    "web_search" => {
      let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or("缺少参数: query")?
        .to_string();
      let max = args.get("max_results").and_then(|v| v.as_u64()).map(|n| n as usize);
      let hits = rt.block_on(web::web_search(query, max))?;
      Ok(
        hits
          .iter()
          .map(|h| format!("{}\n{}\n{}", h.title, h.url, h.snippet))
          .collect::<Vec<_>>()
          .join("\n\n"),
      )
    }
    "web_fetch" => {
      let url = args
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or("缺少参数: url")?
        .to_string();
      let max = args.get("max_chars").and_then(|v| v.as_u64()).map(|n| n as usize);
      let result = rt.block_on(web::web_fetch(url, max))?;
      Ok(result.text)
    }
    _ => Err(format!("未知工具: {}", name)),
  }
}

/// Run the stdio MCP server loop, blocking until stdin closes.
pub fn run_stdio_server() {
  let rt = match tokio::runtime::Runtime::new() {
    Ok(r) => r,
    Err(e) => {
      eprintln!("无法创建异步运行时: {}", e);
      return;
    }
  };
  let stdin = std::io::stdin();
  let stdout = std::io::stdout();
  let mut out = stdout.lock();

  for line in stdin.lock().lines() {
    let Ok(line) = line else { break };
    let line = line.trim();
    if line.is_empty() {
      continue;
    }
    let Ok(req) = serde_json::from_str::<Value>(line) else {
      continue;
    };
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = req.get("id").cloned();

    // Notifications (no id) produce no response.
    let result: Option<Result<Value, (i64, String)>> = match method {
      "initialize" => Some(Ok(json!({
        "protocolVersion": "2024-11-05",
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "jedi", "version": env!("CARGO_PKG_VERSION") }
      }))),
      "notifications/initialized" => None,
      "tools/list" => Some(Ok(json!({ "tools": tool_defs() }))),
      "tools/call" => {
        let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
        Some(Ok(match call_tool(name, &args, &rt) {
          Ok(text) => json!({ "content": [{ "type": "text", "text": text }], "isError": false }),
          Err(e) => json!({ "content": [{ "type": "text", "text": e }], "isError": true }),
        }))
      }
      _ => Some(Err((-32601, format!("Method not found: {}", method)))),
    };

    if let (Some(id), Some(result)) = (id, result) {
      let msg = match result {
        Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
        Err((code, message)) => {
          json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
        }
      };
      if writeln!(out, "{}", msg).is_err() {
        break;
      }
      let _ = out.flush();
    }
  }
}
