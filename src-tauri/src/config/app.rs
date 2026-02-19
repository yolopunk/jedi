use crate::api::app::get_app_info;
use sysinfo::System;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, Manager};

pub fn load_tray_config(app: &App) {
  let show = match MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>) {
    Ok(item) => item,
    Err(e) => {
      eprintln!("failed to create show menu item: {}", e);
      return;
    }
  };

  let quit = match MenuItem::with_id(app, "quit", "退出", true, None::<&str>) {
    Ok(item) => item,
    Err(e) => {
      eprintln!("failed to create quit menu item: {}", e);
      return;
    }
  };

  let menu = match MenuBuilder::new(app).items(&[&show, &quit]).build() {
    Ok(menu) => menu,
    Err(e) => {
      eprintln!("failed to build menu: {}", e);
      return;
    }
  };

  // 获取应用信息
  let app_info = get_app_info().unwrap_or(crate::api::app::AppInfo {
    version: "0.0.0".to_string(),
    name: "Jedi".to_string(),
  });

  // 获取系统信息
  let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
  let os_version = System::os_version().unwrap_or_default();

  // 构建悬停提示文本
  let tooltip = format!(
    "{} v{} - 运行于 {} {}",
    app_info.name, app_info.version, os_name, os_version
  );

  let icon = match Image::from_bytes(include_bytes!("../../icons/icon.png")) {
    Ok(icon) => icon,
    Err(e) => {
      eprintln!("failed to load tray icon: {}", e);
      return;
    }
  };

  let _ = TrayIconBuilder::new()
    .menu(&menu)
    .icon(icon)
    .tooltip(&tooltip)
    .on_menu_event(move |app, event| match event.id.as_ref() {
      "show" => {
        if let Some(window) = app.get_webview_window("main") {
          let _ = window.show();
          let _ = window.set_focus();
        }
      }
      "quit" => {
        app.exit(-1);
      }

      _ => {}
    })
    .on_tray_icon_event(|tray, event| {
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        if let Some(window) = tray.app_handle().get_webview_window("main") {
          let _ = window.show();
          let _ = window.set_focus();
        }
      }
    })
    .icon_as_template(true)
    .build(app);
}
