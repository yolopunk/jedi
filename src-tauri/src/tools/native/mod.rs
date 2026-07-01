// 内置工具（主力层）：直接复用现有 api::* 函数，不走 MCP

pub mod hosts;
pub mod podcast;
pub mod system;
pub mod wallpaper;

use super::ToolRegistry;
use std::sync::OnceLock;
use tauri::AppHandle;

/// 全局 AppHandle：部分内置工具（壁纸/播客/系统）需要它调用现有 Tauri command。
/// 在 setup 中通过 set_app_handle 注入。
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// 注入 AppHandle（在 Tauri setup 中调用一次）
pub fn set_app_handle(app: AppHandle) {
  let _ = APP_HANDLE.set(app);
}

/// 获取 AppHandle（工具执行时调用）
pub(crate) fn app_handle() -> Result<AppHandle, String> {
  APP_HANDLE
    .get()
    .cloned()
    .ok_or_else(|| "AppHandle 尚未初始化".to_string())
}

/// 注册全部内置工具
pub fn register_all(reg: &ToolRegistry) {
  for tool in hosts::tools() {
    let _ = reg.register(tool);
  }
  for tool in system::tools() {
    let _ = reg.register(tool);
  }
  for tool in wallpaper::tools() {
    let _ = reg.register(tool);
  }
  for tool in podcast::tools() {
    let _ = reg.register(tool);
  }
}
