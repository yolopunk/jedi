use crate::api::app::get_app_info;
use crate::api::os::get_os_info;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, Manager};

pub fn load_tray_config(app: &App) {
  let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>).unwrap();
  let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap();

  let menu = MenuBuilder::new(app)
    .items(&[&show, &quit])
    .build()
    .unwrap();

  // 获取应用信息
  let app_info = get_app_info().unwrap();
  let os_info = get_os_info();

  // 构建悬停提示文本
  let tooltip = format!(
    "{} v{} - 运行于 {} {}",
    app_info.name,
    app_info.version,
    os_info.name,
    os_info.os_version.unwrap_or_default()
  );

  let _ = TrayIconBuilder::new()
    .menu(&menu)
    .icon(Image::from_bytes(include_bytes!("../../icons/icon.png")).unwrap())
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
    .on_tray_icon_event(|tray, event| match event {
      TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } => {
        if let Some(window) = tray.app_handle().get_webview_window("main") {
          let _ = window.show();
          let _ = window.set_focus();
        }
      }
      _ => {}
    })
    .icon_as_template(true)
    .build(app);
}
