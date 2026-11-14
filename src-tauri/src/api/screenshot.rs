// 截图裁剪：冻结全屏 → 覆盖窗口框选 → 裁剪输出（剪贴板/文件/AI 聊天）
//
// 流程：begin 时用 xcap 逐显示器捕获并落盘临时 PNG，为每个显示器创建一个
// 无边框置顶覆盖窗口（路由 /screenshot-overlay，显示器编号编码在窗口 label 中）。
// 覆盖窗口回传物理像素选区后由 finish 统一裁剪、输出并拆除全部覆盖窗口。

use crate::api::wallpapers;
use image::RgbaImage;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::webview::WebviewWindowBuilder;
use tauri::{
  AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, State, WebviewUrl,
};

pub const OVERLAY_LABEL_PREFIX: &str = "screenshot-overlay-";
pub const TOOLBAR_LABEL: &str = "screenshot-toolbar";
pub const SCREENSHOTS_DIR_NAME: &str = "screenshots";
pub const FRAME_CACHE_DIR_NAME: &str = "screenshot-frames";
/// 滚动截长图模式注册的全局快捷键（Space=捕获 / Enter=完成 / Esc=取消）
pub(crate) const SCROLL_HOTKEYS: [&str; 3] = ["Space", "Enter", "Escape"];
/// 默认截图快捷键
pub const DEFAULT_CAPTURE_HOTKEY: &str = "F1";

/// 一轮截图会话：所有显示器的冻结帧与对应覆盖窗口。
pub struct CaptureSession {
  monitors: Vec<MonitorFrame>,
}

pub struct MonitorFrame {
  /// 显示器序号，与覆盖窗口 label 后缀一致
  index: usize,
  /// 覆盖窗口 label：`screenshot-overlay-<index>`
  label: String,
  /// 冻结帧 PNG 临时文件路径
  frame_path: PathBuf,
  /// 显示器物理坐标与尺寸（xcap 输出）
  x: i32,
  y: i32,
  width: u32,
  height: u32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MonitorFrameInfo {
  pub index: usize,
  pub frame_path: String,
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotOutcome {
  /// 命中的输出动作：copy | save | chat | scroll
  pub action: String,
  /// 已留存到截图画廊的文件路径
  pub saved_path: Option<String>,
  /// 需要向用户提示的警告码（如拼接降级），由前端映射为 i18n 文案
  pub warnings: Vec<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotItem {
  pub name: String,
  pub path: String,
  pub size: u64,
  /// 修改时间（Unix 毫秒）
  pub modified_ms: i64,
  /// 条目类型：image | video
  pub kind: String,
}

/// 一轮滚动截长图会话：固定选区 + 逐屏捕获的帧序列。
pub struct ScrollSession {
  monitor_index: usize,
  /// 选区在显示器帧内的物理像素坐标
  x: u32,
  y: u32,
  width: u32,
  height: u32,
  frames: Vec<RgbaImage>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScrollStatus {
  pub frame_count: u32,
}

/// 全局会话状态；同一时刻只允许一轮截图。
pub struct ScreenshotState {
  session: Mutex<Option<CaptureSession>>,
  pub(crate) scroll: Mutex<Option<ScrollSession>>,
  /// 当前生效的截图全局快捷键（accelerator 字符串）
  pub(crate) hotkey: Mutex<String>,
}

impl Default for ScreenshotState {
  fn default() -> Self {
    Self {
      session: Mutex::new(None),
      scroll: Mutex::new(None),
      // 空串 = 尚未注册任何快捷键；启动时会注册默认值。
      // 这里若填默认值，register_capture_hotkey 的「同值跳过」会让启动注册被跳过。
      hotkey: Mutex::new(String::new()),
    }
  }
}

pub fn screenshots_dir(app: &AppHandle) -> Result<PathBuf, String> {
  let dir = app
    .path()
    .picture_dir()
    .map_err(|e| e.to_string())?
    .join("jedi");
  fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
  Ok(dir)
}

fn frame_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
  let dir = app
    .path()
    .app_cache_dir()
    .map_err(|e| e.to_string())?
    .join(FRAME_CACHE_DIR_NAME);
  fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
  Ok(dir)
}

/// 把选区收敛到帧范围内：origin 非负、不越过右/下边界、至少 1×1。
pub fn clamp_crop_rect(
  frame_w: u32,
  frame_h: u32,
  x: i32,
  y: i32,
  w: u32,
  h: u32,
) -> (u32, u32, u32, u32) {
  let x = x.clamp(0, frame_w.saturating_sub(1) as i32) as u32;
  let y = y.clamp(0, frame_h.saturating_sub(1) as i32) as u32;
  let w = w.min(frame_w - x).max(1);
  let h = h.min(frame_h - y).max(1);
  (x, y, w, h)
}

pub(crate) fn timestamped_filename(ext: &str) -> String {
  let now = chrono::Local::now();
  format!("Jedi_{}.{}", now.format("%Y%m%d_%H%M%S%3f"), ext)
}

/// 拆除会话：关闭所有覆盖窗口并删除冻结帧临时文件。
/// 用 destroy 而非 close：close 会走 CloseRequested（主窗口的托盘逻辑），
/// 辅助窗一旦被拦截成 hide，同名窗口残留会让下一轮截图的建窗失败。
fn teardown(app: &AppHandle, session: CaptureSession) {
  for monitor in &session.monitors {
    if let Some(window) = app.get_webview_window(&monitor.label) {
      let _ = window.destroy();
    }
    let _ = fs::remove_file(&monitor.frame_path);
  }
}

/// 捕获全部显示器并弹出覆盖窗口。命令、托盘与全局快捷键共用此入口。
pub async fn begin_capture(app: AppHandle) -> Result<Vec<MonitorFrameInfo>, String> {
  let state = app.state::<ScreenshotState>();

  // 重复触发时先拆掉上一轮（含滚动/录屏残留），保证幂等且不会两套 UI 打架
  teardown_all(&app);

  let cache_dir = frame_cache_dir(&app)?;

  // 阻塞捕获 + PNG 编码挪出主线程
  let frames =
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<MonitorFrame>, String> {
      let monitors = xcap::Monitor::all().map_err(|e| format!("枚举显示器失败: {}", e))?;
      if monitors.is_empty() {
        return Err("未发现任何显示器".to_string());
      }
      let mut frames = Vec::with_capacity(monitors.len());
      for (index, monitor) in monitors.into_iter().enumerate() {
        // macOS 未授予屏幕录制权限时 capture_image 会失败或返回空帧
        let image = monitor
          .capture_image()
          .map_err(|e| format!("捕获显示器 {} 失败: {}", index, e))?;
        let frame_path = cache_dir.join(format!("frame_{}.png", index));
        image.save(&frame_path).map_err(|e| e.to_string())?;
        frames.push(MonitorFrame {
          index,
          label: format!("{}{}", OVERLAY_LABEL_PREFIX, index),
          frame_path,
          x: monitor.x().map_err(|e| e.to_string())?,
          y: monitor.y().map_err(|e| e.to_string())?,
          width: monitor.width().map_err(|e| e.to_string())?,
          height: monitor.height().map_err(|e| e.to_string())?,
        });
      }
      Ok(frames)
    })
    .await
    .map_err(|e| format!("捕获任务异常: {}", e))??;

  let infos: Vec<MonitorFrameInfo> = frames
    .iter()
    .map(|f| MonitorFrameInfo {
      index: f.index,
      frame_path: f.frame_path.to_string_lossy().into_owned(),
      x: f.x,
      y: f.y,
      width: f.width,
      height: f.height,
    })
    .collect();

  let primary_label = frames
    .iter()
    .find(|_| true)
    .map(|f| f.label.clone())
    .unwrap_or_default();

  *state.session.lock().unwrap() = Some(CaptureSession { monitors: frames });

  // macOS 要求在主线程创建窗口；其余平台同样安全
  let app_for_windows = app.clone();
  let labels = infos.clone();
  app
    .run_on_main_thread(move || {
      for info in &labels {
        let label = format!("{}{}", OVERLAY_LABEL_PREFIX, info.index);
        // 上一轮若残留同名窗口，先销毁再建，避免 label 冲突导致建窗失败
        if let Some(stale) = app_for_windows.get_webview_window(&label) {
          let _ = stale.destroy();
        }
        let builder = WebviewWindowBuilder::new(
          &app_for_windows,
          &label,
          WebviewUrl::App("screenshot-overlay".into()),
        )
        .title("Jedi Screenshot")
        .visible(false)
        .decorations(false)
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .shadow(false)
        // 录制模式下覆盖窗口切换为透明+点击穿透，需要建窗时开启透明
        .transparent(true)
        // 只有主显示器的窗口接管键盘焦点，其余显示器仅展示冻结帧
        .focused(label == primary_label);
        match builder.build() {
          Ok(window) => {
            let _ = window.set_position(Position::Physical(PhysicalPosition::new(info.x, info.y)));
            let _ =
              window.set_size(Size::Physical(PhysicalSize::new(info.width, info.height)));
          }
          Err(e) => eprintln!("failed to create overlay window {}: {}", label, e),
        }
      }
    })
    .map_err(|e| format!("创建覆盖窗口失败: {}", e))?;

  Ok(infos)
}

/// 完成截图：按物理像素选区裁剪冻结帧并执行输出动作，随后拆除会话。
async fn finish_capture(
  app: AppHandle,
  monitor_index: usize,
  x: i32,
  y: i32,
  width: u32,
  height: u32,
  action: String,
) -> Result<ScreenshotOutcome, String> {
  let state = app.state::<ScreenshotState>();
  let session = state
    .session
    .lock()
    .unwrap()
    .take()
    .ok_or_else(|| "没有正在进行的截图会话".to_string())?;

  let result = (|| async {
    let frame = session
      .monitors
      .iter()
      .find(|m| m.index == monitor_index)
      .ok_or_else(|| format!("选区显示器 {} 不在本轮捕获中", monitor_index))?;

    let frame_path = frame.frame_path.clone();
    let (cx, cy, cw, ch) = clamp_crop_rect(frame.width, frame.height, x, y, width, height);

    // 裁剪 + PNG 编码在阻塞线程完成，产出剪贴板所需的 RGBA 原始字节与 PNG 字节
    let (rgba_raw, png_bytes, out_w, out_h) = tauri::async_runtime::spawn_blocking(
      move || -> Result<(Vec<u8>, Vec<u8>, u32, u32), String> {
        let img = image::open(&frame_path).map_err(|e| format!("读取冻结帧失败: {}", e))?;
        let cropped = img.crop_imm(cx, cy, cw, ch).to_rgba8();
        let (w, h) = (cropped.width(), cropped.height());
        let mut png = Vec::new();
        cropped
          .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
          .map_err(|e| format!("PNG 编码失败: {}", e))?;
        let raw = cropped.into_raw();
        Ok((raw, png, w, h))
      },
    )
    .await
    .map_err(|e| format!("裁剪任务异常: {}", e))??;

    if !matches!(action.as_str(), "copy" | "save" | "chat" | "ocr") {
      return Err(format!("未知输出动作: {}", action));
    }
    let path = screenshots_dir(&app)?.join(timestamped_filename("png"));
    fs::write(&path, &png_bytes).map_err(|e| format!("保存截图失败: {}", e))?;
    let path_str = path.to_string_lossy().into_owned();
    let _ = app.emit_to("main", "screenshot-saved", &path_str);

    match action.as_str() {
      "copy" => {
        use tauri_plugin_clipboard_manager::ClipboardExt;
        let image = tauri::image::Image::new(&rgba_raw, out_w, out_h);
        app
          .clipboard()
          .write_image(&image)
          .map_err(|e| format!("写入剪贴板失败: {}", e))?;
        Ok(ScreenshotOutcome {
          action: "copy".into(),
          saved_path: Some(path_str),
          warnings: Vec::new(),
        })
      }
      "save" => {
        wallpapers::show_in_folder(path_str.clone()).await?;
        Ok(ScreenshotOutcome {
          action: "save".into(),
          saved_path: Some(path_str),
          warnings: Vec::new(),
        })
      }
      "chat" => {
        app
          .emit_to("main", "screenshot-captured", &path_str)
          .map_err(|e| format!("通知主窗口失败: {}", e))?;
        Ok(ScreenshotOutcome {
          action: "chat".into(),
          saved_path: Some(path_str),
          warnings: Vec::new(),
        })
      }
      "ocr" => {
        app
          .emit_to("main", "screenshot-ocr", &path_str)
          .map_err(|e| format!("通知主窗口失败: {}", e))?;
        Ok(ScreenshotOutcome {
          action: "ocr".into(),
          saved_path: Some(path_str),
          warnings: Vec::new(),
        })
      }
      other => Err(format!("未知输出动作: {}", other)),
    }
  })()
  .await;

  teardown(&app, session);
  result
}

/// 取消截图：直接拆除会话，不产出任何文件。
fn cancel_capture(app: AppHandle) -> Result<(), String> {
  let state = app.state::<ScreenshotState>();
  let taken = state.session.lock().unwrap().take();
  if let Some(session) = taken {
    teardown(&app, session);
  }
  Ok(())
}

// ---- 滚动截长图 / 录屏共用基础设施 ----

/// 读取会话内指定显示器的物理几何（monitor_x, monitor_y, w, h）。
pub(crate) fn session_monitor_geometry(
  state: &ScreenshotState,
  monitor_index: usize,
) -> Result<(i32, i32, u32, u32), String> {
  let guard = state.session.lock().unwrap();
  let monitor = session_monitor(&guard, monitor_index)?;
  Ok((monitor.x, monitor.y, monitor.width, monitor.height))
}

/// 读取会话内指定显示器的冻结帧临时文件路径。
fn session_frame_path(state: &ScreenshotState, monitor_index: usize) -> Result<PathBuf, String> {
  let guard = state.session.lock().unwrap();
  Ok(session_monitor(&guard, monitor_index)?.frame_path.clone())
}

fn session_monitor(
  session: &Option<CaptureSession>,
  monitor_index: usize,
) -> Result<&MonitorFrame, String> {
  session
    .as_ref()
    .ok_or_else(|| "没有正在进行的截图会话".to_string())?
    .monitors
    .iter()
    .find(|m| m.index == monitor_index)
    .ok_or_else(|| format!("显示器 {} 不在本轮捕获中", monitor_index))
}

/// 实时捕获指定显示器上的固定区域（帧内物理像素坐标），供滚动帧与录屏共用。
pub(crate) fn capture_live_region(
  monitor_index: usize,
  x: u32,
  y: u32,
  width: u32,
  height: u32,
) -> Result<RgbaImage, String> {
  let monitors = xcap::Monitor::all().map_err(|e| format!("枚举显示器失败: {}", e))?;
  let monitor = monitors
    .into_iter()
    .nth(monitor_index)
    .ok_or_else(|| format!("显示器 {} 不存在", monitor_index))?;
  let frame = monitor
    .capture_image()
    .map_err(|e| format!("捕获显示器失败: {}", e))?;
  let (cx, cy, cw, ch) = clamp_crop_rect(
    frame.width(),
    frame.height(),
    x as i32,
    y as i32,
    width,
    height,
  );
  Ok(image::imageops::crop_imm(&frame, cx, cy, cw, ch).to_image())
}

/// Windows：把窗口从系统捕获中排除（WDA_EXCLUDEFROMCAPTURE），
/// 录屏/截长图时覆盖窗口与悬浮条不会入镜。其余平台无对应 API，保持可见。
#[cfg(target_os = "windows")]
pub(crate) fn exclude_from_capture(window: &tauri::WebviewWindow<tauri::Wry>) {
  use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE,
  };
  if let Ok(hwnd) = window.hwnd() {
    unsafe {
      SetWindowDisplayAffinity(hwnd.0 as _, WDA_EXCLUDEFROMCAPTURE);
    }
  }
}
#[cfg(not(target_os = "windows"))]
pub(crate) fn exclude_from_capture(_window: &tauri::WebviewWindow<tauri::Wry>) {}

/// 关闭悬浮控制条窗口（若存在）。
pub(crate) fn close_toolbar(app: &AppHandle) {
  if let Some(toolbar) = app.get_webview_window(TOOLBAR_LABEL) {
    let _ = toolbar.destroy();
  }
}

/// 在选区下方（放不下则上方）弹出悬浮控制条窗口。
pub(crate) fn show_toolbar(
  app: &AppHandle,
  monitor: (i32, i32, u32, u32),
  region: (u32, u32, u32, u32),
) {
  let (mx, my, mw, mh) = monitor;
  let (_rx, ry, rw, rh) = region;
  const TOOLBAR_H: i32 = 64;
  const MARGIN: i32 = 12;
  let region_bottom = my + ry as i32 + rh as i32;
  let below_y = region_bottom + MARGIN;
  let py = if below_y + TOOLBAR_H <= my + mh as i32 {
    below_y
  } else {
    (my + ry as i32 - MARGIN - TOOLBAR_H).max(my)
  };
  let px = (mx + (rw as i32 / 2) - 240).clamp(mx, mx + mw as i32 - 480);

  let app_for_window = app.clone();
  let _ = app.run_on_main_thread(move || {
    // 上一轮残留的悬浮条：复用窗口，通知其按新会话重新初始化并复位
    if let Some(existing) = app_for_window.get_webview_window(TOOLBAR_LABEL) {
      let _ = existing.set_position(Position::Physical(PhysicalPosition::new(px.max(mx), py)));
      let _ = app_for_window.emit_to(TOOLBAR_LABEL, "toolbar-reactivate", ());
      return;
    }
    let builder = WebviewWindowBuilder::new(
      &app_for_window,
      TOOLBAR_LABEL,
      WebviewUrl::App("screenshot-toolbar".into()),
    )
    .title("Jedi Capture")
    .visible(true)
    .decorations(false)
    .resizable(false)
    .minimizable(false)
    .maximizable(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .shadow(false)
    .transparent(true)
    .focused(true)
    .inner_size(480.0, 64.0);
    if let Ok(window) = builder.build() {
      let _ = window.set_position(Position::Physical(PhysicalPosition::new(px.max(mx), py)));
      exclude_from_capture(&window);
    }
  });
}

/// 注册滚动模式全局快捷键：Space=捕获 / Enter=完成 / Escape=取消。
/// 仅在滚动会话期间存在，事件转发给悬浮条窗口。
fn register_scroll_hotkeys(app: &AppHandle) {
  #[cfg(desktop)]
  {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    for (code, action) in [
      ("Space", "capture"),
      ("Enter", "finish"),
      ("Escape", "cancel"),
    ] {
      match code.parse::<tauri_plugin_global_shortcut::Shortcut>() {
        Ok(shortcut) => {
          let app_handle = app.clone();
          if let Err(e) = app
            .global_shortcut()
            .on_shortcut(shortcut, move |_app, _s, _ev| {
              let _ = app_handle.emit_to(TOOLBAR_LABEL, "scroll-hotkey", action);
            })
          {
            eprintln!("failed to register scroll hotkey {}: {}", code, e);
          }
        }
        Err(e) => eprintln!("failed to parse scroll hotkey {}: {}", code, e),
      }
    }
  }
}

fn unregister_scroll_hotkeys(app: &AppHandle) {
  #[cfg(desktop)]
  {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    for code in SCROLL_HOTKEYS {
      if let Ok(shortcut) = code.parse::<tauri_plugin_global_shortcut::Shortcut>() {
        let _ = app.global_shortcut().unregister(shortcut);
      }
    }
  }
}

// ---- 截图全局快捷键（可配置，默认 F1）----

/// 注册（或换绑）截图全局快捷键，旧的会先注销。
pub(crate) fn register_capture_hotkey(app: &AppHandle, accelerator: &str) -> Result<(), String> {
  #[cfg(desktop)]
  {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let accelerator = accelerator.trim();
    if accelerator.is_empty() {
      return Err("快捷键不能为空".to_string());
    }
    let shortcut = accelerator
      .parse::<tauri_plugin_global_shortcut::Shortcut>()
      .map_err(|e| format!("快捷键格式无效：{}", e))?;

    let state = app.state::<ScreenshotState>();
    let previous = state.hotkey.lock().unwrap().clone();
    if previous == accelerator {
      return Ok(());
    }
    // 先注册新键，成功后再注销旧键：新键被占用时旧键仍可用，不会两头落空
    app
      .global_shortcut()
      .register(shortcut)
      .map_err(|e| format!("注册快捷键失败：{}", e))?;
    if let Ok(old) = previous.parse::<tauri_plugin_global_shortcut::Shortcut>() {
      let _ = app.global_shortcut().unregister(old);
    }
    *state.hotkey.lock().unwrap() = accelerator.to_string();
    Ok(())
  }
  #[cfg(not(desktop))]
  {
    let _ = (app, accelerator);
    Err("当前平台不支持全局快捷键".to_string())
  }
}

/// 当前生效的截图快捷键（accelerator 字符串）
pub(crate) fn current_capture_hotkey(app: &AppHandle) -> String {
  app.state::<ScreenshotState>().hotkey.lock().unwrap().clone()
}

/// 判断某个全局快捷键事件是否属于当前截图快捷键。
/// 插件对每个快捷键都会同时调用专用 handler 与全局 handler，
/// 必须在这里过滤，否则滚动模式下的 Space/Enter/Esc 会顺带重启一次截图。
pub(crate) fn is_capture_hotkey(
  app: &AppHandle,
  shortcut: &tauri_plugin_global_shortcut::Shortcut,
) -> bool {
  current_capture_hotkey(app)
    .parse::<tauri_plugin_global_shortcut::Shortcut>()
    .map(|current| current == *shortcut)
    .unwrap_or(false)
}

#[tauri::command]
pub fn set_capture_hotkey(app: AppHandle, accelerator: String) -> Result<(), String> {
  register_capture_hotkey(&app, &accelerator)
}

#[tauri::command]
pub fn get_capture_hotkey(app: AppHandle) -> String {
  current_capture_hotkey(&app)
}

/// 滚动/录制模式：保留选区所在覆盖窗（点击穿透 + 排除捕获），隐藏其余显示器覆盖窗。
/// 用户能看到真实画面、能用鼠标滚动/操作底下的应用，覆盖窗只画一个选区边框。
pub(crate) fn keep_selection_overlay(app: &AppHandle, monitor_index: usize) {
  let keep_label = format!("{}{}", OVERLAY_LABEL_PREFIX, monitor_index);
  for window in app.webview_windows().values() {
    if !window.label().starts_with(OVERLAY_LABEL_PREFIX) {
      continue;
    }
    if window.label() == keep_label {
      let _ = window.set_ignore_cursor_events(true);
      exclude_from_capture(window);
      let _ = window.show();
    } else {
      let _ = window.hide();
    }
  }
}

/// 全量拆除：覆盖窗口 + 悬浮条 + 滚动会话 + 快捷键 + 临时帧。
/// 滚动完成/取消、录屏结束与应用退出共用。
pub fn teardown_all(app: &AppHandle) {
  unregister_scroll_hotkeys(app);
  close_toolbar(app);
  let state = app.state::<ScreenshotState>();
  let scroll = state.scroll.lock().unwrap().take();
  drop(scroll);
  let session = state.session.lock().unwrap().take();
  if let Some(session) = session {
    teardown(app, session);
  }
}

// ---- 滚动帧拼接 ----

/// FNV-1a 64：对单行 RGBA 字节做快速指纹。
fn row_hash(row: &[u8]) -> u64 {
  let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
  for &byte in row {
    hash ^= byte as u64;
    hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
  }
  hash
}

fn row_hashes(img: &RgbaImage) -> Vec<u64> {
  let stride = (img.width() * 4) as usize;
  let raw = img.as_raw();
  raw.chunks_exact(stride).map(row_hash).collect()
}

/// 容差行比较：采样至多 64 个像素，允许每通道 ±2 抖动（抗锯齿噪声），
/// 96% 一致算相同。容差刻意取小：渐变背景上行距级色差必须判为不同，
/// 否则会产生假重叠锚点。
fn rows_fuzzy_equal(a: &[u8], b: &[u8], tolerance: u8) -> bool {
  let len = a.len().min(b.len());
  if len == 0 {
    return false;
  }
  let px = len / 4;
  let step = (px / 64).max(1);
  let mut matched = 0usize;
  let mut total = 0usize;
  let mut i = 0;
  while i < px {
    let ao = i * 4;
    let bo = i * 4;
    let close = (a[ao] as i16 - b[bo] as i16).abs() <= tolerance as i16
      && (a[ao + 1] as i16 - b[bo + 1] as i16).abs() <= tolerance as i16
      && (a[ao + 2] as i16 - b[bo + 2] as i16).abs() <= tolerance as i16;
    total += 1;
    if close {
      matched += 1;
    }
    i += step;
  }
  total > 0 && matched * 100 >= total * 96
}

/// 在累积图 a 的尾部与新一屏 b 的头部之间寻找滚动重叠行数。
/// 先做行哈希精确匹配，失败后对候选做像素级容差验证；找不到返回 None。
pub(crate) fn best_overlap(a: &RgbaImage, b: &RgbaImage) -> Option<u32> {
  if a.width() != b.width() || a.width() == 0 || a.height() == 0 || b.height() == 0 {
    return None;
  }
  let ha = row_hashes(a);
  let hb = row_hashes(b);
  let h_a = a.height() as usize;
  let h_b = b.height() as usize;
  let max_k = h_a.min(h_b);
  let min_k = 8.min(max_k);

  // 精确：A 的尾部 k 行与 B 的头部 k 行哈希全等
  for k in (min_k..=max_k).rev() {
    let start = h_a - k;
    if (0..k).all(|j| ha[start + j] == hb[j]) {
      return Some(k as u32);
    }
  }

  // 模糊：候选锚点 = B 首行与 A 某行像素级相近，随后验证整段。
  // 自底向上扫描约两屏范围、最多验证 8 个锚点；在通过验证的锚点里
  // 取最大重叠（真重叠总是最大），避免渐变内容上的小 k 假锚点。
  let stride = (a.width() * 4) as usize;
  let raw_a = a.as_raw();
  let raw_b = b.as_raw();
  let b_first = &raw_b[0..stride.min(raw_b.len())];
  let mut best_k: Option<usize> = None;
  let mut verified = 0usize;
  for i in (0..h_a).rev() {
    if h_a - i > h_b * 2 || verified >= 8 {
      break;
    }
    let k = h_a - i;
    if k < min_k || k > max_k {
      continue;
    }
    let row = &raw_a[i * stride..(i * stride + stride).min(raw_a.len())];
    if !rows_fuzzy_equal(row, b_first, 2) {
      continue;
    }
    verified += 1;
    let mut matched = 0usize;
    for j in 0..k {
      let ra = &raw_a[(i + j) * stride..((i + j) * stride + stride).min(raw_a.len())];
      let rb = &raw_b[j * stride..(j * stride + stride).min(raw_b.len())];
      if rows_fuzzy_equal(ra, rb, 2) {
        matched += 1;
      }
    }
    if matched * 20 >= k * 19 && best_k.map_or(true, |best| k > best) {
      best_k = Some(k);
    }
  }
  best_k.map(|k| k as u32)
}

/// 垂直拼接全部帧：相邻帧按重叠行数去重；无重叠时整屏直连并记录警告码，
/// 绝不静默丢帧。
pub(crate) fn stitch_frames(frames: &[RgbaImage]) -> Result<(RgbaImage, Vec<String>), String> {
  if frames.is_empty() {
    return Err("没有可拼接的帧".to_string());
  }
  let width = frames[0].width();
  let mut out = frames[0].clone();
  let mut warnings = Vec::new();
  let mut warned = false;

  for b in frames.iter().skip(1) {
    if b.width() != width {
      return Err("捕获帧宽度不一致，无法拼接".to_string());
    }
    let overlap = best_overlap(&out, b);
    match overlap {
      Some(k) if k == b.height() => {
        // 新一屏与已拼内容完全重复（未滚动），跳过
      }
      Some(k) if k == out.height() && b.height() >= out.height() => {
        // 一次滚过整屏：新帧完全覆盖旧内容，整体替换
        out = b.clone();
      }
      Some(k) => {
        // 常规滚动：新帧去掉与已拼内容重叠的头部后追加
        let tail_height = b.height() - k;
        let mut merged = RgbaImage::new(width, out.height() + tail_height);
        image::imageops::overlay(&mut merged, &out, 0, 0);
        let tail = image::imageops::crop_imm(b, 0, k, width, tail_height).to_image();
        image::imageops::overlay(&mut merged, &tail, 0, out.height() as i64);
        out = merged;
      }
      None => {
        let mut merged = RgbaImage::new(width, out.height() + b.height());
        image::imageops::overlay(&mut merged, &out, 0, 0);
        image::imageops::overlay(&mut merged, b, 0, out.height() as i64);
        out = merged;
        if !warned {
          warnings.push("stitch_fallback".to_string());
          warned = true;
        }
      }
    }
  }
  Ok((out, warnings))
}

/// 列出历史截图（按修改时间倒序），供模块页「最近截图」展示。
pub fn list_recent(app: &AppHandle) -> Result<Vec<ScreenshotItem>, String> {
  let dir = screenshots_dir(app)?;
  let mut items = Vec::new();
  let legacy = app
    .path()
    .app_data_dir()
    .map_err(|e| e.to_string())?
    .join(SCREENSHOTS_DIR_NAME);
  let entries = fs::read_dir(&dir)
    .map_err(|e| e.to_string())?
    .chain(fs::read_dir(legacy).into_iter().flatten());
  for entry in entries.flatten() {
    let path = entry.path();
    let kind = match path.extension().and_then(|e| e.to_str()) {
      Some("png") => "image",
      Some("mp4") => "video",
      _ => continue,
    };
    let meta = match entry.metadata() {
      Ok(m) => m,
      Err(_) => continue,
    };
    let modified_ms = meta
      .modified()
      .ok()
      .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
      .map(|d| d.as_millis() as i64)
      .unwrap_or(0);
    items.push(ScreenshotItem {
      name: path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default(),
      path: path.to_string_lossy().into_owned(),
      size: meta.len(),
      modified_ms,
      kind: kind.into(),
    });
  }
  items.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
  items.truncate(24);
  Ok(items)
}

// ---- Tauri commands ----

#[tauri::command]
pub async fn screenshot_begin(app: AppHandle) -> Result<Vec<MonitorFrameInfo>, String> {
  begin_capture(app).await
}

#[tauri::command]
pub fn screenshot_frame_info(
  state: State<'_, ScreenshotState>,
  monitor_index: usize,
) -> Result<MonitorFrameInfo, String> {
  let guard = state.session.lock().unwrap();
  let session = guard
    .as_ref()
    .ok_or_else(|| "没有正在进行的截图会话".to_string())?;
  let frame = session
    .monitors
    .iter()
    .find(|m| m.index == monitor_index)
    .ok_or_else(|| format!("显示器 {} 不在本轮捕获中", monitor_index))?;
  Ok(MonitorFrameInfo {
    index: frame.index,
    frame_path: frame.frame_path.to_string_lossy().into_owned(),
    x: frame.x,
    y: frame.y,
    width: frame.width,
    height: frame.height,
  })
}

#[tauri::command]
pub async fn screenshot_finish(
  app: AppHandle,
  monitor_index: usize,
  x: i32,
  y: i32,
  width: u32,
  height: u32,
  action: String,
) -> Result<ScreenshotOutcome, String> {
  finish_capture(app, monitor_index, x, y, width, height, action).await
}

#[tauri::command]
pub fn screenshot_cancel(app: AppHandle) -> Result<(), String> {
  cancel_capture(app)
}

// ---- 滚动截长图命令 ----

/// 进入滚动截取模式：固定选区、捕获第一屏、隐藏覆盖窗并弹出悬浮控制条。
#[tauri::command]
pub async fn screenshot_scroll_begin(
  app: AppHandle,
  monitor_index: usize,
  x: i32,
  y: i32,
  width: u32,
  height: u32,
) -> Result<ScrollStatus, String> {
  let state = app.state::<ScreenshotState>();
  let geometry = session_monitor_geometry(&state, monitor_index)?;
  let (_, _, frame_w, frame_h) = geometry;
  let (rx, ry, rw, rh) = clamp_crop_rect(frame_w, frame_h, x, y, width, height);

  if state.scroll.lock().unwrap().is_some() {
    return Err("已有滚动截取进行中".to_string());
  }

  // 第一屏直接取自冻结帧：用户框选时看到的内容，且不会把覆盖窗自身的
  // 边框/控件捕获进画面（此刻覆盖窗尚未隐藏）。
  let frame_path = session_frame_path(&state, monitor_index)?;
  let frame = tauri::async_runtime::spawn_blocking(move || -> Result<image::RgbaImage, String> {
    let img = image::open(&frame_path).map_err(|e| format!("读取冻结帧失败: {}", e))?;
    Ok(image::imageops::crop_imm(&img, rx, ry, rw, rh).to_image())
  })
  .await
  .map_err(|e| format!("捕获任务异常: {}", e))??;

  *state.scroll.lock().unwrap() = Some(ScrollSession {
    monitor_index,
    x: rx,
    y: ry,
    width: rw,
    height: rh,
    frames: vec![frame],
  });

  // 保留选区框（点击穿透 + 排除捕获），让用户看得见截取范围并自由滚动页面
  keep_selection_overlay(&app, monitor_index);
  show_toolbar(&app, geometry, (rx, ry, rw, rh));
  register_scroll_hotkeys(&app);
  Ok(ScrollStatus { frame_count: 1 })
}

/// 捕获一屏：把当前屏幕同一区域追加进帧序列。
#[tauri::command]
pub async fn screenshot_scroll_capture(app: AppHandle) -> Result<ScrollStatus, String> {
  let state = app.state::<ScreenshotState>();
  let (monitor_index, rx, ry, rw, rh) = {
    let guard = state.scroll.lock().unwrap();
    let session = guard
      .as_ref()
      .ok_or_else(|| "没有正在进行的滚动截取".to_string())?;
    (
      session.monitor_index,
      session.x,
      session.y,
      session.width,
      session.height,
    )
  };

  let frame = tauri::async_runtime::spawn_blocking(move || {
    capture_live_region(monitor_index, rx, ry, rw, rh)
  })
  .await
  .map_err(|e| format!("捕获任务异常: {}", e))??;

  let count = {
    let mut guard = state.scroll.lock().unwrap();
    let session = guard
      .as_mut()
      .ok_or_else(|| "没有正在进行的滚动截取".to_string())?;
    session.frames.push(frame);
    session.frames.len() as u32
  };
  let _ = app.emit_to(
    TOOLBAR_LABEL,
    "scroll-frame",
    &ScrollStatus { frame_count: count },
  );
  Ok(ScrollStatus { frame_count: count })
}

/// 完成滚动截取：重叠检测拼接为单张长图并入库。
#[tauri::command]
pub async fn screenshot_scroll_finish(app: AppHandle) -> Result<ScreenshotOutcome, String> {
  let state = app.state::<ScreenshotState>();
  let session = state
    .scroll
    .lock()
    .unwrap()
    .take()
    .ok_or_else(|| "没有正在进行的滚动截取".to_string())?;
  unregister_scroll_hotkeys(&app);
  if session.frames.is_empty() {
    teardown_all(&app);
    return Err("没有捕获到任何帧".to_string());
  }

  let frames = session.frames;
  let (png_bytes, warnings) =
    tauri::async_runtime::spawn_blocking(move || -> Result<(Vec<u8>, Vec<String>), String> {
      let (image, warnings) = stitch_frames(&frames)?;
      let mut png = Vec::new();
      image
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| format!("PNG 编码失败: {}", e))?;
      Ok((png, warnings))
    })
    .await
    .map_err(|e| format!("拼接任务异常: {}", e))??;

  let dir = screenshots_dir(&app)?;
  let path = dir.join(format!("Scroll_{}", timestamped_filename("png")));
  fs::write(&path, &png_bytes).map_err(|e| format!("保存长图失败: {}", e))?;
  let path_str = path.to_string_lossy().into_owned();
  let _ = app.emit_to("main", "screenshot-saved", &path_str);
  if !warnings.is_empty() {
    let _ = app.emit_to("main", "screenshot-notice", &warnings);
  }

  teardown_all(&app);
  Ok(ScreenshotOutcome {
    action: "scroll".into(),
    saved_path: Some(path_str),
    warnings,
  })
}

/// 取消滚动截取：丢弃全部帧，不产出文件。
#[tauri::command]
pub fn screenshot_scroll_cancel(app: AppHandle) -> Result<(), String> {
  let state = app.state::<ScreenshotState>();
  let had_scroll = state.scroll.lock().unwrap().take().is_some();
  if had_scroll {
    unregister_scroll_hotkeys(&app);
  }
  teardown_all(&app);
  Ok(())
}

/// 悬浮控制条挂载时查询当前模式与帧数；无活动会话时返回错误以便其自关闭。
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolbarInfo {
  /// scroll | record
  pub mode: String,
  pub frame_count: u32,
  /// 录屏最长时长（毫秒），滚动模式为 0
  pub max_duration_ms: u64,
}

#[tauri::command]
pub fn screenshot_toolbar_info(app: AppHandle) -> Result<ToolbarInfo, String> {
  let state = app.state::<ScreenshotState>();
  if let Some(scroll) = state.scroll.lock().unwrap().as_ref() {
    return Ok(ToolbarInfo {
      mode: "scroll".into(),
      frame_count: scroll.frames.len() as u32,
      max_duration_ms: 0,
    });
  }
  if let Some(max_ms) = crate::api::screenshot_record::recording_max_duration(&app) {
    return Ok(ToolbarInfo {
      mode: "record".into(),
      frame_count: 0,
      max_duration_ms: max_ms,
    });
  }
  Err("没有进行中的捕获会话".to_string())
}

/// 用系统默认程序打开截图库文件（录屏 MP4 的「打开文件」入口）。
#[tauri::command]
pub fn screenshot_open_file(path: String) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    std::process::Command::new("explorer")
      .arg(&path)
      .spawn()
      .map_err(|e| e.to_string())?;
  }
  #[cfg(target_os = "macos")]
  {
    std::process::Command::new("open")
      .arg(&path)
      .spawn()
      .map_err(|e| e.to_string())?;
  }
  #[cfg(target_os = "linux")]
  {
    std::process::Command::new("xdg-open")
      .arg(&path)
      .spawn()
      .map_err(|e| e.to_string())?;
  }
  Ok(())
}

#[tauri::command]
pub fn list_screenshots(app: AppHandle) -> Result<Vec<ScreenshotItem>, String> {
  list_recent(&app)
}

/// 美化结果另存为新图片，保留原始截图。
#[tauri::command]
pub async fn screenshot_save_styled(app: AppHandle, data_url: String) -> Result<String, String> {
  let dir = screenshots_dir(&app)?;
  let path = tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
    use base64::Engine;
    if data_url.len() > 64 * 1024 * 1024 {
      return Err("图片过大".into());
    }
    let encoded = data_url
      .strip_prefix("data:image/png;base64,")
      .ok_or("仅支持 PNG 图片")?;
    let bytes = base64::engine::general_purpose::STANDARD
      .decode(encoded)
      .map_err(|e| e.to_string())?;
    let mut reader =
      image::ImageReader::with_format(std::io::Cursor::new(&bytes), image::ImageFormat::Png);
    reader.limits(image::Limits::default());
    reader.decode().map_err(|e| format!("无效图片: {}", e))?;
    let path = dir.join(format!("Styled_{}", timestamped_filename("png")));
    fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
  })
  .await
  .map_err(|e| e.to_string())??;
  let _ = app.emit_to("main", "screenshot-saved", &path);
  Ok(path)
}

/// 删除截图库中的一条记录（图片/视频）。
/// 只允许删除截图目录内的文件，避免被当作任意文件删除接口。
#[tauri::command]
pub fn screenshot_delete(app: AppHandle, path: String) -> Result<(), String> {
  let dir = screenshots_dir(&app)?;
  let target = PathBuf::from(&path);

  // 规范化后必须仍在截图目录内（同时挡住 ../ 与符号链接逃逸）
  let canon_dir = fs::canonicalize(&dir).map_err(|e| format!("截图目录不可用: {}", e))?;
  let canon = fs::canonicalize(&target).map_err(|e| format!("文件不存在: {}", e))?;
  if !canon.starts_with(&canon_dir) {
    return Err("只能删除截图库中的文件".to_string());
  }
  if canon.is_dir() {
    return Err("目标是目录，拒绝删除".to_string());
  }

  fs::remove_file(&canon).map_err(|e| format!("删除失败: {}", e))?;
  let _ = app.emit_to("main", "screenshot-deleted", &path);
  Ok(())
}

/// 读取图片文件为 data URL，供 AI 聊天发送多模态消息使用。
#[tauri::command]
pub fn read_image_data_url(path: String) -> Result<String, String> {
  let bytes = fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
  let mime = match path.rsplit('.').next().map(|e| e.to_ascii_lowercase()) {
    Some(ref ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
    Some(ref ext) if ext == "webp" => "image/webp",
    Some(ref ext) if ext == "gif" => "image/gif",
    _ => "image/png",
  };
  use base64::Engine;
  let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
  Ok(format!("data:{};base64,{}", mime, encoded))
}

#[cfg(test)]
mod tests {
  use super::{
    best_overlap, clamp_crop_rect, stitch_frames, MonitorFrameInfo, ScreenshotItem,
    ScreenshotOutcome,
  };
  use image::{Rgba, RgbaImage};

  /// 行号决定颜色的合成帧：row y 的 RGB = (y + offset, ...)
  fn frame(rows: u32, offset: u32) -> RgbaImage {
    RgbaImage::from_fn(24, rows, |_x, y| {
      let v = (y + offset) as u8;
      Rgba([v, v, v, 255])
    })
  }

  /// 前端按 camelCase 读取这些字段（如 info.framePath），漏标 rename_all 会让
  /// 路径变成 undefined 并导致覆盖层只显示黑屏，这里锁定字段名。
  #[test]
  fn serializes_camel_case_for_frontend() {
    let frame = MonitorFrameInfo {
      index: 0,
      frame_path: "C:/frames/frame_0.png".into(),
      x: 0,
      y: 0,
      width: 1920,
      height: 1080,
    };
    let json = serde_json::to_value(&frame).unwrap();
    assert!(
      json.get("framePath").is_some(),
      "expected framePath, got {json}"
    );

    let outcome = ScreenshotOutcome {
      action: "save".into(),
      saved_path: Some("C:/shots/a.png".into()),
      warnings: Vec::new(),
    };
    let json = serde_json::to_value(&outcome).unwrap();
    assert!(
      json.get("savedPath").is_some(),
      "expected savedPath, got {json}"
    );

    let item = ScreenshotItem {
      name: "a.png".into(),
      path: "C:/shots/a.png".into(),
      size: 1,
      modified_ms: 2,
      kind: "image".into(),
    };
    let json = serde_json::to_value(&item).unwrap();
    assert!(
      json.get("modifiedMs").is_some(),
      "expected modifiedMs, got {json}"
    );
  }

  #[test]
  fn keeps_valid_rect() {
    assert_eq!(
      clamp_crop_rect(1920, 1080, 100, 200, 300, 400),
      (100, 200, 300, 400)
    );
  }

  #[test]
  fn clamps_negative_origin() {
    assert_eq!(
      clamp_crop_rect(1920, 1080, -50, -10, 300, 400),
      (0, 0, 300, 400)
    );
  }

  #[test]
  fn clamps_overshoot() {
    assert_eq!(
      clamp_crop_rect(1920, 1080, 1800, 1000, 500, 200),
      (1800, 1000, 120, 80)
    );
  }

  #[test]
  fn clamps_zero_size_to_one() {
    assert_eq!(
      clamp_crop_rect(1920, 1080, 100, 100, 0, 0),
      (100, 100, 1, 1)
    );
  }

  #[test]
  fn clamps_origin_beyond_frame() {
    assert_eq!(clamp_crop_rect(100, 100, 500, 500, 10, 10), (99, 99, 1, 1));
  }

  #[test]
  fn detects_exact_scroll_overlap() {
    let a = frame(30, 0);
    // 向下滚 10 行：b 的首行对应 a 的第 10 行 → 重叠 20 行
    let b = frame(30, 10);
    assert_eq!(best_overlap(&a, &b), Some(20));
    // 未滚动：完全重复
    assert_eq!(best_overlap(&a, &a), Some(30));
    // 滚动超过一屏（剩余重叠 < 8 行）→ 无可靠重叠
    let c = frame(30, 26);
    assert_eq!(best_overlap(&a, &c), None);
  }

  #[test]
  fn detects_fuzzy_overlap_with_noise() {
    let a = frame(30, 0);
    let mut b = frame(30, 10);
    // 模拟抗锯齿噪声：部分像素加 ±2 抖动，精确哈希必然失配
    for (i, pixel) in b.pixels_mut().enumerate() {
      let jitter = if i % 3 == 0 { 2 } else { 0 };
      pixel[0] = pixel[0].saturating_add(jitter);
    }
    assert_eq!(best_overlap(&a, &b), Some(20));
  }

  #[test]
  fn stitches_three_screens_with_continuity() {
    let frames = vec![frame(30, 0), frame(30, 10), frame(30, 10)];
    let (image, warnings) = stitch_frames(&frames).unwrap();
    assert_eq!(image.height(), 40); // 30 + 追加 10 + 重复帧去重
    assert!(warnings.is_empty());
    // 内容连续：第 y 行颜色 = y
    for y in 0..image.height() {
      assert_eq!(image.get_pixel(0, y)[0], y as u8, "row {}", y);
    }
  }

  #[test]
  fn falls_back_to_concat_without_silent_loss() {
    let a = frame(20, 0);
    let b = frame(20, 100); // 完全不同的内容
    let (image, warnings) = stitch_frames(&[a, b]).unwrap();
    assert_eq!(image.height(), 40); // 一行不丢
    assert!(warnings.contains(&"stitch_fallback".to_string()));
  }

  #[test]
  fn skips_duplicate_capture_without_growth() {
    let frames = vec![frame(20, 0), frame(20, 0)];
    let (image, warnings) = stitch_frames(&frames).unwrap();
    assert_eq!(image.height(), 20);
    assert!(warnings.is_empty());
  }

  #[test]
  fn rejects_mixed_width_frames() {
    let a = frame(10, 0);
    let b = RgbaImage::from_fn(16, 10, |_, _| Rgba([0, 0, 0, 255]));
    assert!(stitch_frames(&[a, b]).is_err());
  }
}
