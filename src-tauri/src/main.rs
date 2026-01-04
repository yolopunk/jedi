// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::api::app::{
  disable_autostart, enable_autostart, ensure_jedi_dir, get_app_info, is_autostart_enabled,
};
use crate::api::hosts::{read_system_hosts, revert_hosts, update_hosts_with_groups};
use crate::api::os::{get_os_info, SystemState};
use crate::api::wallpapers::{get_wallpapers, set_desktop_wallpaper, sync_wallpapers};
use crate::utils::logger;
use std::sync::Mutex;
use sysinfo::{Networks, System};
use tauri::RunEvent::WindowEvent;
use tauri::{Manager, RunEvent};

mod api;
mod config;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let _logger_guard = logger::init();

  let app = tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_autostart::Builder::new().build())
    .plugin(tauri_plugin_store::Builder::default().build())
    .manage(SystemState {
      system: Mutex::new(System::new_all()),
      networks: Mutex::new(Networks::new_with_refreshed_list()),
    })
    .setup(|app| {
      config::app::load_tray_config(app);

      #[cfg(desktop)]
      {
        use tauri_plugin_autostart::MacosLauncher;
        use tauri_plugin_autostart::ManagerExt;

        let _ = app.handle().plugin(tauri_plugin_autostart::init(
          MacosLauncher::LaunchAgent,
          Some(vec!["--flag1", "--flag2"]),
        ));

        // Get the autostart manager
        let autostart_manager = app.autolaunch();
        // Enable autostart
        let _ = autostart_manager.enable();
        // Check enable state
        match autostart_manager.is_enabled() {
          Ok(enabled) => println!("registered for autostart? {}", enabled),
          Err(e) => eprintln!("failed to check autostart status: {}", e),
        }
        // Disable autostart
        let _ = autostart_manager.disable();
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      update_hosts_with_groups,
      revert_hosts,
      read_system_hosts,
      get_os_info,
      get_app_info,
      enable_autostart,
      disable_autostart,
      is_autostart_enabled,
      ensure_jedi_dir,
      get_wallpapers,
      set_desktop_wallpaper,
      sync_wallpapers
    ])
    .build(tauri::generate_context!())?;

  let _app_handle = app.handle();

  app.run(move |_app_handle, _event| match &_event {
    WindowEvent {
      event: tauri::WindowEvent::CloseRequested { api, .. },
      label,
      ..
    } => {
      api.prevent_close();
      if let Some(window) = _app_handle.get_webview_window(label) {
        if let Err(e) = window.hide() {
          eprintln!("failed to hide window: {}", e);
        }
      }
    }
    RunEvent::ExitRequested { api, code, .. } => {
      if code.is_none() {
        api.prevent_exit();
      }

      if let Err(e) = revert_hosts() {
        eprintln!("failed to revert hosts on exit: {}", e);
      }
    }
    _ => (),
  });

  Ok(())
}
