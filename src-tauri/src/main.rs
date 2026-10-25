// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::agent::detector::{detect_agents, DetectorState};
use crate::agent::procman::{agent_proc_kill, agent_proc_spawn, agent_proc_stdin, ProcManState};
use crate::api::agent_fs::{file_exists, read_file, write_file};
use crate::api::ai_chat::{
  append_message,
  create_session,
  delete_api_key,
  delete_session,
  encode_html_entities,
  // Phase 3: Models.dev 集成
  fetch_models_dev,
  get_api_key,
  get_api_key_info,
  get_models_dev_provider,
  get_models_for_provider,
  get_models_providers,
  get_session,
  has_api_key,
  list_api_key_providers,
  list_sessions,
  log_security_event,
  query_security_logs,
  sanitize,
  search_models_dev,
  // Phase 2: 模型和会话管理
  send_chat_message,
  store_api_key,
  update_session_title,
  validate_input,
  validate_key,
  validate_message,
  validate_url,
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
use crate::api::memory::{memory_delete, memory_list, memory_recall, memory_save};
use crate::api::os::{get_os_info, SystemState};
use crate::api::podcast::{
  fetch_episodes, fetch_rss_channel, get_subscriptions, import_opml, refresh_subscription,
  remove_subscription, resolve_xiaoyuzhou_podcast, save_subscription,
};
use crate::api::screenshot::{
  list_screenshots, read_image_data_url, screenshot_begin, screenshot_cancel, screenshot_finish,
  screenshot_frame_info, screenshot_save_styled, ScreenshotState,
};
use crate::api::skills::{
  skill_stats_clear, skill_stats_list, skill_stats_save, skills_delete, skills_list, skills_read,
  skills_read_file, skills_save,
};
use crate::api::terminal::execute_command;
use crate::api::wallpapers::{
  get_current_wallpaper, get_wallpapers, set_desktop_wallpaper, show_in_folder, sync_wallpapers,
};
use crate::api::web::{web_fetch, web_search};
use crate::mcp::manager::{
  mcp_call_tool, mcp_connect, mcp_disconnect, mcp_list_connected, McpManager,
};
use crate::utils::logger;
use std::sync::Mutex;
use sysinfo::{Networks, System};
use tauri::RunEvent::WindowEvent;
use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder};

mod agent;
mod api;
mod config;
mod mcp;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Outbound MCP server mode: `jedi --mcp-server` runs a stdio MCP server that
  // exposes Jedi's read-only tools to external MCP clients, then exits. No GUI.
  if std::env::args().any(|a| a == "--mcp-server") {
    crate::mcp::server::run_stdio_server();
    return Ok(());
  }

  let _logger_guard = logger::init();

  // 初始化审计日志记录器状态
  let audit_logger_state = AuditLoggerState::new().expect("Failed to initialize audit logger");

  // 初始化 API Key SQLite 存储管理器状态
  let keyring_manager_state =
    KeyringManagerState::new().expect("Failed to initialize API key storage manager");

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
    // 必须最先注册：第二个实例在此退出并聚焦已有窗口。
    // WebView2 的用户数据目录被进程独占，多实例会让后启动者
    // 在创建 webview 时 panic（HRESULT 0x8007139F），窗口永远白屏。
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
      if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
      }
    }))
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_autostart::Builder::new().build())
    .plugin(tauri_plugin_store::Builder::default().build())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    // 全局快捷键触发截图；具体快捷键在 setup 中注册
    .plugin(
      tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
          use tauri_plugin_global_shortcut::ShortcutState;
          if event.state == ShortcutState::Pressed {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
              if let Err(e) = crate::api::screenshot::begin_capture(app.clone()).await {
                eprintln!("screenshot begin failed: {}", e);
              }
            });
          }
        })
        .build(),
    )
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
    .manage(DetectorState::new())
    .manage(ProcManState(std::sync::Arc::new(tokio::sync::Mutex::new(
      std::collections::HashMap::new(),
    ))))
    // Third-party MCP client connections
    .manage(McpManager::new())
    // Screenshot capture session
    .manage(ScreenshotState::default())
    .setup(|app| {
      // 创建主窗口
      let mut win_builder =
        WebviewWindowBuilder::new(app, "main", WebviewUrl::App(Default::default()))
          .title("")
          .inner_size(1200.0, 768.0)
          .min_inner_size(980.0, 600.0)
          .resizable(true);

      // Tauri v2: title_bar_style is configured in tauri.conf.json
      #[cfg(target_os = "macos")]
      {
        win_builder = win_builder
          .decorations(true)
          .title_bar_style(tauri::TitleBarStyle::Overlay);
      }
      #[cfg(target_os = "linux")]
      {
        win_builder = win_builder.decorations(true);
      }
      #[cfg(target_os = "windows")]
      {
        win_builder = win_builder.decorations(false);
      }

      let _window = win_builder.center().build()?;

      config::app::load_tray_config(app);

      // 注册截图全局快捷键（默认 Ctrl+Alt+S）
      #[cfg(desktop)]
      {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;
        let app_handle = app.handle().clone();
        match "ctrl+alt+s".parse::<tauri_plugin_global_shortcut::Shortcut>() {
          Ok(shortcut) => {
            if let Err(e) = app_handle.global_shortcut().register(shortcut) {
              eprintln!("failed to register screenshot shortcut: {}", e);
            }
          }
          Err(e) => eprintln!("failed to parse screenshot shortcut: {}", e),
        }
      }
      // 壁纸启动自愈：数据目录被外部清空后，注册表会指向已消失的壁纸文件，
      // explorer 重读时把桌面落成纯黑。按 备份→缓存重下→内置兜底 恢复。
      tauri::async_runtime::spawn(crate::api::wallpapers::ensure_wallpaper_health(
        app.handle().clone(),
      ));

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
      create_session,
      list_sessions,
      get_session,
      delete_session,
      update_session_title,
      append_message,
      get_api_key,
      // Phase 3: Models.dev commands
      fetch_models_dev,
      get_models_dev_provider,
      get_models_for_provider,
      get_models_providers,
      search_models_dev,
      // Agent hub: detection + external agent process management
      detect_agents,
      agent_proc_spawn,
      agent_proc_stdin,
      agent_proc_kill,
      // Web access tools (AI agent)
      web_fetch,
      web_search,
      // Shell execution tool (AI agent)
      execute_command,
      // Filesystem tool (AI agent)
      read_file,
      write_file,
      file_exists,
      // Cross-session memory tool (AI agent)
      memory_save,
      memory_recall,
      memory_list,
      memory_delete,
      // Third-party MCP client commands
      mcp_connect,
      mcp_disconnect,
      mcp_list_connected,
      mcp_call_tool,
      // Skill package management commands (SKILL.md directories)
      skills_list,
      skills_read,
      skills_read_file,
      skills_save,
      skills_delete,
      // Skill execution stats commands
      skill_stats_list,
      skill_stats_save,
      skill_stats_clear,
      // Screenshot capture commands
      screenshot_begin,
      screenshot_frame_info,
      screenshot_finish,
      screenshot_save_styled,
      screenshot_cancel,
      list_screenshots,
      read_image_data_url
    ])
    .build(tauri::generate_context!())?;

  let _app_handle = app.handle();

  app.run(move |_app_handle, _event| match &_event {
    WindowEvent {
      event: tauri::WindowEvent::CloseRequested { api, .. },
      label,
      ..
    } => {
      // release 隐藏到托盘；dev 直接退出——隐藏实例会锁住 WebView2 用户
      // 数据目录，让下一次 `tauri dev` 的新实例 panic 成白屏僵尸
      #[cfg(not(debug_assertions))]
      {
        api.prevent_close();
        if let Some(window) = _app_handle.get_webview_window(label) {
          if let Err(e) = window.hide() {
            eprintln!("failed to hide window: {}", e);
          }
        }
      }
      #[cfg(debug_assertions)]
      let _ = (&api, label);
    }
    RunEvent::ExitRequested { api, code, .. } => {
      #[cfg(debug_assertions)]
      let _ = (&api, &code);
      // release：无退出码的退出请求（关闭最后一个窗口）同样拦下，常驻托盘
      #[cfg(not(debug_assertions))]
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
