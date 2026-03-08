// MCP 协议类型定义
// Phase 3: MCP 客户端实现 - 类型系统

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP 协议版本
pub const MCP_VERSION: &str = "2024-11-05";

/// JSON-RPC 版本
pub const JSONRPC_VERSION: &str = "2.0";

// ============================================================================
// JSON-RPC 基础类型
// ============================================================================

/// JSON-RPC 请求 ID
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
  /// 数字 ID
  Number(i64),
  /// 字符串 ID
  String(String),
}

impl Default for RequestId {
  fn default() -> Self {
    RequestId::Number(0)
  }
}

impl From<i64> for RequestId {
  fn from(n: i64) -> Self {
    RequestId::Number(n)
  }
}

impl From<String> for RequestId {
  fn from(s: String) -> Self {
    RequestId::String(s)
  }
}

impl From<&str> for RequestId {
  fn from(s: &str) -> Self {
    RequestId::String(s.to_string())
  }
}

/// JSON-RPC 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
  /// JSON-RPC 版本
  pub jsonrpc: String,
  /// 请求 ID
  pub id: RequestId,
  /// 方法名
  pub method: String,
  /// 参数
  #[serde(skip_serializing_if = "Option::is_none")]
  pub params: Option<serde_json::Value>,
}

impl JsonRpcRequest {
  /// 创建新的 JSON-RPC 请求
  pub fn new(id: RequestId, method: impl Into<String>) -> Self {
    Self {
      jsonrpc: JSONRPC_VERSION.to_string(),
      id,
      method: method.into(),
      params: None,
    }
  }

  /// 设置参数
  pub fn with_params(mut self, params: serde_json::Value) -> Self {
    self.params = Some(params);
    self
  }
}

/// JSON-RPC 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
  /// JSON-RPC 版本
  pub jsonrpc: String,
  /// 请求 ID
  pub id: RequestId,
  /// 结果（成功时）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub result: Option<serde_json::Value>,
  /// 错误（失败时）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
  /// 创建成功响应
  pub fn success(id: RequestId, result: serde_json::Value) -> Self {
    Self {
      jsonrpc: JSONRPC_VERSION.to_string(),
      id,
      result: Some(result),
      error: None,
    }
  }

  /// 创建错误响应
  pub fn error(id: RequestId, error: JsonRpcError) -> Self {
    Self {
      jsonrpc: JSONRPC_VERSION.to_string(),
      id,
      result: None,
      error: Some(error),
    }
  }

  /// 检查是否成功
  pub fn is_success(&self) -> bool {
    self.error.is_none() && self.result.is_some()
  }
}

/// JSON-RPC 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
  /// 错误码
  pub code: i64,
  /// 错误消息
  pub message: String,
  /// 额外数据
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
  /// 解析错误
  pub const PARSE_ERROR: i64 = -32700;
  /// 无效请求
  pub const INVALID_REQUEST: i64 = -32600;
  /// 方法未找到
  pub const METHOD_NOT_FOUND: i64 = -32601;
  /// 无效参数
  pub const INVALID_PARAMS: i64 = -32602;
  /// 内部错误
  pub const INTERNAL_ERROR: i64 = -32603;

  /// 创建新错误
  pub fn new(code: i64, message: impl Into<String>) -> Self {
    Self {
      code,
      message: message.into(),
      data: None,
    }
  }

  /// 设置额外数据
  pub fn with_data(mut self, data: serde_json::Value) -> Self {
    self.data = Some(data);
    self
  }

  /// 创建解析错误
  pub fn parse_error() -> Self {
    Self::new(Self::PARSE_ERROR, "Parse error")
  }

  /// 创建无效请求错误
  pub fn invalid_request() -> Self {
    Self::new(Self::INVALID_REQUEST, "Invalid request")
  }

  /// 创建方法未找到错误
  pub fn method_not_found(method: &str) -> Self {
    Self::new(
      Self::METHOD_NOT_FOUND,
      format!("Method not found: {}", method),
    )
  }

  /// 创建无效参数错误
  pub fn invalid_params(message: impl Into<String>) -> Self {
    Self::new(Self::INVALID_PARAMS, message)
  }

  /// 创建内部错误
  pub fn internal_error(message: impl Into<String>) -> Self {
    Self::new(Self::INTERNAL_ERROR, message)
  }
}

/// JSON-RPC 通知（无 ID）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
  /// JSON-RPC 版本
  pub jsonrpc: String,
  /// 方法名
  pub method: String,
  /// 参数
  #[serde(skip_serializing_if = "Option::is_none")]
  pub params: Option<serde_json::Value>,
}

impl JsonRpcNotification {
  /// 创建新通知
  pub fn new(method: impl Into<String>) -> Self {
    Self {
      jsonrpc: JSONRPC_VERSION.to_string(),
      method: method.into(),
      params: None,
    }
  }

  /// 设置参数
  pub fn with_params(mut self, params: serde_json::Value) -> Self {
    self.params = Some(params);
    self
  }
}

// ============================================================================
// MCP 协议类型
// ============================================================================

/// MCP 客户端能力
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
  /// 实验性功能
  #[serde(skip_serializing_if = "Option::is_none")]
  pub experimental: Option<HashMap<String, serde_json::Value>>,
  /// 根目录支持
  #[serde(skip_serializing_if = "Option::is_none")]
  pub roots: Option<RootsCapability>,
  /// 采样支持
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sampling: Option<serde_json::Value>,
}

/// 根目录能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsCapability {
  /// 是否支持列表变更通知
  #[serde(skip_serializing_if = "Option::is_none")]
  pub list_changed: Option<bool>,
}

/// MCP 服务器能力
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
  /// 实验性功能
  #[serde(skip_serializing_if = "Option::is_none")]
  pub experimental: Option<HashMap<String, serde_json::Value>>,
  /// 日志支持
  #[serde(skip_serializing_if = "Option::is_none")]
  pub logging: Option<serde_json::Value>,
  /// 提示支持
  #[serde(skip_serializing_if = "Option::is_none")]
  pub prompts: Option<PromptsCapability>,
  /// 资源支持
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resources: Option<ResourcesCapability>,
  /// 工具支持
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools: Option<ToolsCapability>,
}

/// 提示能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
  /// 是否支持列表变更通知
  #[serde(skip_serializing_if = "Option::is_none")]
  pub list_changed: Option<bool>,
}

/// 资源能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
  /// 是否支持订阅
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subscribe: Option<bool>,
  /// 是否支持列表变更通知
  #[serde(skip_serializing_if = "Option::is_none")]
  pub list_changed: Option<bool>,
}

/// 工具能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
  /// 是否支持列表变更通知
  #[serde(skip_serializing_if = "Option::is_none")]
  pub list_changed: Option<bool>,
}

/// 实现信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
  /// 名称
  pub name: String,
  /// 版本
  pub version: String,
}

impl Implementation {
  /// 创建新实现信息
  pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      version: version.into(),
    }
  }
}

// ============================================================================
// 初始化相关类型
// ============================================================================

/// 初始化请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
  /// 协议版本
  pub protocol_version: String,
  /// 客户端能力
  pub capabilities: ClientCapabilities,
  /// 客户端实现信息
  pub client_info: Implementation,
}

impl InitializeParams {
  /// 创建新的初始化参数
  pub fn new(client_info: Implementation) -> Self {
    Self {
      protocol_version: MCP_VERSION.to_string(),
      capabilities: ClientCapabilities::default(),
      client_info,
    }
  }

  /// 设置客户端能力
  pub fn with_capabilities(mut self, capabilities: ClientCapabilities) -> Self {
    self.capabilities = capabilities;
    self
  }
}

/// 初始化响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
  /// 协议版本
  pub protocol_version: String,
  /// 服务器能力
  pub capabilities: ServerCapabilities,
  /// 服务器实现信息
  pub server_info: Implementation,
  /// 服务器指令
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instructions: Option<String>,
}

// ============================================================================
// 工具相关类型
// ============================================================================

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
  /// 工具名称
  pub name: String,
  /// 工具描述
  pub description: Option<String>,
  /// 输入模式（JSON Schema）
  pub input_schema: ToolInputSchema,
}

/// 工具输入模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
  /// 模式类型
  #[serde(rename = "type")]
  pub schema_type: String,
  /// 属性定义
  #[serde(skip_serializing_if = "Option::is_none")]
  pub properties: Option<HashMap<String, serde_json::Value>>,
  /// 必需属性
  #[serde(skip_serializing_if = "Option::is_none")]
  pub required: Option<Vec<String>>,
}

impl ToolInputSchema {
  /// 创建新的输入模式
  pub fn new() -> Self {
    Self {
      schema_type: "object".to_string(),
      properties: None,
      required: None,
    }
  }

  /// 设置属性
  pub fn with_properties(mut self, properties: HashMap<String, serde_json::Value>) -> Self {
    self.properties = Some(properties);
    self
  }

  /// 设置必需属性
  pub fn with_required(mut self, required: Vec<String>) -> Self {
    self.required = Some(required);
    self
  }
}

impl Default for ToolInputSchema {
  fn default() -> Self {
    Self::new()
  }
}

/// 工具列表请求参数
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListToolsParams {
  /// 游标（用于分页）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cursor: Option<String>,
}

/// 工具列表响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
  /// 工具列表
  pub tools: Vec<Tool>,
  /// 下一页游标
  #[serde(skip_serializing_if = "Option::is_none")]
  pub next_cursor: Option<String>,
}

/// 工具调用请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
  /// 工具名称
  pub name: String,
  /// 工具参数
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arguments: Option<HashMap<String, serde_json::Value>>,
}

impl CallToolParams {
  /// 创建新的工具调用参数
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      arguments: None,
    }
  }

  /// 设置参数
  pub fn with_arguments(mut self, arguments: HashMap<String, serde_json::Value>) -> Self {
    self.arguments = Some(arguments);
    self
  }
}

/// 工具调用响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
  /// 内容列表
  pub content: Vec<Content>,
  /// 是否错误
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_error: Option<bool>,
}

/// 内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
  /// 文本内容
  #[serde(rename = "text")]
  Text {
    /// 文本内容
    text: String,
  },
  /// 图片内容
  #[serde(rename = "image")]
  Image {
    /// 数据（base64）
    data: String,
    /// MIME 类型
    mime_type: String,
  },
  /// 资源内容
  #[serde(rename = "resource")]
  Resource {
    /// 资源
    resource: ResourceContents,
  },
}

impl Content {
  /// 创建文本内容
  pub fn text(text: impl Into<String>) -> Self {
    Content::Text { text: text.into() }
  }

  /// 创建图片内容
  pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
    Content::Image {
      data: data.into(),
      mime_type: mime_type.into(),
    }
  }
}

// ============================================================================
// 资源相关类型
// ============================================================================

/// 资源定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
  /// 资源 URI
  pub uri: String,
  /// 资源名称
  pub name: String,
  /// 资源描述
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// MIME 类型
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mime_type: Option<String>,
}

/// 资源内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContents {
  /// 资源 URI
  pub uri: String,
  /// MIME 类型
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mime_type: Option<String>,
  /// 文本内容
  #[serde(skip_serializing_if = "Option::is_none")]
  pub text: Option<String>,
  /// 二进制内容（base64）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub blob: Option<String>,
}

/// 资源列表请求参数
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListResourcesParams {
  /// 游标（用于分页）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cursor: Option<String>,
}

/// 资源列表响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResult {
  /// 资源列表
  pub resources: Vec<Resource>,
  /// 下一页游标
  #[serde(skip_serializing_if = "Option::is_none")]
  pub next_cursor: Option<String>,
}

/// 读取资源请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceParams {
  /// 资源 URI
  pub uri: String,
}

/// 读取资源响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
  /// 内容列表
  pub contents: Vec<ResourceContents>,
}

// ============================================================================
// 提示相关类型
// ============================================================================

/// 提示定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
  /// 提示名称
  pub name: String,
  /// 提示描述
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// 参数列表
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arguments: Option<Vec<PromptArgument>>,
}

/// 提示参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
  /// 参数名称
  pub name: String,
  /// 参数描述
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// 是否必需
  #[serde(skip_serializing_if = "Option::is_none")]
  pub required: Option<bool>,
}

/// 提示列表请求参数
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListPromptsParams {
  /// 游标（用于分页）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cursor: Option<String>,
}

/// 提示列表响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResult {
  /// 提示列表
  pub prompts: Vec<Prompt>,
  /// 下一页游标
  #[serde(skip_serializing_if = "Option::is_none")]
  pub next_cursor: Option<String>,
}

/// 获取提示请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptParams {
  /// 提示名称
  pub name: String,
  /// 参数值
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arguments: Option<HashMap<String, String>>,
}

/// 获取提示响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
  /// 提示描述
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// 消息列表
  pub messages: Vec<PromptMessage>,
}

/// 提示消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
  /// 角色
  pub role: PromptMessageRole,
  /// 内容
  pub content: Content,
}

/// 提示消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptMessageRole {
  /// 用户
  User,
  /// 助手
  Assistant,
}

// ============================================================================
// 日志相关类型
// ============================================================================

/// 日志级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
  /// 调试
  Debug,
  /// 信息
  Info,
  /// 注意
  Notice,
  /// 警告
  Warning,
  /// 错误
  Error,
  /// 严重
  Critical,
  /// 警报
  Alert,
  /// 紧急
  Emergency,
}

/// 设置日志级别请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLevelParams {
  /// 日志级别
  pub level: LogLevel,
}

// ============================================================================
// 完成相关类型
// ============================================================================

/// 完成请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteParams {
  /// 引用
  pub r#ref: Reference,
  /// 参数
  pub argument: CompleteArgument,
}

/// 引用类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Reference {
  /// 资源引用
  #[serde(rename = "ref/resource")]
  Resource {
    /// 资源 URI
    uri: String,
  },
  /// 提示引用
  #[serde(rename = "ref/prompt")]
  Prompt {
    /// 提示名称
    name: String,
  },
}

/// 完成参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteArgument {
  /// 参数名称
  pub name: String,
  /// 参数值
  pub value: String,
}

/// 完成响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteResult {
  /// 完成值
  pub completion: CompleteValues,
  /// 是否总结果数
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total: Option<i64>,
  /// 是否有更多
  #[serde(skip_serializing_if = "Option::is_none")]
  pub has_more: Option<bool>,
}

/// 完成值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteValues {
  /// 值列表
  pub values: Vec<String>,
  /// 是否有更多
  #[serde(skip_serializing_if = "Option::is_none")]
  pub has_more: Option<bool>,
  /// 总数
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total: Option<i64>,
}

// ============================================================================
// 通知类型
// ============================================================================

/// 通知方法名
pub mod notifications {
  /// 工具列表变更通知
  pub const TOOLS_LIST_CHANGED: &str = "notifications/tools/list_changed";
  /// 资源列表变更通知
  pub const RESOURCES_LIST_CHANGED: &str = "notifications/resources/list_changed";
  /// 提示列表变更通知
  pub const PROMPTS_LIST_CHANGED: &str = "notifications/prompts/list_changed";
  /// 日志通知
  pub const LOGGING_MESSAGE: &str = "notifications/message";
  /// 资源更新通知
  pub const RESOURCE_UPDATED: &str = "notifications/resources/updated";
  /// 根目录列表变更通知
  pub const ROOTS_LIST_CHANGED: &str = "notifications/roots/list_changed";
  /// 进度通知
  pub const PROGRESS: &str = "notifications/progress";
  /// 取消通知
  pub const CANCELLED: &str = "notifications/cancelled";
}

/// 日志通知参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingMessageParams {
  /// 日志级别
  pub level: LogLevel,
  /// 日志数据
  pub data: serde_json::Value,
  /// 日志来源
  #[serde(skip_serializing_if = "Option::is_none")]
  pub logger: Option<String>,
}

/// 进度通知参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressParams {
  /// 进度令牌
  pub progress_token: String,
  /// 当前进度
  pub progress: f64,
  /// 总进度
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total: Option<f64>,
}

/// 取消通知参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelledParams {
  /// 请求 ID
  pub request_id: RequestId,
  /// 取消原因
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reason: Option<String>,
}

/// 资源更新通知参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdatedParams {
  /// 资源 URI
  pub uri: String,
}

// ============================================================================
// 错误类型
// ============================================================================

/// MCP 错误
#[derive(Debug, Clone)]
pub enum McpError {
  /// 传输错误
  Transport(String),
  /// 协议错误
  Protocol(String),
  /// 序列化错误
  Serialization(String),
  /// IO 错误
  Io(String),
  /// 超时错误
  Timeout(String),
  /// 服务器错误
  Server(JsonRpcError),
  /// 未初始化
  NotInitialized,
  /// 无效响应
  InvalidResponse(String),
  /// 方法未找到
  MethodNotFound(String),
}

impl std::fmt::Display for McpError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      McpError::Transport(msg) => write!(f, "Transport error: {}", msg),
      McpError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
      McpError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
      McpError::Io(msg) => write!(f, "IO error: {}", msg),
      McpError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
      McpError::Server(err) => write!(f, "Server error: {} - {}", err.code, err.message),
      McpError::NotInitialized => write!(f, "Client not initialized"),
      McpError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
      McpError::MethodNotFound(method) => write!(f, "Method not found: {}", method),
    }
  }
}

impl std::error::Error for McpError {}

impl From<serde_json::Error> for McpError {
  fn from(error: serde_json::Error) -> Self {
    McpError::Serialization(error.to_string())
  }
}

impl From<std::io::Error> for McpError {
  fn from(error: std::io::Error) -> Self {
    McpError::Io(error.to_string())
  }
}

impl From<JsonRpcError> for McpError {
  fn from(error: JsonRpcError) -> Self {
    McpError::Server(error)
  }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_json_rpc_request() {
    let request = JsonRpcRequest::new(RequestId::Number(1), "initialize")
      .with_params(serde_json::json!({"protocol_version": MCP_VERSION}));
    
    assert_eq!(request.jsonrpc, JSONRPC_VERSION);
    assert_eq!(request.id, RequestId::Number(1));
    assert_eq!(request.method, "initialize");
    assert!(request.params.is_some());
  }

  #[test]
  fn test_json_rpc_response_success() {
    let response = JsonRpcResponse::success(
      RequestId::Number(1),
      serde_json::json!({"status": "ok"}),
    );
    
    assert!(response.is_success());
    assert!(response.result.is_some());
    assert!(response.error.is_none());
  }

  #[test]
  fn test_json_rpc_response_error() {
    let response = JsonRpcResponse::error(
      RequestId::Number(1),
      JsonRpcError::method_not_found("unknown"),
    );
    
    assert!(!response.is_success());
    assert!(response.result.is_none());
    assert!(response.error.is_some());
  }

  #[test]
  fn test_initialize_params() {
    let params = InitializeParams::new(Implementation::new("test-client", "1.0.0"));
    
    assert_eq!(params.protocol_version, MCP_VERSION);
    assert_eq!(params.client_info.name, "test-client");
    assert_eq!(params.client_info.version, "1.0.0");
  }

  #[test]
  fn test_tool_input_schema() {
    let schema = ToolInputSchema::new()
      .with_properties(HashMap::from([
        ("query".to_string(), serde_json::json!({"type": "string"})),
      ]))
      .with_required(vec!["query".to_string()]);
    
    assert_eq!(schema.schema_type, "object");
    assert!(schema.properties.is_some());
    assert!(schema.required.is_some());
  }

  #[test]
  fn test_content_text() {
    let content = Content::text("Hello, world!");
    
    match content {
      Content::Text { text } => assert_eq!(text, "Hello, world!"),
      _ => panic!("Expected Text content"),
    }
  }

  #[test]
  fn test_mcp_error() {
    let error = McpError::Transport("Connection failed".to_string());
    assert!(error.to_string().contains("Transport error"));
  }
}