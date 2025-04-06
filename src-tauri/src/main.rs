// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::api::dns::{revert_hosts, update_hosts_with_tag};
use crate::api::os::get_os_info;
use crate::utils::logger;
use tauri::RunEvent::WindowEvent;
use tauri::{Manager, RunEvent};

mod api;
mod config;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let _logger_guard = logger::init();

  let app = tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .setup(|app| {
      config::app::load_tray_config(app);
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      update_hosts_with_tag,
      revert_hosts,
      get_os_info
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
      _app_handle
        .get_webview_window(label)
        .unwrap()
        .hide()
        .unwrap();
    }
    RunEvent::ExitRequested { api, code, .. } => {
      if code.is_none() {
        api.prevent_exit();
      }

      revert_hosts().unwrap();
    }
    _ => (),
  });

  Ok(())
}
