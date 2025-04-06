use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Manager};

pub fn load_tray_config(app: &App) {
  let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>).unwrap();
  let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap();

  let menu = MenuBuilder::new(app)
    .items(&[&show, &quit])
    .build()
    .unwrap();

  let _ = TrayIconBuilder::new()
    .menu(&menu)
    .icon(Image::from_bytes(include_bytes!("../../icons/icon.png")).unwrap())
    .show_menu_on_left_click(true)
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
    .icon_as_template(true)
    .build(app);
}
