use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
  pub version: String,
  pub name: String,
}

#[tauri::command]
pub fn get_app_info() -> Result<AppInfo, String> {
  Ok(AppInfo {
    version: env!("CARGO_PKG_VERSION").to_string(),
    name: env!("CARGO_PKG_NAME").to_string(),
  })
}

/// 启用开机自启动
#[tauri::command]
pub fn enable_autostart(app: AppHandle) -> Result<(), String> {
  let autostart_manager = app.autolaunch();
  autostart_manager.enable().map_err(|e| e.to_string())
}

/// 禁用开机自启动
#[tauri::command]
pub fn disable_autostart(app: AppHandle) -> Result<(), String> {
  let autostart_manager = app.autolaunch();
  autostart_manager.disable().map_err(|e| e.to_string())
}

/// 检查是否已启用开机自启动
#[tauri::command]
pub fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
  let autostart_manager = app.autolaunch();
  autostart_manager.is_enabled().map_err(|e| e.to_string())
}
