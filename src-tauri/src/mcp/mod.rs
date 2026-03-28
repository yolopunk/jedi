// MCP 客户端模块
// Phase 3: MCP 客户端实现
//
// Model Context Protocol (MCP) 客户端实现
// 协议版本: 2024-11-05
//
// 参考文档: https://modelcontextprotocol.io/specification

//! MCP 客户端实现
//!
//! 提供与 MCP 服务器通信的完整功能：
//! - 协议类型定义（types）
//! - 传输层实现（transport）
//! - 协议处理（protocol）
//!
//! # 示例
//!
//! ```rust,ignore
//! use jedi::mcp::{McpClientBuilder, TransportConfig};
//!
//! // 创建客户端
//! let mut client = McpClientBuilder::new()
//!     .name("my-client")
//!     .version("1.0.0")
//!     .server_command("node")
//!     .server_args(vec!["mcp-server.js".to_string()])
//!     .build();
//!
//! // 启动并初始化
//! client.start()?;
//!
//! // 获取工具列表
//! let tools = client.list_tools()?;
//!
//! // 调用工具
//! let result = client.call_tool("my-tool", None)?;
//!
//! // 关闭客户端
//! client.stop()?;
//! ```

pub mod protocol;
pub mod servers;
pub mod transport;
pub mod types;

// 重新导出常用类型
#[allow(unused_imports)]
pub use protocol::{ClientState, McpClient, McpClientBuilder, McpClientConfig};
#[allow(unused_imports)]
pub use servers::HostsMcpServer;
#[allow(unused_imports)]
pub use transport::{StdioTransport, TransportConfig};
#[allow(unused_imports)]
pub use types::{
  // 通知相关
  notifications,
  // 工具相关
  CallToolParams,
  CallToolResult,
  CancelledParams,
  // 初始化相关
  ClientCapabilities,
  // 完成相关
  CompleteArgument,
  CompleteParams,
  CompleteResult,
  CompleteValues,
  // 基础类型
  Content,
  // 提示相关
  GetPromptParams,
  GetPromptResult,
  Implementation,
  InitializeParams,
  InitializeResult,
  JsonRpcError,
  JsonRpcNotification,
  JsonRpcRequest,
  JsonRpcResponse,
  ListPromptsParams,
  ListPromptsResult,
  // 资源相关
  ListResourcesParams,
  ListResourcesResult,
  ListToolsParams,
  ListToolsResult,
  // 日志相关
  LogLevel,
  LoggingMessageParams,
  McpError,
  ProgressParams,
  Prompt,
  PromptArgument,
  PromptMessage,
  PromptMessageRole,
  ReadResourceParams,
  ReadResourceResult,
  Reference,
  RequestId,
  Resource,
  ResourceContents,
  ResourceUpdatedParams,
  ServerCapabilities,
  SetLevelParams,
  Tool,
  ToolInputSchema,
  JSONRPC_VERSION,
  // 常量
  MCP_VERSION,
};

// ============================================================================
// 模块文档测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mcp_version() {
    assert_eq!(MCP_VERSION, "2024-11-05");
  }

  #[test]
  fn test_jsonrpc_version() {
    assert_eq!(JSONRPC_VERSION, "2.0");
  }

  #[test]
  fn test_module_exports() {
    // 测试主要类型是否正确导出
    let _ = MCP_VERSION;
    let _ = JSONRPC_VERSION;

    // 测试构建器
    let _builder = McpClientBuilder::new();

    // 测试类型创建
    let _request = JsonRpcRequest::new(RequestId::Number(1), "test");
    let _notification = JsonRpcNotification::new("test");
    let _error = JsonRpcError::new(-1, "test error");
  }

  #[test]
  fn test_transport_config_builder() {
    let config = TransportConfig::new("node")
      .with_args(vec!["server.js".to_string()])
      .with_startup_timeout(5000);

    assert_eq!(config.command, "node");
    assert_eq!(config.args, vec!["server.js"]);
    assert_eq!(config.startup_timeout_ms, 5000);
  }

  #[test]
  fn test_client_builder() {
    let client = McpClientBuilder::new()
      .name("test")
      .version("1.0.0")
      .build();

    assert_eq!(client.state(), ClientState::Uninitialized);
  }
}
