// MCP Streamable HTTP 传输层（协议 2025-03-26）
//
// 与 2024-11-05 的 HTTP+SSE 不同，这里只有**一个端点**：
//   - 客户端把 JSON-RPC 消息 POST 到该端点，Accept: application/json, text/event-stream
//   - 服务器可以直接回 `application/json`（单条响应），也可以回 `text/event-stream`
//     （在流里回推响应），两种都要支持
//   - initialize 的响应头可能带 `Mcp-Session-Id`，后续请求需回传
//   - 通知（无 id）POST 后服务器返回 202，无响应体
//
// 沿用同步阻塞的 Transport 抽象（reqwest::blocking），与 stdio / SSE 可互换。

use crate::mcp::sse_transport::{build_header_map, id_key};
use crate::mcp::transport::Transport;
use crate::mcp::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, McpError};
use std::io::{BufRead, BufReader};
use std::time::Duration;
use tracing::debug;

const SESSION_HEADER: &str = "mcp-session-id";

/// Streamable HTTP 传输配置
#[derive(Debug, Clone)]
pub struct StreamableHttpConfig {
  pub url: String,
  pub headers: Vec<(String, String)>,
  pub request_timeout_ms: u64,
}

impl StreamableHttpConfig {
  pub fn new(url: impl Into<String>) -> Self {
    Self {
      url: url.into(),
      headers: Vec::new(),
      request_timeout_ms: 30_000,
    }
  }

  pub fn with_headers(mut self, headers: Vec<(String, String)>) -> Self {
    self.headers = headers;
    self
  }
}

pub struct StreamableHttpTransport {
  config: StreamableHttpConfig,
  http: reqwest::blocking::Client,
  session_id: Option<String>,
  running: bool,
}

impl StreamableHttpTransport {
  pub fn new(config: StreamableHttpConfig) -> Self {
    let http = reqwest::blocking::Client::builder()
      .timeout(Duration::from_millis(config.request_timeout_ms))
      .build()
      .unwrap_or_else(|_| reqwest::blocking::Client::new());
    Self {
      config,
      http,
      session_id: None,
      running: false,
    }
  }

  fn post(&self, body: String) -> Result<reqwest::blocking::Response, McpError> {
    let mut rb = self
      .http
      .post(&self.config.url)
      .headers(build_header_map(&self.config.headers))
      .header("Content-Type", "application/json")
      .header("Accept", "application/json, text/event-stream")
      .body(body);
    if let Some(sid) = &self.session_id {
      rb = rb.header("Mcp-Session-Id", sid.clone());
    }
    rb.send()
      .map_err(|e| McpError::Transport(format!("POST 失败: {}", e)))
  }
}

/// 从 SSE 响应体里读出 id 匹配的那条 JSON-RPC 响应（逐行读，命中即返回）
fn read_sse_response(
  resp: reqwest::blocking::Response,
  want_id: &str,
) -> Result<JsonRpcResponse, McpError> {
  let mut reader = BufReader::new(resp);
  let mut data = String::new();
  let mut line = String::new();

  loop {
    line.clear();
    match reader.read_line(&mut line) {
      Ok(0) => break, // EOF
      Ok(_) => {}
      Err(e) => return Err(McpError::Transport(format!("读取 SSE 失败: {}", e))),
    }
    let trimmed = line.trim_end_matches(['\r', '\n']);

    if trimmed.is_empty() {
      // 事件结束 → 尝试解析
      if !data.is_empty() {
        if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&data) {
          if id_key(&resp.id) == want_id {
            return Ok(resp);
          }
        }
        data.clear();
      }
      continue;
    }
    if let Some(rest) = trimmed.strip_prefix("data:") {
      if !data.is_empty() {
        data.push('\n');
      }
      data.push_str(rest.strip_prefix(' ').unwrap_or(rest));
    }
    // event: / id: / 注释行忽略
  }

  // 流结束前最后一个事件可能没有空行收尾
  if !data.is_empty() {
    if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&data) {
      if id_key(&resp.id) == want_id {
        return Ok(resp);
      }
    }
  }
  Err(McpError::Transport(format!(
    "SSE 流结束但未收到 id={} 的响应",
    want_id
  )))
}

impl Transport for StreamableHttpTransport {
  fn start(&mut self) -> Result<(), McpError> {
    // Streamable HTTP 无需预先建立长连接：会话在首个 POST（initialize）时建立
    self.running = true;
    Ok(())
  }

  fn stop(&mut self) -> Result<(), McpError> {
    // 有会话则尽力通知服务器结束（失败不影响本地关闭）
    if let Some(sid) = self.session_id.take() {
      let _ = self
        .http
        .delete(&self.config.url)
        .headers(build_header_map(&self.config.headers))
        .header("Mcp-Session-Id", sid)
        .send();
    }
    self.running = false;
    Ok(())
  }

  fn send_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
    let want = id_key(&request.id);
    let body = serde_json::to_string(&request)?;
    let resp = self.post(body)?;

    if !resp.status().is_success() {
      return Err(McpError::Transport(format!(
        "POST 返回状态 {}",
        resp.status()
      )));
    }

    // 消费 body 前先取出需要的响应头
    if let Some(sid) = resp
      .headers()
      .get(SESSION_HEADER)
      .and_then(|v| v.to_str().ok())
      .map(|s| s.to_string())
    {
      self.session_id = Some(sid);
    }
    let content_type = resp
      .headers()
      .get(reqwest::header::CONTENT_TYPE)
      .and_then(|v| v.to_str().ok())
      .unwrap_or("")
      .to_lowercase();

    if content_type.contains("text/event-stream") {
      read_sse_response(resp, &want)
    } else {
      let text = resp
        .text()
        .map_err(|e| McpError::Transport(format!("读取响应体失败: {}", e)))?;
      serde_json::from_str(&text).map_err(McpError::from)
    }
  }

  fn send_notification(&mut self, notification: JsonRpcNotification) -> Result<(), McpError> {
    let body = serde_json::to_string(&notification)?;
    let resp = self.post(body)?;
    if !resp.status().is_success() {
      debug!(status = %resp.status(), "Streamable HTTP 通知 POST 非 2xx");
    }
    Ok(())
  }

  fn is_running(&self) -> bool {
    self.running
  }
}

impl Drop for StreamableHttpTransport {
  fn drop(&mut self) {
    let _ = self.stop();
  }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use crate::mcp::types::RequestId;

  #[test]
  fn test_config_builder() {
    let c = StreamableHttpConfig::new("http://x/mcp")
      .with_headers(vec![("Authorization".into(), "Bearer t".into())]);
    assert_eq!(c.url, "http://x/mcp");
    assert_eq!(c.headers.len(), 1);
    assert_eq!(c.request_timeout_ms, 30_000);
  }

  #[test]
  fn test_transport_not_running_before_start() {
    let t = StreamableHttpTransport::new(StreamableHttpConfig::new("http://x/mcp"));
    assert!(!t.is_running());
  }

  #[test]
  fn test_start_marks_running_without_connection() {
    // 不同于 SSE，start 不需要真的连上服务器
    let mut t = StreamableHttpTransport::new(StreamableHttpConfig::new("http://127.0.0.1:1/mcp"));
    assert!(t.start().is_ok());
    assert!(t.is_running());
    assert!(t.stop().is_ok());
    assert!(!t.is_running());
  }

  #[test]
  fn test_id_key_matching_is_shared_with_sse() {
    assert_eq!(id_key(&RequestId::Number(3)), "3");
    assert_eq!(id_key(&RequestId::String("x".into())), "x");
  }
}
