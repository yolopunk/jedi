// Async MCP client over the HTTP+SSE transport (MCP 2024-11-05).
//
// The built-in stdio McpClient can only spawn local processes, so this is a
// parallel, self-contained async client for remote servers. It speaks JSON-RPC
// over Server-Sent Events:
//   1. GET {url} (Accept: text/event-stream) opens the event stream.
//   2. The server's first `endpoint` event carries the URL to POST messages to.
//   3. Requests are POSTed there; responses/notifications arrive as `message`
//      SSE events on the open stream, correlated back by JSON-RPC id.

use crate::mcp::manager::McpToolInfo;
use futures::StreamExt;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, Mutex};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

type PendingMap = Arc<Mutex<HashMap<i64, oneshot::Sender<Value>>>>;

pub struct SseClient {
  http: reqwest::Client,
  message_url: String,
  next_id: AtomicI64,
  pending: PendingMap,
  reader: tokio::task::JoinHandle<()>,
}

impl Drop for SseClient {
  fn drop(&mut self) {
    self.reader.abort();
  }
}

/// Parse one raw SSE event block into (event, data).
fn parse_event(raw: &str) -> (String, String) {
  let mut event = "message".to_string();
  let mut data = String::new();
  for line in raw.lines() {
    let line = line.trim_end_matches('\r');
    if let Some(v) = line.strip_prefix("event:") {
      event = v.strip_prefix(' ').unwrap_or(v).to_string();
    } else if let Some(v) = line.strip_prefix("data:") {
      if !data.is_empty() {
        data.push('\n');
      }
      data.push_str(v.strip_prefix(' ').unwrap_or(v));
    }
  }
  (event, data)
}

impl SseClient {
  /// Open an SSE connection, run the MCP initialize handshake, and return the
  /// live client together with its tool list.
  pub async fn connect(
    url: &str,
    name: &str,
    version: &str,
  ) -> Result<(SseClient, Vec<McpToolInfo>), String> {
    let http = reqwest::Client::new();
    let resp = http
      .get(url)
      .header(reqwest::header::ACCEPT, "text/event-stream")
      .send()
      .await
      .map_err(|e| format!("SSE 连接失败: {}", e))?;
    if !resp.status().is_success() {
      return Err(format!("SSE 连接返回 {}", resp.status()));
    }

    let base = reqwest::Url::parse(url).map_err(|e| format!("无效 URL: {}", e))?;
    let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
    let (endpoint_tx, endpoint_rx) = oneshot::channel::<String>();

    let reader_pending = pending.clone();
    let reader = tokio::spawn(async move {
      let mut endpoint_tx = Some(endpoint_tx);
      let mut stream = resp.bytes_stream();
      let mut buf = String::new();
      while let Some(chunk) = stream.next().await {
        let Ok(bytes) = chunk else { break };
        buf.push_str(&String::from_utf8_lossy(&bytes));
        while let Some(pos) = buf.find("\n\n") {
          let raw: String = buf.drain(..pos + 2).collect();
          let (event, data) = parse_event(&raw);
          if event == "endpoint" {
            if let Some(tx) = endpoint_tx.take() {
              let _ = tx.send(data);
            }
          } else if event == "message" {
            if let Ok(val) = serde_json::from_str::<Value>(&data) {
              if let Some(id) = val.get("id").and_then(|v| v.as_i64()) {
                if let Some(tx) = reader_pending.lock().await.remove(&id) {
                  let _ = tx.send(val);
                }
              }
            }
          }
        }
      }
    });

    let endpoint = tokio::time::timeout(CONNECT_TIMEOUT, endpoint_rx)
      .await
      .map_err(|_| "等待 SSE endpoint 超时".to_string())?
      .map_err(|_| "SSE 连接在握手前关闭".to_string())?;
    let message_url = base
      .join(&endpoint)
      .map_err(|e| format!("无效 message endpoint: {}", e))?
      .to_string();

    let client = SseClient {
      http,
      message_url,
      next_id: AtomicI64::new(1),
      pending,
      reader,
    };

    client
      .request(
        "initialize",
        json!({
          "protocolVersion": "2024-11-05",
          "capabilities": {},
          "clientInfo": { "name": name, "version": version }
        }),
      )
      .await?;
    client.notify("notifications/initialized", json!({})).await?;

    let tools = client.list_tools().await?;
    Ok((client, tools))
  }

  async fn request(&self, method: &str, params: Value) -> Result<Value, String> {
    let id = self.next_id.fetch_add(1, Ordering::SeqCst);
    let (tx, rx) = oneshot::channel();
    self.pending.lock().await.insert(id, tx);

    let body = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
    let resp = self
      .http
      .post(&self.message_url)
      .json(&body)
      .send()
      .await
      .map_err(|e| format!("发送 MCP 请求失败: {}", e))?;
    let status = resp.status();
    if !status.is_success() && status.as_u16() != 202 {
      self.pending.lock().await.remove(&id);
      return Err(format!("MCP 请求返回 {}", status));
    }

    let val = tokio::time::timeout(REQUEST_TIMEOUT, rx)
      .await
      .map_err(|_| {
        // best-effort cleanup on timeout
        "MCP SSE 请求超时".to_string()
      })?
      .map_err(|_| "MCP SSE 响应通道关闭".to_string())?;

    if let Some(err) = val.get("error") {
      return Err(err.to_string());
    }
    Ok(val.get("result").cloned().unwrap_or(Value::Null))
  }

  async fn notify(&self, method: &str, params: Value) -> Result<(), String> {
    let body = json!({ "jsonrpc": "2.0", "method": method, "params": params });
    self
      .http
      .post(&self.message_url)
      .json(&body)
      .send()
      .await
      .map_err(|e| format!("发送 MCP 通知失败: {}", e))?;
    Ok(())
  }

  pub async fn list_tools(&self) -> Result<Vec<McpToolInfo>, String> {
    let result = self.request("tools/list", json!({})).await?;
    let tools = result
      .get("tools")
      .and_then(|v| v.as_array())
      .cloned()
      .unwrap_or_default();
    Ok(
      tools
        .into_iter()
        .map(|t| McpToolInfo {
          name: t.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
          description: t
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
          input_schema: t.get("inputSchema").cloned().unwrap_or(Value::Null),
        })
        .collect(),
    )
  }

  pub async fn call_tool(&self, tool: &str, args: Option<Value>) -> Result<String, String> {
    let result = self
      .request(
        "tools/call",
        json!({ "name": tool, "arguments": args.unwrap_or(json!({})) }),
      )
      .await?;

    let is_error = result.get("isError").and_then(|v| v.as_bool()).unwrap_or(false);
    let text = result
      .get("content")
      .and_then(|v| v.as_array())
      .map(|arr| {
        arr
          .iter()
          .map(|c| match c.get("type").and_then(|v| v.as_str()) {
            Some("text") => c.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            Some("image") => "[image]".to_string(),
            Some("resource") => "[resource]".to_string(),
            _ => String::new(),
          })
          .collect::<Vec<_>>()
          .join("\n")
      })
      .unwrap_or_default();

    if is_error {
      Err(if text.is_empty() { "工具返回错误".into() } else { text })
    } else {
      Ok(text)
    }
  }
}
