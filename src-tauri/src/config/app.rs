use crate::api::app::get_app_info;
use sysinfo::System;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, Emitter, Manager};

pub fn load_tray_config(app: &App) {
  let nav_chat = match MenuItem::with_id(app, "nav_chat", "💬 AI Chat", true, None::<&str>) {
    Ok(item) => item,
    Err(e) => {
      eprintln!("failed to create nav_chat menu item: {}", e);
      return;
    }
  };

  let nav_hosts = match MenuItem::with_id(app, "nav_hosts", "🌐 Hosts", true, None::<&str>) {
    Ok(item) => item,
    Err(e) => {
      eprintln!("failed to create nav_hosts menu item: {}", e);
      return;
    }
  };

  let nav_wallpapers =
    match MenuItem::with_id(app, "nav_wallpapers", "🖼️ Wallpapers", true, None::<&str>) {
      Ok(item) => item,
      Err(e) => {
        eprintln!("failed to create nav_wallpapers menu item: {}", e);
        return;
      }
    };

  let nav_podcast = match MenuItem::with_id(app, "nav_podcast", "🎧 Podcast", true, None::<&str>)
  {
    Ok(item) => item,
    Err(e) => {
      eprintln!("failed to create nav_podcast menu item: {}", e);
      return;
    }
  };

  let separator = match PredefinedMenuItem::separator(app) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("failed to create separator: {}", e);
      return;
    }
  };

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

  let menu = match MenuBuilder::new(app)
    .items(&[
      &nav_chat,
      &nav_hosts,
      &nav_wallpapers,
      &nav_podcast,
      &separator,
      &show,
      &quit,
    ])
    .build()
  {
    Ok(menu) => menu,
    Err(e) => {
      eprintln!("failed to build menu: {}", e);
      return;
    }
  };

  let app_info = get_app_info().unwrap_or(crate::api::app::AppInfo {
    version: "0.0.0".to_string(),
    name: "Jedi".to_string(),
  });

  let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
  let os_version = System::os_version().unwrap_or_default();

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
      "nav_chat" | "nav_hosts" | "nav_wallpapers" | "nav_podcast" => {
        let route = match event.id.as_ref() {
          "nav_chat" => "/chat",
          "nav_hosts" => "/hosts",
          "nav_wallpapers" => "/wallpapers",
          "nav_podcast" => "/podcast",
          _ => "",
        };
        if let Some(window) = app.get_webview_window("main") {
          let _ = window.show();
          let _ = window.set_focus();
          let _ = app.emit("tray-navigate", route);
        }
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
