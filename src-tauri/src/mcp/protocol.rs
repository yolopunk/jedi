// MCP 协议处理
// Phase 3: MCP 客户端实现 - 协议层

use crate::mcp::types::*;
use crate::mcp::transport::{StdioTransport, TransportConfig};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// MCP 客户端状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
  /// 未初始化
  Uninitialized,
  /// 正在初始化
  Initializing,
  /// 已初始化
  Initialized,
  /// 已关闭
  Closed,
}

/// MCP 客户端配置
#[derive(Debug, Clone)]
pub struct McpClientConfig {
  /// 客户端名称
  pub name: String,
  /// 客户端版本
  pub version: String,
  /// 传输层配置
  pub transport_config: TransportConfig,
  /// 客户端能力
  pub capabilities: ClientCapabilities,
}

impl Default for McpClientConfig {
  fn default() -> Self {
    Self {
      name: "jedi-mcp-client".to_string(),
      version: env!("CARGO_PKG_VERSION").to_string(),
      transport_config: TransportConfig::default(),
      capabilities: ClientCapabilities::default(),
    }
  }
}

impl McpClientConfig {
  /// 创建新配置
  pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      version: version.into(),
      ..Default::default()
    }
  }

  /// 设置传输层配置
  pub fn with_transport(mut self, config: TransportConfig) -> Self {
    self.transport_config = config;
    self
  }

  /// 设置客户端能力
  pub fn with_capabilities(mut self, capabilities: ClientCapabilities) -> Self {
    self.capabilities = capabilities;
    self
  }
}

/// MCP 客户端
/// 
/// 提供 MCP 协议的高级接口
pub struct McpClient {
  /// 配置
  config: McpClientConfig,
  /// 传输层
  transport: StdioTransport,
  /// 当前状态
  state: ClientState,
  /// 服务器能力
  server_capabilities: Option<ServerCapabilities>,
  /// 服务器信息
  server_info: Option<Implementation>,
  /// 请求 ID 计数器
  request_id_counter: u64,
}

impl McpClient {
  /// 创建新的 MCP 客户端
  pub fn new(config: McpClientConfig) -> Self {
    let transport = StdioTransport::new(config.transport_config.clone());
    Self {
      config,
      transport,
      state: ClientState::Uninitialized,
      server_capabilities: None,
      server_info: None,
      request_id_counter: 0,
    }
  }

  /// 获取当前状态
  pub fn state(&self) -> ClientState {
    self.state
  }

  /// 检查是否已初始化
  pub fn is_initialized(&self) -> bool {
    self.state == ClientState::Initialized
  }

  /// 获取服务器能力
  pub fn server_capabilities(&self) -> Option<&ServerCapabilities> {
    self.server_capabilities.as_ref()
  }

  /// 获取服务器信息
  pub fn server_info(&self) -> Option<&Implementation> {
    self.server_info.as_ref()
  }

  /// 生成下一个请求 ID
  fn next_request_id(&mut self) -> RequestId {
    self.request_id_counter += 1;
    RequestId::Number(self.request_id_counter)
  }

  /// 启动客户端
  /// 
  /// 启动传输层并初始化连接
  pub fn start(&mut self) -> Result<(), McpError> {
    if self.state != ClientState::Uninitialized {
      return Err(McpError::Protocol("Client already started".to_string()));
    }

    info!(
      name = %self.config.name,
      version = %self.config.version,
      "Starting MCP client"
    );

    // 启动传输层
    self.transport.start()?;

    // 执行初始化握手
    self.initialize()?;

    Ok(())
  }

  /// 停止客户端
  /// 
  /// 关闭连接并停止传输层
  pub fn stop(&mut self) -> Result<(), McpError> {
    if self.state == ClientState::Closed {
      return Ok(());
    }

    info!("Stopping MCP client");

    // 发送关闭通知（可选）
    // self.send_notification(JsonRpcNotification::new("shutdown"))?;

    // 停止传输层
    self.transport.stop()?;

    self.state = ClientState::Closed;
    self.server_capabilities = None;
    self.server_info = None;

    Ok(())
  }

  /// 执行初始化握手
  /// 
  /// 发送 initialize 请求并处理响应
  fn initialize(&mut self) -> Result<(), McpError> {
    self.state = ClientState::Initializing;

    // 构建初始化参数
    let params = InitializeParams::new(Implementation::new(
      &self.config.name,
      &self.config.version,
    ))
    .with_capabilities(self.config.capabilities.clone());

    // 发送初始化请求
    let request = JsonRpcRequest::new(self.next_request_id(), "initialize")
      .with_params(serde_json::to_value(params)?);

    debug!(request = ?request, "Sending initialize request");

    let response = self.transport.send_request(request)?;

    // 检查响应
    if let Some(error) = response.error {
      return Err(McpError::Server(error));
    }

    // 解析响应
    let result: InitializeResult = response
      .result
      .ok_or_else(|| McpError::InvalidResponse("Missing result".to_string()))
      .and_then(|r| serde_json::from_value(r).map_err(McpError::from))?;

    info!(
      protocol_version = %result.protocol_version,
      server_name = %result.server_info.name,
      server_version = %result.server_info.version,
      "MCP server initialized"
    );

    // 保存服务器信息
    self.server_capabilities = Some(result.capabilities);
    self.server_info = Some(result.server_info);

    // 发送 initialized 通知
    let notification = JsonRpcNotification::new("notifications/initialized");
    self.transport.send_notification(notification)?;

    self.state = ClientState::Initialized;

    Ok(())
  }

  /// 获取工具列表
  /// 
  /// 获取服务器提供的所有工具
  pub fn list_tools(&mut self) -> Result<Vec<Tool>, McpError> {
    self.ensure_initialized()?;

    let params = ListToolsParams::default();
    let request = JsonRpcRequest::new(self.next_request_id(), "tools/list")
      .with_params(serde_json::to_value(params)?);

    let response = self.transport.send_request(request)?;

    if let Some(error) = response.error {
      return Err(McpError::Server(error));
    }

    let result: ListToolsResult = response
      .result
      .ok_or_else(|| McpError::InvalidResponse("Missing result".to_string()))
      .and_then(|r| serde_json::from_value(r).map_err(McpError::from))?;

    Ok(result.tools)
  }

  /// 调用工具
  /// 
  /// 调用服务器上的工具
  pub fn call_tool(
    &mut self,
    name: &str,
    arguments: Option<HashMap<String, serde_json::Value>>,
  ) -> Result<CallToolResult, McpError> {
    self.ensure_initialized()?;

    let params = CallToolParams::new(name);
    let params = if let Some(args) = arguments {
      params.with_arguments(args)
    } else {
      params
    };

    let request = JsonRpcRequest::new(self.next_request_id(), "tools/call")
      .with_params(serde_json::to_value(params)?);

    let response = self.transport.send_request(request)?;

    if let Some(error) = response.error {
      return Err(McpError::Server(error));
    }

    let result: CallToolResult = response
      .result
      .ok_or_else(|| McpError::InvalidResponse("Missing result".to_string()))
      .and_then(|r| serde_json::from_value(r).map_err(McpError::from))?;

    Ok(result)
  }

  /// 获取资源列表
  /// 
  /// 获取服务器提供的所有资源
  pub fn list_resources(&mut self) -> Result<Vec<Resource>, McpError> {
    self.ensure_initialized()?;

    let params = ListResourcesParams::default();
    let request = JsonRpcRequest::new(self.next_request_id(), "resources/list")
      .with_params(serde_json::to_value(params)?);

    let response = self.transport.send_request(request)?;

    if let Some(error) = response.error {
      return Err(McpError::Server(error));
    }

    let result: ListResourcesResult = response
      .result
      .ok_or_else(|| McpError::InvalidResponse("Missing result".to_string()))
      .and_then(|r| serde_json::from_value(r).map_err(McpError::from))?;

    Ok(result.resources)
  }

  /// 读取资源
  /// 
  /// 读取指定资源的内容
  pub fn read_resource(&mut self, uri: &str) -> Result<Vec<ResourceContents>, McpError> {
    self.ensure_initialized()?;

    let params = ReadResourceParams {
      uri: uri.to_string(),
    };

    let request = JsonRpcRequest::new(self.next_request_id(), "resources/read")
      .with_params(serde_json::to_value(params)?);

    let response = self.transport.send_request(request)?;

    if let Some(error) = response.error {
      return Err(McpError::Server(error));
    }

    let result: ReadResourceResult = response
      .result
      .ok_or_else(|| McpError::InvalidResponse("Missing result".to_string()))
      .and_then(|r| serde_json::from_value(r).map_err(McpError::from))?;

    Ok(result.contents)
  }

  /// 获取提示列表
  /// 
  /// 获取服务器提供的所有提示
  pub fn list_prompts(&mut self) -> Result<Vec<Prompt>, McpError> {
    self.ensure_initialized()?;

    let params = ListPromptsParams::default();
    let request = JsonRpcRequest::new(self.next_request_id(), "prompts/list")
      .with_params(serde_json::to_value(params)?);

    let response = self.transport.send_request(request)?;

    if let Some(error) = response.error {
      return Err(McpError::Server(error));
    }

    let result: ListPromptsResult = response
      .result
      .ok_or_else(|| McpError::InvalidResponse("Missing result".to_string()))
      .and_then(|r| serde_json::from_value(r).map_err(McpError::from))?;

    Ok(result.prompts)
  }

  /// 获取提示
  /// 
  /// 获取指定提示的内容
  pub fn get_prompt(
    &mut self,
    name: &str,
    arguments: Option<HashMap<String, String>>,
  ) -> Result<GetPromptResult, McpError> {
    self.ensure_initialized()?;

    let params = GetPromptParams {
      name: name.to_string(),
      arguments,
    };

    let request = JsonRpcRequest::new(self.next_request_id(), "prompts/get")
      .with_params(serde_json::to_value(params)?);

    let response = self.transport.send_request(request)?;

    if let Some(error) = response.error {
      return Err(McpError::Server(error));
    }

    let result: GetPromptResult = response
      .result
      .ok_or_else(|| McpError::InvalidResponse("Missing result".to_string()))
      .and_then(|r| serde_json::from_value(r).map_err(McpError::from))?;

    Ok(result)
  }

  /// 设置日志级别
  /// 
  /// 设置服务器的日志级别
  pub fn set_log_level(&mut self, level: LogLevel) -> Result<(), McpError> {
    self.ensure_initialized()?;

    let params = SetLevelParams { level };
    let request = JsonRpcRequest::new(self.next_request_id(), "logging/setLevel")
      .with_params(serde_json::to_value(params)?);

    let response = self.transport.send_request(request)?;

    if let Some(error) = response.error {
      return Err(McpError::Server(error));
    }

    Ok(())
  }

  /// 完成请求
  /// 
  /// 请求服务器完成一个值
  pub fn complete(
    &mut self,
    r#ref: Reference,
    argument: CompleteArgument,
  ) -> Result<CompleteResult, McpError> {
    self.ensure_initialized()?;

    let params = CompleteParams {
      r#ref,
      argument,
    };

    let request = JsonRpcRequest::new(self.next_request_id(), "completion/complete")
      .with_params(serde_json::to_value(params)?);

    let response = self.transport.send_request(request)?;

    if let Some(error) = response.error {
      return Err(McpError::Server(error));
    }

    let result: CompleteResult = response
      .result
      .ok_or_else(|| McpError::InvalidResponse("Missing result".to_string()))
      .and_then(|r| serde_json::from_value(r).map_err(McpError::from))?;

    Ok(result)
  }

  /// 检查是否已初始化
  fn ensure_initialized(&self) -> Result<(), McpError> {
    if self.state != ClientState::Initialized {
      return Err(McpError::NotInitialized);
    }
    Ok(())
  }

  /// 检查服务器是否支持工具
  pub fn supports_tools(&self) -> bool {
    self
      .server_capabilities
      .as_ref()
      .map(|c| c.tools.is_some())
      .unwrap_or(false)
  }

  /// 检查服务器是否支持资源
  pub fn supports_resources(&self) -> bool {
    self
      .server_capabilities
      .as_ref()
      .map(|c| c.resources.is_some())
      .unwrap_or(false)
  }

  /// 检查服务器是否支持提示
  pub fn supports_prompts(&self) -> bool {
    self
      .server_capabilities
      .as_ref()
      .map(|c| c.prompts.is_some())
      .unwrap_or(false)
  }

  /// 检查服务器是否支持日志
  pub fn supports_logging(&self) -> bool {
    self
      .server_capabilities
      .as_ref()
      .map(|c| c.logging.is_some())
      .unwrap_or(false)
  }
}

impl Drop for McpClient {
  fn drop(&mut self) {
    if let Err(e) = self.stop() {
      warn!(error = %e, "Failed to stop MCP client during drop");
    }
  }
}

// ============================================================================
// 构建器
// ============================================================================

/// MCP 客户端构建器
pub struct McpClientBuilder {
  config: McpClientConfig,
}

impl McpClientBuilder {
  /// 创建新构建器
  pub fn new() -> Self {
    Self {
      config: McpClientConfig::default(),
    }
  }

  /// 设置客户端名称
  pub fn name(mut self, name: impl Into<String>) -> Self {
    self.config.name = name.into();
    self
  }

  /// 设置客户端版本
  pub fn version(mut self, version: impl Into<String>) -> Self {
    self.config.version = version.into();
    self
  }

  /// 设置传输层配置
  pub fn transport(mut self, config: TransportConfig) -> Self {
    self.config.transport_config = config;
    self
  }

  /// 设置 MCP 服务器命令
  pub fn server_command(mut self, command: impl Into<String>) -> Self {
    self.config.transport_config.command = command.into();
    self
  }

  /// 设置 MCP 服务器参数
  pub fn server_args(mut self, args: Vec<String>) -> Self {
    self.config.transport_config.args = args;
    self
  }

  /// 设置客户端能力
  pub fn capabilities(mut self, capabilities: ClientCapabilities) -> Self {
    self.config.capabilities = capabilities;
    self
  }

  /// 构建 MCP 客户端
  pub fn build(self) -> McpClient {
    McpClient::new(self.config)
  }
}

impl Default for McpClientBuilder {
  fn default() -> Self {
    Self::new()
  }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_client_state() {
    assert_eq!(ClientState::Uninitialized, ClientState::Uninitialized);
    assert_ne!(ClientState::Uninitialized, ClientState::Initialized);
  }

  #[test]
  fn test_mcp_client_config() {
    let config = McpClientConfig::new("test-client", "1.0.0");
    
    assert_eq!(config.name, "test-client");
    assert_eq!(config.version, "1.0.0");
  }

  #[test]
  fn test_mcp_client_new() {
    let config = McpClientConfig::default();
    let client = McpClient::new(config);

    assert_eq!(client.state(), ClientState::Uninitialized);
    assert!(!client.is_initialized());
    assert!(client.server_capabilities().is_none());
    assert!(client.server_info().is_none());
  }

  #[test]
  fn test_mcp_client_builder() {
    let client = McpClientBuilder::new()
      .name("test-client")
      .version("1.0.0")
      .server_command("node")
      .server_args(vec!["server.js".to_string()])
      .build();

    assert_eq!(client.state(), ClientState::Uninitialized);
  }

  #[test]
  fn test_supports_methods() {
    let config = McpClientConfig::default();
    let client = McpClient::new(config);

    // 未初始化时不支持任何功能
    assert!(!client.supports_tools());
    assert!(!client.supports_resources());
    assert!(!client.supports_prompts());
    assert!(!client.supports_logging());
  }
}