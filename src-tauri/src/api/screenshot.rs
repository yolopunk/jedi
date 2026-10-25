// 截图裁剪：冻结全屏 → 覆盖窗口框选 → 裁剪输出（剪贴板/文件/AI 聊天）
//
// 流程：begin 时用 xcap 逐显示器捕获并落盘临时 PNG，为每个显示器创建一个
// 无边框置顶覆盖窗口（路由 /screenshot-overlay，显示器编号编码在窗口 label 中）。
// 覆盖窗口回传物理像素选区后由 finish 统一裁剪、输出并拆除全部覆盖窗口。

use crate::api::wallpapers;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::webview::WebviewWindowBuilder;
use tauri::{
  AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, State, WebviewUrl,
};

pub const OVERLAY_LABEL_PREFIX: &str = "screenshot-overlay-";
pub const SCREENSHOTS_DIR_NAME: &str = "screenshots";
pub const FRAME_CACHE_DIR_NAME: &str = "screenshot-frames";

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
  /// 命中的输出动作：copy | save | chat
  pub action: String,
  /// 已留存到截图画廊的文件路径
  pub saved_path: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotItem {
  pub name: String,
  pub path: String,
  pub size: u64,
  /// 修改时间（Unix 毫秒）
  pub modified_ms: i64,
}

/// 全局会话状态；同一时刻只允许一轮截图。
#[derive(Default)]
pub struct ScreenshotState {
  session: Mutex<Option<CaptureSession>>,
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

fn timestamped_filename(ext: &str) -> String {
  let now = chrono::Local::now();
  format!("Jedi_{}.{}", now.format("%Y%m%d_%H%M%S%3f"), ext)
}

/// 拆除会话：关闭所有覆盖窗口并删除冻结帧临时文件。
fn teardown(app: &AppHandle, session: CaptureSession) {
  for monitor in &session.monitors {
    if let Some(window) = app.get_webview_window(&monitor.label) {
      let _ = window.close();
    }
    let _ = fs::remove_file(&monitor.frame_path);
  }
}

/// 捕获全部显示器并弹出覆盖窗口。命令、托盘与全局快捷键共用此入口。
pub async fn begin_capture(app: AppHandle) -> Result<Vec<MonitorFrameInfo>, String> {
  let state = app.state::<ScreenshotState>();

  // 重复触发时先拆掉上一轮，保证幂等
  if let Some(prev) = state.session.lock().unwrap().take() {
    teardown(&app, prev);
  }

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
        let builder = WebviewWindowBuilder::new(
          &app_for_windows,
          format!("{}{}", OVERLAY_LABEL_PREFIX, info.index),
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
        // 只有主显示器的窗口接管键盘焦点，其余显示器仅展示冻结帧
        .focused(format!("{}{}", OVERLAY_LABEL_PREFIX, info.index) == primary_label);
        if let Ok(window) = builder.build() {
          let _ = window.set_position(Position::Physical(PhysicalPosition::new(info.x, info.y)));
          let _ = window.set_size(Size::Physical(PhysicalSize::new(info.width, info.height)));
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
        })
      }
      "save" => {
        wallpapers::show_in_folder(path_str.clone()).await?;
        Ok(ScreenshotOutcome {
          action: "save".into(),
          saved_path: Some(path_str),
        })
      }
      "chat" => {
        app
          .emit_to("main", "screenshot-captured", &path_str)
          .map_err(|e| format!("通知主窗口失败: {}", e))?;
        Ok(ScreenshotOutcome {
          action: "chat".into(),
          saved_path: Some(path_str),
        })
      }
      "ocr" => {
        app
          .emit_to("main", "screenshot-ocr", &path_str)
          .map_err(|e| format!("通知主窗口失败: {}", e))?;
        Ok(ScreenshotOutcome {
          action: "ocr".into(),
          saved_path: Some(path_str),
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
  if let Some(session) = state.session.lock().unwrap().take() {
    teardown(&app, session);
  }
  Ok(())
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
    if path.extension().and_then(|e| e.to_str()) != Some("png") {
      continue;
    }
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
  use super::{clamp_crop_rect, MonitorFrameInfo, ScreenshotItem, ScreenshotOutcome};

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
}
