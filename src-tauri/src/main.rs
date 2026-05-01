// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::api::ai_chat::{
  append_message,
  create_session,
  delete_api_key,
  delete_session,
  encode_html_entities,
  get_api_key_info,
  get_session,
  has_api_key,
  list_api_key_providers,
  list_sessions,
  log_security_event,
  get_api_key,
  query_security_logs,
  sanitize,
  // Phase 2: 模型和会话管理
  send_chat_message,
  send_chat_message_stream,
  store_api_key,
  validate_input,
  validate_key,
  validate_message,
  validate_url,
  // Phase 3: Models.dev 集成
  fetch_models_dev,
  get_models_dev_provider,
  get_models_for_provider,
  get_models_providers,
  search_models_dev,
  AuditLoggerState,
  ChatSessionManagerState,
  KeyringManagerState,
  ModelProviderManagerState,
  ModelsDevManagerState,
};
use crate::api::app::{
  disable_autostart, enable_autostart, ensure_jedi_dir, get_app_info, is_autostart_enabled,
};
use crate::api::hosts::{read_system_hosts, revert_hosts, update_hosts_with_groups};
use crate::api::os::{get_os_info, SystemState};
use crate::api::podcast::{
  fetch_episodes, fetch_rss_channel, get_subscriptions, import_opml, refresh_subscription,
  remove_subscription, resolve_xiaoyuzhou_podcast, save_subscription,
};
use crate::api::wallpapers::{
  get_current_wallpaper, get_wallpapers, set_desktop_wallpaper, show_in_folder, sync_wallpapers,
};
use crate::agent::commands::{init_pool, get_pool_status, schedule_task, stop_worker, remove_worker, PoolState};
use crate::agent::pool::AgentPool;
use crate::utils::logger;
use std::sync::Mutex;
use sysinfo::{Networks, System};
use tauri::RunEvent::WindowEvent;
use tauri::{Manager, RunEvent, TitleBarStyle, WebviewUrl, WebviewWindowBuilder};

mod agent;
mod api;
mod config;
mod mcp;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let _logger_guard = logger::init();

  // 初始化审计日志记录器状态
  let audit_logger_state = AuditLoggerState::new().expect("Failed to initialize audit logger");

  // 初始化密钥链管理器状态
  let keyring_manager_state =
    KeyringManagerState::new().expect("Failed to initialize keyring manager");

  // Phase 2: 初始化会话管理器状态
  use crate::api::ai_chat::sessions::ChatSessionManager;
  use crate::utils::security::AuditLogger;
  let audit_logger_for_sessions =
    AuditLogger::new().expect("Failed to initialize audit logger for sessions");
  let chat_session_manager = ChatSessionManager::new(audit_logger_for_sessions);
  let chat_session_manager_state = ChatSessionManagerState::new(chat_session_manager);

  // Phase 2: 初始化模型提供商管理器状态
  let model_provider_manager_state =
    ModelProviderManagerState::new(keyring_manager_state.get_manager().clone());

  // Phase 3: 初始化 models.dev 管理器状态
  let models_dev_manager_state = ModelsDevManagerState::new();

  let app = tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_autostart::Builder::new().build())
    .plugin(tauri_plugin_store::Builder::default().build())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(tauri_plugin_process::init())
    .manage(SystemState {
      system: Mutex::new(System::new_all()),
      networks: Mutex::new(Networks::new_with_refreshed_list()),
    })
    .manage(audit_logger_state)
    .manage(keyring_manager_state)
    // Phase 2: 管理新增状态
    .manage(chat_session_manager_state)
    .manage(model_provider_manager_state)
    // Phase 3: 管理 models.dev 状态
    .manage(models_dev_manager_state)
    // Agent pool state
    .manage(PoolState(tokio::sync::Mutex::new(AgentPool::new(4))))
    .setup(|app| {
      // 创建主窗口
      let mut win_builder =
        WebviewWindowBuilder::new(app, "main", WebviewUrl::App(Default::default()))
          .title("Jedi")
          .inner_size(1200.0, 768.0)
          .min_inner_size(980.0, 600.0)
          .resizable(true);

      #[cfg(target_os = "macos")]
      {
        win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);
      }

      let _window = win_builder.build()?;

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
      sync_wallpapers,
      get_current_wallpaper,
      show_in_folder,
      get_subscriptions,
      save_subscription,
      remove_subscription,
      refresh_subscription,
      fetch_rss_channel,
      fetch_episodes,
      import_opml,
      resolve_xiaoyuzhou_podcast,
      // AI Chat 安全审计日志 commands
      log_security_event,
      query_security_logs,
      // AI Chat API Key 管理 commands
      store_api_key,
      get_api_key_info,
      delete_api_key,
      has_api_key,
      list_api_key_providers,
      // AI Chat 输入验证 commands
      validate_input,
      validate_key,
      validate_url,
      validate_message,
      sanitize,
      encode_html_entities,
      // Phase 2: AI Chat 模型和会话管理 commands
      send_chat_message,
      send_chat_message_stream,
      create_session,
      list_sessions,
      get_session,
      delete_session,
      append_message,
      get_api_key,
      // Phase 3: Models.dev commands
      fetch_models_dev,
      get_models_dev_provider,
      get_models_for_provider,
      get_models_providers,
      search_models_dev,
      // Agent pool commands
      init_pool,
      get_pool_status,
      schedule_task,
      stop_worker,
      remove_worker
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
