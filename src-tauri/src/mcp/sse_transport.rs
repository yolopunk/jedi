// MCP HTTP+SSE 传输层（协议 2024-11-05）
//
// 连接模型：
//   1. GET <url> 打开 text/event-stream；服务器先发 `event: endpoint`，data 为消息 POST 地址
//   2. 客户端把 JSON-RPC 请求 POST 到该地址（服务器返回 202，不带响应体）
//   3. 实际的 JSON-RPC 响应由服务器经 SSE 流以 `event: message` 推回
//
// 因 McpClient 为同步阻塞式，这里用 reqwest::blocking + 一个后台读取线程实现。

use crate::mcp::transport::Transport;
use crate::mcp::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, McpError, RequestId};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// SSE 传输配置
#[derive(Debug, Clone)]
pub struct SseConfig {
  pub url: String,
  pub headers: Vec<(String, String)>,
  pub startup_timeout_ms: u64,
  pub request_timeout_ms: u64,
}

impl SseConfig {
  pub fn new(url: impl Into<String>) -> Self {
    Self {
      url: url.into(),
      headers: Vec::new(),
      startup_timeout_ms: 10_000,
      request_timeout_ms: 30_000,
    }
  }

  pub fn with_headers(mut self, headers: Vec<(String, String)>) -> Self {
    self.headers = headers;
    self
  }
}

/// 请求 id → 字符串键
pub(crate) fn id_key(id: &RequestId) -> String {
  match id {
    RequestId::Number(n) => n.to_string(),
    RequestId::String(s) => s.clone(),
  }
}

/// 把 endpoint（可能是相对路径）解析为绝对 URL
pub fn resolve_endpoint(base: &str, path: &str) -> String {
  if path.starts_with("http://") || path.starts_with("https://") {
    return path.to_string();
  }
  // origin = scheme://host[:port]
  if let Some(scheme_end) = base.find("://") {
    let after = scheme_end + 3;
    let origin_end = base[after..]
      .find('/')
      .map(|i| after + i)
      .unwrap_or(base.len());
    let origin = &base[..origin_end];
    if path.starts_with('/') {
      return format!("{}{}", origin, path);
    }
    return format!("{}/{}", origin, path);
  }
  path.to_string()
}

pub(crate) fn build_header_map(headers: &[(String, String)]) -> HeaderMap {
  let mut map = HeaderMap::new();
  for (k, v) in headers {
    if let (Ok(name), Ok(val)) = (
      HeaderName::from_bytes(k.as_bytes()),
      HeaderValue::from_str(v),
    ) {
      map.insert(name, val);
    }
  }
  map
}

/// 后台线程与主线程共享的状态
struct Shared {
  endpoint: Mutex<Option<String>>,
  responses: Mutex<HashMap<String, JsonRpcResponse>>,
  cv: Condvar,
  stop: AtomicBool,
}

impl Shared {
  fn new() -> Self {
    Self {
      endpoint: Mutex::new(None),
      responses: Mutex::new(HashMap::new()),
      cv: Condvar::new(),
      stop: AtomicBool::new(false),
    }
  }
}

/// SSE 传输层
pub struct SseTransport {
  config: SseConfig,
  http: reqwest::blocking::Client,
  shared: Arc<Shared>,
  reader: Option<JoinHandle<()>>,
  running: bool,
}

impl SseTransport {
  pub fn new(config: SseConfig) -> Self {
    Self {
      config,
      http: reqwest::blocking::Client::new(),
      shared: Arc::new(Shared::new()),
      reader: None,
      running: false,
    }
  }

  fn post_endpoint(&self) -> Result<String, McpError> {
    self
      .shared
      .endpoint
      .lock()
      .map_err(|e| McpError::Transport(e.to_string()))?
      .clone()
      .ok_or_else(|| McpError::Transport("SSE endpoint 尚未就绪".to_string()))
  }
}

/// 后台读取 SSE 流，解析事件
fn run_reader(base_url: String, resp: reqwest::blocking::Response, shared: Arc<Shared>) {
  let mut reader = BufReader::new(resp);
  let mut event = String::from("message");
  let mut data = String::new();

  let mut line = String::new();
  loop {
    if shared.stop.load(Ordering::Relaxed) {
      break;
    }
    line.clear();
    match reader.read_line(&mut line) {
      Ok(0) => break, // EOF
      Ok(_) => {}
      Err(_) => break,
    }
    let trimmed = line.trim_end_matches(['\r', '\n']);

    if trimmed.is_empty() {
      // 事件结束 → 分派
      if !data.is_empty() {
        dispatch(&base_url, &event, &data, &shared);
      }
      event = String::from("message");
      data.clear();
      continue;
    }
    if let Some(rest) = trimmed.strip_prefix("event:") {
      event = rest.trim().to_string();
    } else if let Some(rest) = trimmed.strip_prefix("data:") {
      if !data.is_empty() {
        data.push('\n');
      }
      data.push_str(rest.strip_prefix(' ').unwrap_or(rest));
    }
    // 其它字段（id: / : 注释）忽略
  }
}

fn dispatch(base_url: &str, event: &str, data: &str, shared: &Arc<Shared>) {
  match event {
    "endpoint" => {
      let abs = resolve_endpoint(base_url, data.trim());
      if let Ok(mut ep) = shared.endpoint.lock() {
        *ep = Some(abs);
      }
      shared.cv.notify_all();
    }
    _ => {
      // message：应为 JSON-RPC 响应
      if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(data) {
        let key = id_key(&resp.id);
        if let Ok(mut map) = shared.responses.lock() {
          map.insert(key, resp);
        }
        shared.cv.notify_all();
      }
    }
  }
}

impl Transport for SseTransport {
  fn start(&mut self) -> Result<(), McpError> {
    if self.running {
      return Err(McpError::Transport("SSE 传输已启动".to_string()));
    }

    let headers = build_header_map(&self.config.headers);
    let resp = self
      .http
      .get(&self.config.url)
      .headers(headers)
      .send()
      .map_err(|e| McpError::Transport(format!("SSE 连接失败: {}", e)))?;

    if !resp.status().is_success() {
      return Err(McpError::Transport(format!(
        "SSE 连接返回状态 {}",
        resp.status()
      )));
    }

    let base = self.config.url.clone();
    let shared = self.shared.clone();
    self.reader = Some(std::thread::spawn(move || run_reader(base, resp, shared)));
    self.running = true;

    // 等待 endpoint 就绪
    let deadline = Instant::now() + Duration::from_millis(self.config.startup_timeout_ms);
    let mut guard = self
      .shared
      .endpoint
      .lock()
      .map_err(|e| McpError::Transport(e.to_string()))?;
    while guard.is_none() {
      let now = Instant::now();
      if now >= deadline {
        return Err(McpError::Transport("等待 SSE endpoint 超时".to_string()));
      }
      let (g, timeout) = self
        .shared
        .cv
        .wait_timeout(guard, deadline - now)
        .map_err(|e| McpError::Transport(e.to_string()))?;
      guard = g;
      if timeout.timed_out() && guard.is_none() {
        return Err(McpError::Transport("等待 SSE endpoint 超时".to_string()));
      }
    }
    Ok(())
  }

  fn stop(&mut self) -> Result<(), McpError> {
    self.shared.stop.store(true, Ordering::Relaxed);
    self.running = false;
    // 不 join：读取线程阻塞在 read 上，连接关闭后自行退出
    self.reader = None;
    Ok(())
  }

  fn send_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
    let endpoint = self.post_endpoint()?;
    let key = id_key(&request.id);
    let body = serde_json::to_string(&request)?;

    let headers = build_header_map(&self.config.headers);
    let resp = self
      .http
      .post(&endpoint)
      .headers(headers)
      .header("Content-Type", "application/json")
      .body(body)
      .send()
      .map_err(|e| McpError::Transport(format!("POST 失败: {}", e)))?;
    if !resp.status().is_success() {
      return Err(McpError::Transport(format!("POST 返回状态 {}", resp.status())));
    }

    // 等待经 SSE 回推的响应
    let deadline = Instant::now() + Duration::from_millis(self.config.request_timeout_ms);
    let mut map = self
      .shared
      .responses
      .lock()
      .map_err(|e| McpError::Transport(e.to_string()))?;
    loop {
      if let Some(resp) = map.remove(&key) {
        return Ok(resp);
      }
      let now = Instant::now();
      if now >= deadline {
        return Err(McpError::Timeout(format!("等待响应 {} 超时", key)));
      }
      let (m, timeout) = self
        .shared
        .cv
        .wait_timeout(map, deadline - now)
        .map_err(|e| McpError::Transport(e.to_string()))?;
      map = m;
      if timeout.timed_out() && !map.contains_key(&key) {
        return Err(McpError::Timeout(format!("等待响应 {} 超时", key)));
      }
    }
  }

  fn send_notification(&mut self, notification: JsonRpcNotification) -> Result<(), McpError> {
    let endpoint = self.post_endpoint()?;
    let body = serde_json::to_string(&notification)?;
    let headers = build_header_map(&self.config.headers);
    let resp = self
      .http
      .post(&endpoint)
      .headers(headers)
      .header("Content-Type", "application/json")
      .body(body)
      .send()
      .map_err(|e| McpError::Transport(format!("POST 通知失败: {}", e)))?;
    if !resp.status().is_success() {
      debug!(status = %resp.status(), "SSE 通知 POST 非 2xx");
    }
    Ok(())
  }

  fn is_running(&self) -> bool {
    self.running
  }
}

impl Drop for SseTransport {
  fn drop(&mut self) {
    if let Err(e) = self.stop() {
      warn!(error = %e, "停止 SSE 传输失败");
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
  fn test_resolve_endpoint_relative() {
    assert_eq!(
      resolve_endpoint("http://127.0.0.1:9000/sse", "/messages?sid=1"),
      "http://127.0.0.1:9000/messages?sid=1"
    );
  }

  #[test]
  fn test_resolve_endpoint_absolute() {
    assert_eq!(
      resolve_endpoint("http://x/sse", "http://other/msg"),
      "http://other/msg"
    );
  }

  #[test]
  fn test_resolve_endpoint_no_leading_slash() {
    assert_eq!(
      resolve_endpoint("https://host:8080/sse", "messages"),
      "https://host:8080/messages"
    );
  }

  #[test]
  fn test_id_key() {
    assert_eq!(id_key(&RequestId::Number(7)), "7");
    assert_eq!(id_key(&RequestId::String("abc".into())), "abc");
  }
}
