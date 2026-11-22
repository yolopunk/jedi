use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
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

fn is_scoop_install_path(executable: &Path) -> bool {
  executable.parent().is_some_and(|install_dir| {
    if install_dir.join(".scoop-install").is_file() {
      return true;
    }

    fs::read_to_string(install_dir.join("scoop-install.json"))
      .ok()
      .and_then(|metadata| serde_json::from_str::<serde_json::Value>(&metadata).ok())
      .and_then(|metadata| metadata.get("bucket")?.as_str().map(str::to_owned))
      .is_some_and(|bucket| bucket.eq_ignore_ascii_case("jedi"))
  })
}

#[tauri::command]
pub fn is_scoop_install() -> bool {
  std::env::current_exe()
    .map(|executable| is_scoop_install_path(&executable))
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
  use super::is_scoop_install_path;
  use std::{fs, path::Path};

  #[test]
  fn detects_scoop_install_marker_next_to_executable() {
    let install_dir = tempfile::tempdir().unwrap();
    fs::write(install_dir.path().join(".scoop-install"), "").unwrap();

    assert!(is_scoop_install_path(&install_dir.path().join("Jedi.exe")));
  }

  #[test]
  fn detects_existing_scoop_metadata_without_marker() {
    let install_dir = tempfile::tempdir().unwrap();
    fs::write(
      install_dir.path().join("scoop-install.json"),
      r#"{"bucket":"jedi","architecture":"64bit"}"#,
    )
    .unwrap();

    assert!(is_scoop_install_path(&install_dir.path().join("Jedi.exe")));
  }

  #[test]
  fn does_not_detect_metadata_for_another_bucket() {
    let install_dir = tempfile::tempdir().unwrap();
    fs::write(
      install_dir.path().join("scoop-install.json"),
      r#"{"bucket":"other","architecture":"64bit"}"#,
    )
    .unwrap();

    assert!(!is_scoop_install_path(&install_dir.path().join("Jedi.exe")));
  }

  #[test]
  fn does_not_detect_normal_install_without_marker() {
    assert!(!is_scoop_install_path(Path::new(
      "C:/Users/user/AppData/Local/Jedi/Jedi.exe"
    )));
  }

  #[test]
  fn does_not_detect_other_scoop_apps() {
    assert!(!is_scoop_install_path(Path::new(
      "C:/Users/user/scoop/apps/other/current/Other.exe"
    )));
  }

  #[test]
  fn does_not_detect_unmarked_executable_in_scoop_named_path() {
    assert!(!is_scoop_install_path(Path::new(
      "C:/Users/user/scoop/apps/jedi/current/Jedi.exe"
    )));
  }
}

/// 按应用主题切换窗口/任务栏与托盘图标（dark=true 使用深底白标，浅色主题用白底黑标）
#[tauri::command]
pub fn set_icon_theme(app: AppHandle, dark: bool) {
  crate::config::app::apply_icon_variant(&app, dark);
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
