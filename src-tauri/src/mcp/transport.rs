// MCP 传输层实现
// Phase 3: MCP 客户端实现 - 传输层

use crate::mcp::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, McpError};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tracing::{debug, info, warn};

/// MCP 传输层配置
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TransportConfig {
  /// MCP 服务器命令
  pub command: String,
  /// 命令参数
  pub args: Vec<String>,
  /// 环境变量
  pub env: Vec<(String, String)>,
  /// 启动超时（毫秒）
  pub startup_timeout_ms: u64,
  /// 请求超时（毫秒）
  pub request_timeout_ms: u64,
}

impl Default for TransportConfig {
  fn default() -> Self {
    Self {
      command: String::new(),
      args: Vec::new(),
      env: Vec::new(),
      startup_timeout_ms: 5000,
      request_timeout_ms: 30000,
    }
  }
}

#[allow(dead_code)]
impl TransportConfig {
  /// 创建新配置
  pub fn new(command: impl Into<String>) -> Self {
    Self {
      command: command.into(),
      ..Default::default()
    }
  }

  /// 设置参数
  pub fn with_args(mut self, args: Vec<String>) -> Self {
    self.args = args;
    self
  }

  /// 设置环境变量
  pub fn with_env(mut self, env: Vec<(String, String)>) -> Self {
    self.env = env;
    self
  }

  /// 设置启动超时
  pub fn with_startup_timeout(mut self, timeout_ms: u64) -> Self {
    self.startup_timeout_ms = timeout_ms;
    self
  }

  /// 设置请求超时
  pub fn with_request_timeout(mut self, timeout_ms: u64) -> Self {
    self.request_timeout_ms = timeout_ms;
    self
  }
}

/// Stdio 传输层
///
/// 通过子进程的 stdin/stdout 与 MCP 服务器通信
#[allow(dead_code)]
pub struct StdioTransport {
  /// 配置
  config: TransportConfig,
  /// 子进程
  process: Option<Child>,
  /// 标准输入
  stdin: Option<ChildStdin>,
  /// 标准输出读取器
  stdout: Option<BufReader<ChildStdout>>,
  /// 请求 ID 计数器
  request_id_counter: AtomicU64,
  /// 待处理响应
  pending_responses: Mutex<Vec<(u64, Option<JsonRpcResponse>)>>,
}

#[allow(dead_code)]
impl StdioTransport {
  /// 创建新的 Stdio 传输层
  pub fn new(config: TransportConfig) -> Self {
    Self {
      config,
      process: None,
      stdin: None,
      stdout: None,
      request_id_counter: AtomicU64::new(1),
      pending_responses: Mutex::new(Vec::new()),
    }
  }

  /// 启动传输层
  ///
  /// 启动 MCP 服务器子进程
  pub fn start(&mut self) -> Result<(), McpError> {
    if self.process.is_some() {
      return Err(McpError::Transport("Transport already started".to_string()));
    }

    info!(
      command = %self.config.command,
      args = ?self.config.args,
      "Starting MCP server process"
    );

    let mut cmd = Command::new(&self.config.command);
    cmd.args(&self.config.args);

    // 设置环境变量
    for (key, value) in &self.config.env {
      cmd.env(key, value);
    }

    // 设置标准输入输出
    cmd
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped());

    // 启动进程
    let mut process = cmd
      .spawn()
      .map_err(|e| McpError::Transport(format!("Failed to start MCP server: {}", e)))?;

    // 获取标准输入输出
    let stdin = process
      .stdin
      .take()
      .ok_or_else(|| McpError::Transport("Failed to get stdin".to_string()))?;

    let stdout = process
      .stdout
      .take()
      .ok_or_else(|| McpError::Transport("Failed to get stdout".to_string()))?;

    self.stdin = Some(stdin);
    self.stdout = Some(BufReader::new(stdout));
    self.process = Some(process);

    info!("MCP server process started successfully");

    Ok(())
  }

  /// 停止传输层
  ///
  /// 终止 MCP 服务器子进程
  pub fn stop(&mut self) -> Result<(), McpError> {
    if let Some(mut process) = self.process.take() {
      debug!("Stopping MCP server process");

      // 尝试优雅终止
      process
        .kill()
        .map_err(|e| McpError::Transport(format!("Failed to kill MCP server: {}", e)))?;

      self.stdin = None;
      self.stdout = None;

      info!("MCP server process stopped");
    }

    Ok(())
  }

  /// 检查传输层是否已启动
  pub fn is_running(&self) -> bool {
    self.process.is_some()
  }

  /// 生成下一个请求 ID
  fn next_request_id(&self) -> u64 {
    self.request_id_counter.fetch_add(1, Ordering::SeqCst)
  }

  /// 发送请求
  ///
  /// 发送 JSON-RPC 请求并等待响应
  pub fn send_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
    if !self.is_running() {
      return Err(McpError::Transport("Transport not started".to_string()));
    }

    // 序列化请求
    let request_str = serde_json::to_string(&request)?;
    debug!(request = %request_str, "Sending request");

    // 发送请求
    self.write_line(&request_str)?;

    // 读取响应
    let response_str = self.read_line()?;
    debug!(response = %response_str, "Received response");

    // 解析响应
    let response: JsonRpcResponse = serde_json::from_str(&response_str)?;

    Ok(response)
  }

  /// 发送通知
  ///
  /// 发送 JSON-RPC 通知（不等待响应）
  pub fn send_notification(&mut self, notification: JsonRpcNotification) -> Result<(), McpError> {
    if !self.is_running() {
      return Err(McpError::Transport("Transport not started".to_string()));
    }

    // 序列化通知
    let notification_str = serde_json::to_string(&notification)?;
    debug!(notification = %notification_str, "Sending notification");

    // 发送通知
    self.write_line(&notification_str)?;

    Ok(())
  }

  /// 写入一行数据
  fn write_line(&mut self, line: &str) -> Result<(), McpError> {
    if let Some(ref mut stdin) = self.stdin {
      stdin.write_all(line.as_bytes())?;
      stdin.write_all(b"\n")?;
      stdin.flush()?;
      Ok(())
    } else {
      Err(McpError::Transport("stdin not available".to_string()))
    }
  }

  /// 读取一行数据
  fn read_line(&mut self) -> Result<String, McpError> {
    if let Some(ref mut stdout) = self.stdout {
      let mut line = String::new();
      let bytes_read = stdout.read_line(&mut line)?;

      if bytes_read == 0 {
        return Err(McpError::Transport("EOF reached".to_string()));
      }

      // 移除换行符
      let line = line.trim_end().to_string();
      Ok(line)
    } else {
      Err(McpError::Transport("stdout not available".to_string()))
    }
  }

  /// 读取通知
  ///
  /// 尝试读取一个通知（非阻塞）
  pub fn try_read_notification(&mut self) -> Result<Option<JsonRpcNotification>, McpError> {
    if let Some(ref mut stdout) = self.stdout {
      let mut line = String::new();
      let bytes_read = stdout.read_line(&mut line)?;

      if bytes_read == 0 {
        return Err(McpError::Transport("EOF reached".to_string()));
      }

      let line = line.trim_end();
      if line.is_empty() {
        return Ok(None);
      }

      debug!(line = %line, "Received line");

      // 尝试解析为通知
      if let Ok(notification) = serde_json::from_str::<JsonRpcNotification>(line) {
        return Ok(Some(notification));
      }

      // 如果不是通知，可能是响应，忽略
      Ok(None)
    } else {
      Err(McpError::Transport("stdout not available".to_string()))
    }
  }
}

impl Drop for StdioTransport {
  fn drop(&mut self) {
    if let Err(e) = self.stop() {
      warn!(error = %e, "Failed to stop transport during drop");
    }
  }
}

// ============================================================================
// 同步传输层抽象（stdio / sse 共用）
// ============================================================================

/// 同步传输层：McpClient 通过它收发 JSON-RPC。
pub trait Transport: Send {
  fn start(&mut self) -> Result<(), McpError>;
  fn stop(&mut self) -> Result<(), McpError>;
  fn send_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError>;
  fn send_notification(&mut self, notification: JsonRpcNotification) -> Result<(), McpError>;
  #[allow(dead_code)]
  fn is_running(&self) -> bool;
}

impl Transport for StdioTransport {
  fn start(&mut self) -> Result<(), McpError> {
    StdioTransport::start(self)
  }
  fn stop(&mut self) -> Result<(), McpError> {
    StdioTransport::stop(self)
  }
  fn send_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
    StdioTransport::send_request(self, request)
  }
  fn send_notification(&mut self, notification: JsonRpcNotification) -> Result<(), McpError> {
    StdioTransport::send_notification(self, notification)
  }
  fn is_running(&self) -> bool {
    StdioTransport::is_running(self)
  }
}

// ============================================================================
// 异步传输层（预留接口）
// ============================================================================

/// 异步传输层 trait
///
/// 为将来支持异步传输预留接口
#[allow(dead_code)]
mod async_transport {
  use super::*;

  #[async_trait::async_trait]
  pub trait AsyncTransport: Send + Sync {
    /// 启动传输层
    async fn start(&mut self) -> Result<(), McpError>;

    /// 停止传输层
    async fn stop(&mut self) -> Result<(), McpError>;

    /// 发送请求
    async fn send_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError>;

    /// 发送通知
    async fn send_notification(
      &mut self,
      notification: JsonRpcNotification,
    ) -> Result<(), McpError>;

    /// 检查是否运行中
    fn is_running(&self) -> bool;
  }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_transport_config() {
    let config = TransportConfig::new("node")
      .with_args(vec!["server.js".to_string()])
      .with_env(vec![("NODE_ENV".to_string(), "production".to_string())])
      .with_startup_timeout(10000)
      .with_request_timeout(60000);

    assert_eq!(config.command, "node");
    assert_eq!(config.args, vec!["server.js"]);
    assert_eq!(config.env.len(), 1);
    assert_eq!(config.startup_timeout_ms, 10000);
    assert_eq!(config.request_timeout_ms, 60000);
  }

  #[test]
  fn test_stdio_transport_new() {
    let config = TransportConfig::new("test-command");
    let transport = StdioTransport::new(config);

    assert!(!transport.is_running());
    assert!(transport.process.is_none());
    assert!(transport.stdin.is_none());
    assert!(transport.stdout.is_none());
  }

  #[test]
  fn test_next_request_id() {
    let config = TransportConfig::new("test-command");
    let transport = StdioTransport::new(config);

    let id1 = transport.next_request_id();
    let id2 = transport.next_request_id();
    let id3 = transport.next_request_id();

    assert!(id2 > id1);
    assert!(id3 > id2);
  }
}
