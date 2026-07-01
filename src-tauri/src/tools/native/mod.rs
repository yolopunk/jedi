// 内置工具（主力层）：直接复用现有 api::* 函数，不走 MCP

pub mod hosts;

use super::ToolRegistry;

/// 注册全部内置工具
pub fn register_all(reg: &ToolRegistry) {
  for tool in hosts::tools() {
    // 内置工具在编译期保证不重名，这里忽略结果即可
    let _ = reg.register(tool);
  }
  // P2：wallpaper / podcast / system
}
