use serde::{Deserialize, Serialize};
use std::fs;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use dirs;

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

/// 确保 .jedi 目录存在
#[tauri::command]
pub fn ensure_jedi_dir() -> Result<String, String> {
  let home_dir = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
  let jedi_dir = home_dir.join(".jedi");

  // 创建目录（如果不存在）
  if !jedi_dir.exists() {
    fs::create_dir_all(&jedi_dir).map_err(|e| format!("创建 .jedi 目录失败: {}", e))?;
  }

  // 返回目录路径
  Ok(jedi_dir.to_string_lossy().to_string())
}
