// MCP 服务器模块
// Phase 3: MCP 服务器实现

//! MCP 服务器实现
//!
//! 提供内置的 MCP 服务器：
//! - Hosts MCP Server - Hosts 文件管理工具

pub mod hosts;

// 重新导出
pub use hosts::HostsMcpServer;
