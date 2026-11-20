// 截图裁剪：冻结全屏 → 覆盖窗口框选 → 裁剪输出（剪贴板/文件/AI 聊天）
//
// 流程：begin 时用 xcap 逐显示器捕获并落盘临时 PNG，为每个显示器创建一个
// 无边框置顶覆盖窗口（路由 /screenshot-overlay，显示器编号编码在窗口 label 中）。
// 覆盖窗口回传物理像素选区后由 finish 统一裁剪、输出并拆除全部覆盖窗口。

use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::webview::WebviewWindowBuilder;
use tauri::{
  AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl,
};

mod window_targets;
use window_targets::WindowRect;

pub const OVERLAY_LABEL_PREFIX: &str = "screenshot-overlay-";
pub const TOOLBAR_LABEL: &str = "screenshot-toolbar";
pub const PIN_LABEL_PREFIX: &str = "screenshot-pin-";
pub const SCREENSHOTS_DIR_NAME: &str = "screenshots";
pub const FRAME_CACHE_DIR_NAME: &str = "screenshot-frames";
/// 默认截图快捷键
pub const DEFAULT_CAPTURE_HOTKEY: &str = "F1";

/// 一轮截图会话：所有显示器的冻结帧与对应覆盖窗口。
pub struct CaptureSession {
  monitors: Vec<MonitorFrame>,
  cursor: Option<(f64, f64)>,
  focus_index: usize,
}

#[derive(Clone)]
pub struct MonitorFrame {
  /// 显示器序号，与覆盖窗口 label 后缀一致
  index: usize,
  /// 覆盖窗口 label：`screenshot-overlay-<index>`
  label: String,
  /// 冻结帧 PNG 临时文件路径
  frame_path: PathBuf,
  /// 最后一个会话/输出引用释放时清理单个冻结帧，包含抓帧中途失败的路径。
  _frame_file: Arc<tempfile::TempPath>,
  /// xcap 的桌面坐标，macOS 为点，Windows 为物理像素。
  bounds: WindowRect,
  /// 实际冻结帧的像素尺寸（Retina 与桌面坐标尺寸不同）。
  width: u32,
  height: u32,
  /// 显示器缩放比，用于桌面上的贴图边距。
  scale: f32,
  window_targets: Vec<WindowRect>,
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
  window_targets: Vec<WindowRect>,
  cursor: Option<(f64, f64)>,
  focused: bool,
}

impl MonitorFrame {
  fn info(&self, cursor: Option<(f64, f64)>, focus_index: usize) -> MonitorFrameInfo {
    let bounds = self.bounds;
    let sx = f64::from(self.width) / f64::from(bounds.width);
    let sy = f64::from(self.height) / f64::from(bounds.height);
    MonitorFrameInfo {
      index: self.index,
      frame_path: self.frame_path.to_string_lossy().into_owned(),
      x: bounds.x,
      y: bounds.y,
      width: self.width,
      height: self.height,
      window_targets: self
        .window_targets
        .iter()
        .map(|target| {
          let x = (f64::from(target.x) * sx).round() as i32;
          let y = (f64::from(target.y) * sy).round() as i32;
          let right = ((f64::from(target.x) + f64::from(target.width)) * sx).round() as i32;
          let bottom = ((f64::from(target.y) + f64::from(target.height)) * sy).round() as i32;
          WindowRect {
            x,
            y,
            width: (right - x) as u32,
            height: (bottom - y) as u32,
          }
        })
        .collect(),
      cursor: cursor.and_then(|(x, y)| {
        let x = (x - f64::from(bounds.x)) * sx;
        let y = (y - f64::from(bounds.y)) * sy;
        ((0.0..f64::from(self.width)).contains(&x) && (0.0..f64::from(self.height)).contains(&y))
          .then_some((x, y))
      }),
      focused: self.index == focus_index,
    }
  }
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

/// 一轮滚动截长图会话：固定选区 + 随滚动自动捕获，增量拼接成单张长图。
pub struct ScrollSession {
  monitor_index: usize,
  /// 显示器物理原点：捕获时按原点匹配显示器（多屏枚举顺序不稳定，不能按下标）
  monitor_origin: (i32, i32),
  /// 选区在显示器帧内的物理像素坐标
  x: u32,
  y: u32,
  width: u32,
  height: u32,
  /// 增量拼接的累积长图
  stitched: RgbaImage,
  /// 最近一次匹配失败、等待恢复的帧（用户回滚后重新匹配则丢弃）
  pending: Option<RgbaImage>,
  /// 已处理帧数（含未匹配帧）
  frame_count: u32,
  /// 最近一帧是否成功拼上（前端用它提示「未匹配，回滚一点再继续」）
  match_ok: bool,
  /// 看守线程停止标志（线程句柄在 ScreenshotState.scroll_watcher）
  pub(crate) stop: Arc<AtomicBool>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScrollStatus {
  pub frame_count: u32,
  /// 累积长图高度（物理像素）
  pub stitched_height: u32,
  /// 累积长图缩略图 data URL（前端实时预览）
  #[serde(default)]
  pub stitched_thumb: String,
  /// 最近一帧是否匹配成功
  pub match_ok: bool,
}

/// 全局会话状态；同一时刻只允许一轮截图。
pub struct ScreenshotState {
  session: Mutex<Option<CaptureSession>>,
  /// 防止全局快捷键在抓帧或保存对话框期间拆除当前会话。
  pub(crate) capture_operation: tokio::sync::Mutex<()>,
  /// 输出或实时模式初始化期间拒绝其它屏幕的取消；不阻止覆盖窗初始化失败时取消。
  pub(crate) capture_output: tokio::sync::Mutex<()>,
  pub(crate) scroll: Mutex<Option<ScrollSession>>,
  /// 当前生效的截图全局快捷键（accelerator 字符串）
  pub(crate) hotkey: Mutex<String>,
  /// 贴图窗口待领取的图片路径：窗口 label -> 图片绝对路径
  pub(crate) pin_paths: Mutex<HashMap<String, String>>,
}

impl Default for ScreenshotState {
  fn default() -> Self {
    Self {
      session: Mutex::new(None),
      capture_operation: tokio::sync::Mutex::new(()),
      capture_output: tokio::sync::Mutex::new(()),
      scroll: Mutex::new(None),
      // 空串 = 尚未注册任何快捷键；启动时会注册默认值。
      // 这里若填默认值，register_capture_hotkey 的「同值跳过」会让启动注册被跳过。
      hotkey: Mutex::new(String::new()),
      pin_paths: Mutex::new(HashMap::new()),
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

/// 把图片缩到指定宽度的小 PNG data URL。
fn thumb_data_url_at(img: &RgbaImage, thumb_w: u32) -> Option<String> {
  use base64::Engine;
  let scale = if img.width() > thumb_w {
    thumb_w as f32 / img.width() as f32
  } else {
    1.0
  };
  let thumb = image::imageops::resize(
    img,
    ((img.width() as f32 * scale).round() as u32).max(1),
    ((img.height() as f32 * scale).round() as u32).max(1),
    image::imageops::FilterType::Triangle,
  );
  let mut png = Vec::new();
  image::DynamicImage::ImageRgba8(thumb)
    .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
    .ok()?;
  Some(format!(
    "data:image/png;base64,{}",
    base64::engine::general_purpose::STANDARD.encode(png)
  ))
}

/// 滚动会话状态快照：累积长图缩略图（超高时截顶部预览段）。
fn scroll_status_of(session: &ScrollSession) -> ScrollStatus {
  const THUMB_W: u32 = 120;
  const PREVIEW_MAX_H: u32 = 2400;
  let preview = if session.stitched.height() > PREVIEW_MAX_H {
    image::imageops::crop_imm(
      &session.stitched,
      0,
      0,
      session.stitched.width(),
      PREVIEW_MAX_H,
    )
    .to_image()
  } else {
    session.stitched.clone()
  };
  ScrollStatus {
    frame_count: session.frame_count,
    stitched_height: session.stitched.height(),
    stitched_thumb: thumb_data_url_at(&preview, THUMB_W).unwrap_or_default(),
    match_ok: session.match_ok,
  }
}

/// 整数因子降采样（看守线程差分基准用，Nearest 足够）。
fn downsample(img: &RgbaImage, factor: u32) -> RgbaImage {
  let w = (img.width() / factor).max(1);
  let h = (img.height() / factor).max(1);
  image::imageops::resize(img, w, h, image::imageops::FilterType::Nearest)
}

/// 逐字节平均绝对差（RGB 参与，忽略 alpha），同尺寸图片间比较。
fn mean_abs_diff(a: &RgbaImage, b: &RgbaImage) -> f32 {
  if a.dimensions() != b.dimensions() || a.as_raw().is_empty() {
    return f32::MAX;
  }
  let (ra, rb) = (a.as_raw(), b.as_raw());
  let mut total: u64 = 0;
  let mut count: u64 = 0;
  for (pa, pb) in ra.chunks_exact(4).zip(rb.chunks_exact(4)) {
    total += (pa[0] as i16 - pb[0] as i16).unsigned_abs() as u64
      + (pa[1] as i16 - pb[1] as i16).unsigned_abs() as u64
      + (pa[2] as i16 - pb[2] as i16).unsigned_abs() as u64;
    count += 3;
  }
  if count == 0 {
    f32::MAX
  } else {
    total as f32 / count as f32
  }
}

/// 用一张已入库的图片开一个置顶贴图窗口（Pin）。
/// 路径放进 pin_paths 由贴图窗口挂载后用 `screenshot_pin_take` 领取，
/// 避免 Windows 路径进 URL query 的编码问题。
/// `position`：窗口物理屏幕坐标（贴在截取位置，PixPin 行为）；None 用系统默认级联位置。
pub(crate) async fn spawn_pin_window(
  app: &AppHandle,
  path: &str,
  position: Option<Position>,
) -> Result<(), String> {
  let (img_w, img_h) = tauri::async_runtime::spawn_blocking({
    let path = path.to_string();
    move || -> Result<(u32, u32), String> {
      let reader = image::ImageReader::open(&path).map_err(|e| format!("读取图片失败: {}", e))?;
      reader
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_dimensions()
        .map_err(|e| format!("读取图片尺寸失败: {}", e))
    }
  })
  .await
  .map_err(|e| format!("读取任务异常: {}", e))??;

  // 初始窗口尺寸（逻辑像素）：默认 1:1，过大时按比例缩到 760×520 以内
  let mut w = img_w as f64;
  let mut h = img_h as f64;
  let fit = (760.0 / w).min(520.0 / h).min(1.0);
  w *= fit;
  h *= fit;

  let label = format!(
    "{}{}",
    PIN_LABEL_PREFIX,
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|d| d.as_millis())
      .unwrap_or_default()
  );
  app
    .state::<ScreenshotState>()
    .pin_paths
    .lock()
    .unwrap()
    .insert(label.clone(), path.to_string());

  let app_for_window = app.clone();
  // 建窗结果经 channel 回传：失败要让调用方看到错误，而不是静默「没效果」
  let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
  app
    .run_on_main_thread(move || {
      let builder = WebviewWindowBuilder::new(
        &app_for_window,
        &label,
        WebviewUrl::App("screenshot-pin".into()),
      )
      .title("Jedi Pin")
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
      .inner_size(w.max(120.0), h.max(80.0));
      let sent = match builder.build() {
        Ok(window) => {
          if let Some(position) = position {
            let _ = window.set_position(position);
          }
          Ok(())
        }
        Err(e) => Err(format!("创建贴图窗口失败: {}", e)),
      };
      let _ = tx.send(sent);
    })
    .map_err(|e| format!("调度贴图建窗失败: {}", e))?;
  rx.recv()
    .map_err(|e| format!("等待贴图建窗结果失败: {}", e))??;
  Ok(())
}

/// 拆除会话：关闭所有覆盖窗口并删除冻结帧临时文件。
/// 用 destroy 而非 close：close 会走 CloseRequested（主窗口的托盘逻辑），
/// 辅助窗一旦被拦截成 hide，同名窗口残留会让下一轮截图的建窗失败。
fn teardown(app: &AppHandle, session: CaptureSession) {
  for monitor in &session.monitors {
    if let Some(window) = app.get_webview_window(&monitor.label) {
      let _ = window.destroy();
    }
  }
}

/// 捕获全部显示器并弹出覆盖窗口。命令、托盘与全局快捷键共用此入口。
pub async fn begin_capture(app: AppHandle) -> Result<Vec<MonitorFrameInfo>, String> {
  let state = app.state::<ScreenshotState>();
  let _operation = state
    .capture_operation
    .try_lock()
    .map_err(|_| "截图操作进行中")?;
  // 保留已经打开的选区、长截图或录屏，防止重复按下快捷键丢失编辑。
  if let Some(session) = state.session.lock().unwrap().as_ref() {
    if session
      .monitors
      .iter()
      .any(|frame| app.get_webview_window(&frame.label).is_some())
    {
      return Ok(
        session
          .monitors
          .iter()
          .map(|frame| frame.info(session.cursor, session.focus_index))
          .collect(),
      );
    }
  }
  teardown_all(&app);

  let cache_dir = frame_cache_dir(&app)?;

  // 阻塞捕获 + PNG 编码挪出主线程
  let frames =
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<MonitorFrame>, String> {
      let monitors = xcap::Monitor::all().map_err(|e| format!("枚举显示器失败: {}", e))?;
      if monitors.is_empty() {
        return Err("未发现任何显示器".to_string());
      }
      let window_targets = window_targets::visible_windows();
      let mut frames = Vec::with_capacity(monitors.len());
      for (index, monitor) in monitors.into_iter().enumerate() {
        // macOS 未授予屏幕录制权限时 capture_image 会失败或返回空帧
        let image = monitor
          .capture_image()
          .map_err(|e| format!("捕获显示器 {} 失败: {}", index, e))?;
        let bounds = WindowRect {
          x: monitor.x().map_err(|e| e.to_string())?,
          y: monitor.y().map_err(|e| e.to_string())?,
          width: monitor.width().map_err(|e| e.to_string())?,
          height: monitor.height().map_err(|e| e.to_string())?,
        };
        let scale = monitor.scale_factor().map_err(|e| e.to_string())?.max(1.0);
        if bounds.width == 0 || bounds.height == 0 || image.width() == 0 || image.height() == 0 {
          return Err(format!("显示器 {} 的画面为空", index));
        }
        let frame_file = tempfile::Builder::new()
          .prefix("frame_")
          .suffix(".png")
          .tempfile_in(&cache_dir)
          .map_err(|e| format!("创建冻结帧失败: {}", e))?
          .into_temp_path();
        let frame_path = frame_file.to_path_buf();
        image.save(&frame_path).map_err(|e| e.to_string())?;
        frames.push(MonitorFrame {
          index,
          label: format!("{}{}", OVERLAY_LABEL_PREFIX, index),
          frame_path,
          _frame_file: Arc::new(frame_file),
          bounds,
          width: image.width(),
          height: image.height(),
          scale,
          window_targets: window_targets
            .iter()
            .filter_map(|target| target.on_monitor(bounds))
            .collect(),
        });
      }
      Ok(frames)
    })
    .await
    .map_err(|e| format!("捕获任务异常: {}", e))??;

  let cursor = desktop_cursor(&app);
  let focus_index = frames
    .iter()
    .position(|frame| {
      cursor.is_some_and(|(x, y)| {
        let bounds = frame.bounds;
        (f64::from(bounds.x)..f64::from(bounds.x) + f64::from(bounds.width)).contains(&x)
          && (f64::from(bounds.y)..f64::from(bounds.y) + f64::from(bounds.height)).contains(&y)
      })
    })
    .unwrap_or(0);
  let infos = frames
    .iter()
    .map(|frame| frame.info(cursor, focus_index))
    .collect();
  let overlays: Vec<_> = frames
    .iter()
    .map(|frame| (frame.label.clone(), frame.bounds))
    .collect();

  *state.session.lock().unwrap() = Some(CaptureSession {
    monitors: frames,
    cursor,
    focus_index,
  });

  // macOS 要求在主线程创建窗口；其余平台同样安全
  let app_for_windows = app.clone();
  let (tx, rx) = tokio::sync::oneshot::channel();
  let scheduled = app.run_on_main_thread(move || {
    let mut built = 0;
    for (index, (label, bounds)) in overlays.into_iter().enumerate() {
      if app_for_windows
        .state::<ScreenshotState>()
        .session
        .lock()
        .unwrap()
        .is_none()
      {
        break; // 某个覆盖窗加载失败并取消后，不再创建其余屏幕的窗口。
      }
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
      .focused(index == focus_index);
      match builder.build() {
        Ok(window) => {
          if app_for_windows
            .state::<ScreenshotState>()
            .session
            .lock()
            .unwrap()
            .is_none()
          {
            let _ = window.destroy();
            break;
          }
          #[cfg(target_os = "macos")]
          let (position, size) = (
            Position::Logical(tauri::LogicalPosition::new(
              f64::from(bounds.x),
              f64::from(bounds.y),
            )),
            Size::Logical(tauri::LogicalSize::new(
              f64::from(bounds.width),
              f64::from(bounds.height),
            )),
          );
          #[cfg(not(target_os = "macos"))]
          let (position, size) = (
            Position::Physical(PhysicalPosition::new(bounds.x, bounds.y)),
            Size::Physical(PhysicalSize::new(bounds.width, bounds.height)),
          );
          let _ = window.set_position(position);
          let _ = window.set_size(size);
          built += 1;
        }
        Err(e) => eprintln!("failed to create overlay window {}: {}", label, e),
      }
    }
    let _ = tx.send(built);
  });
  if scheduled.is_err() || rx.await.unwrap_or_default() == 0 {
    cancel_capture(&app);
    return Err("无法创建截图窗口".into());
  }

  Ok(infos)
}

fn desktop_cursor(app: &AppHandle) -> Option<(f64, f64)> {
  let cursor = app.cursor_position().ok()?;
  #[cfg(target_os = "macos")]
  let scale = app.primary_monitor().ok().flatten()?.scale_factor();
  #[cfg(not(target_os = "macos"))]
  let scale = 1.0;
  Some((cursor.x / scale, cursor.y / scale))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureOutput {
  monitor_index: usize,
  x: i32,
  y: i32,
  width: u32,
  height: u32,
  action: String,
  annotated_png: Option<String>,
  save_title: Option<String>,
}

/// 解码时限制分配与尺寸，确保标注结果仍是同一个选区大小。
fn decode_annotated_png(data_url: &str, width: u32, height: u32) -> Result<RgbaImage, String> {
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
    image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
  let mut limits = image::Limits::default();
  limits.max_image_width = Some(width);
  limits.max_image_height = Some(height);
  reader.limits(limits);
  let image = reader
    .decode()
    .map_err(|e| format!("无效的标注图片: {}", e))?;
  if (image.width(), image.height()) != (width, height) {
    return Err("标注图片尺寸与选区不符".into());
  }
  Ok(image.to_rgba8())
}

fn pin_position(frame: &MonitorFrame, x: u32, y: u32) -> Position {
  let bounds = frame.bounds;
  #[cfg(target_os = "macos")]
  let scale = 1.0;
  #[cfg(not(target_os = "macos"))]
  let scale = f64::from(frame.scale);
  // 贴图上方 44 点控制条和周围 7 点边距不承载图片。
  let px = f64::from(x) * f64::from(bounds.width) / f64::from(frame.width) - 7.0 * scale;
  let py = f64::from(y) * f64::from(bounds.height) / f64::from(frame.height) - 51.0 * scale;
  let px = f64::from(bounds.x) + px.clamp(0.0, f64::from(bounds.width.saturating_sub(200)));
  let py = f64::from(bounds.y) + py.clamp(0.0, f64::from(bounds.height.saturating_sub(120)));
  #[cfg(target_os = "macos")]
  return Position::Logical(tauri::LogicalPosition::new(px, py));
  #[cfg(not(target_os = "macos"))]
  Position::Physical(PhysicalPosition::new(px.round() as i32, py.round() as i32))
}

/// 取消截图：直接拆除会话，不产出任何文件。
fn cancel_capture(app: &AppHandle) {
  let state = app.state::<ScreenshotState>();
  let taken = state.session.lock().unwrap().take();
  if let Some(session) = taken {
    teardown(app, session);
  }
}

// ---- 滚动截长图 / 录屏共用基础设施 ----

/// 读取显示器的桌面原点和冻结帧像素尺寸；macOS 的桌面坐标使用点。
pub(crate) fn session_monitor_geometry(
  state: &ScreenshotState,
  monitor_index: usize,
) -> Result<(i32, i32, u32, u32), String> {
  let guard = state.session.lock().unwrap();
  let monitor = session_monitor(&guard, monitor_index)?;
  Ok((
    monitor.bounds.x,
    monitor.bounds.y,
    monitor.width,
    monitor.height,
  ))
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

/// 实时捕获固定区域：按显示器物理原点匹配显示器（多屏枚举顺序不稳定，
/// 按下标取会裁错屏），原点找不到时退回序号。
pub(crate) fn capture_live_region_at(
  monitor_index: usize,
  origin: (i32, i32),
  x: u32,
  y: u32,
  width: u32,
  height: u32,
) -> Result<RgbaImage, String> {
  let monitors = xcap::Monitor::all().map_err(|e| format!("枚举显示器失败: {}", e))?;
  let monitor = monitors
    .iter()
    .find(|m| match (m.x(), m.y()) {
      (Ok(mx), Ok(my)) => (mx, my) == origin,
      _ => false,
    })
    .or_else(|| monitors.get(monitor_index))
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

/// Windows：恢复窗口可被系统捕获（WDA_NONE）。滚动/录屏会话结束后
/// 把临时排除的贴图窗口还原——平时贴图应能被普通截图截到（PixPin 行为）。
#[cfg(target_os = "windows")]
pub(crate) fn include_in_capture(window: &tauri::WebviewWindow<tauri::Wry>) {
  use windows_sys::Win32::UI::WindowsAndMessaging::SetWindowDisplayAffinity;
  const WDA_NONE: u32 = 0x0;
  if let Ok(hwnd) = window.hwnd() {
    unsafe {
      SetWindowDisplayAffinity(hwnd.0 as _, WDA_NONE);
    }
  }
}
#[cfg(not(target_os = "windows"))]
pub(crate) fn include_in_capture(_window: &tauri::WebviewWindow<tauri::Wry>) {}

/// 滚动/录屏会话期间临时把所有贴图窗口排除出捕获（防止贴图入镜污染帧）。
/// `excluded=false` 时还原（会话结束调用）。
fn set_pins_excluded(app: &AppHandle, excluded: bool) {
  for window in app.webview_windows().values() {
    if !window.label().starts_with(PIN_LABEL_PREFIX) {
      continue;
    }
    if excluded {
      exclude_from_capture(window);
    } else {
      include_in_capture(window);
    }
  }
}

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
  let state = app.state::<ScreenshotState>();
  let frame = {
    let guard = state.session.lock().unwrap();
    let Some(frame) = guard.as_ref().and_then(|session| {
      session
        .monitors
        .iter()
        .find(|frame| (frame.bounds.x, frame.bounds.y, frame.width, frame.height) == monitor)
    }) else {
      return; // 取消后延迟任务不能重新弹出悬浮条。
    };
    frame.clone()
  };
  let bounds = frame.bounds;
  let (mx, my) = (f64::from(bounds.x), f64::from(bounds.y));
  let (mw, mh) = (f64::from(bounds.width), f64::from(bounds.height));
  let (rx, ry, rw, rh) = region;
  let sx = mw / f64::from(frame.width);
  let sy = mh / f64::from(frame.height);
  #[cfg(target_os = "macos")]
  let scale = 1.0;
  #[cfg(not(target_os = "macos"))]
  let scale = f64::from(frame.scale);
  let toolbar_w = 480.0 * scale;
  let toolbar_h = 64.0 * scale;
  let margin = 12.0 * scale;
  let region_top = my + f64::from(ry) * sy;
  let region_bottom = region_top + f64::from(rh) * sy;
  let below_y = region_bottom + margin;
  let py = if below_y + toolbar_h <= my + mh {
    below_y
  } else {
    (region_top - margin - toolbar_h).max(my)
  };
  let px = (mx + (f64::from(rx) + f64::from(rw) / 2.0) * sx - toolbar_w / 2.0)
    .clamp(mx, (mx + mw - toolbar_w).max(mx));
  #[cfg(target_os = "macos")]
  let position = Position::Logical(tauri::LogicalPosition::new(px, py));
  #[cfg(not(target_os = "macos"))]
  let position = Position::Physical(PhysicalPosition::new(px.round() as i32, py.round() as i32));

  let app_for_window = app.clone();
  let _ = app.run_on_main_thread(move || {
    let state = app_for_window.state::<ScreenshotState>();
    let current = state
      .session
      .lock()
      .unwrap()
      .as_ref()
      .is_some_and(|session| {
        session
          .monitors
          .iter()
          .any(|current| Arc::ptr_eq(&current._frame_file, &frame._frame_file))
      });
    if !current {
      return;
    }
    // 上一轮残留的悬浮条：复用窗口，通知其按新会话重新初始化并复位
    if let Some(existing) = app_for_window.get_webview_window(TOOLBAR_LABEL) {
      let _ = existing.set_position(position);
      exclude_from_capture(&existing);
      let _ = existing.show();
      let _ = existing.set_focus();
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
      let _ = window.set_position(position);
      exclude_from_capture(&window);
    }
  });
}

// 滚动模式不再注册任何全局快捷键：RegisterHotKey 会系统级吞键
// （滚动期间所有应用无法输入空格/回车）。操作一律走悬浮控制条按钮。

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
  app
    .state::<ScreenshotState>()
    .hotkey
    .lock()
    .unwrap()
    .clone()
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
/// 同时把贴图窗口临时排除出捕获，防止贴图入镜污染帧（会话结束还原）。
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
  set_pins_excluded(app, true);
}

/// 停止滚动自动捕获看守线程（幂等：无会话时直接返回）。
///
/// 只置停止标志、绝不 join：tauri 启用 tracing feature 后，eval_script
/// （emit 的底层）会阻塞等待主线程执行脚本；若本函数在主线程被调用
/// （同步命令 screenshot_scroll_cancel、RunEvent::ExitRequested），
/// join 会与「看守线程正阻塞等主线程」形成循环等待 → AppHang。
/// 会话串行化由 scroll 互斥锁保证：看守线程下一轮发现会话已被取走
/// （ptr_eq 失败）即自行退出，无需 join。
fn stop_scroll_watcher(app: &AppHandle) {
  let state = app.state::<ScreenshotState>();
  let stop = state
    .scroll
    .lock()
    .unwrap()
    .as_ref()
    .map(|s| s.stop.clone());
  if let Some(stop) = stop {
    stop.store(true, Ordering::Release);
  }
}

/// 全量拆除：覆盖窗口 + 悬浮条 + 滚动会话（含看守线程）+ 临时帧，并还原贴图捕获。
/// 滚动完成/取消、录屏结束与应用退出共用。
pub fn teardown_all(app: &AppHandle) {
  stop_scroll_watcher(app);
  close_toolbar(app);
  let state = app.state::<ScreenshotState>();
  let scroll = state.scroll.lock().unwrap().take();
  drop(scroll);
  let session = state.session.lock().unwrap().take();
  if let Some(session) = session {
    teardown(app, session);
  }
  set_pins_excluded(app, false);
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

/// 拼接比较忽略左右各 1/25 列（滚动条、边缘抖动不参与匹配）。
const STITCH_EDGE_DIV: u32 = 25;

/// 行内比较带（列 x0..x1）的字节范围。
fn stitch_band(width: u32) -> (usize, usize) {
  let stride = (width * 4) as usize;
  let margin = ((width / STITCH_EDGE_DIV).max(1) as usize) * 4;
  (margin, stride.saturating_sub(margin))
}

/// 每行对比较带做的指纹序列。
fn band_row_hashes(img: &RgbaImage) -> Vec<u64> {
  let (x0, x1) = stitch_band(img.width());
  let stride = (img.width() * 4) as usize;
  let raw = img.as_raw();
  raw
    .chunks_exact(stride)
    .map(|row| row_hash(&row[x0..x1]))
    .collect()
}

/// 取某行比较带的字节切片。
fn band_row(img: &RgbaImage, y: usize) -> &[u8] {
  let (x0, x1) = stitch_band(img.width());
  let stride = (img.width() * 4) as usize;
  let raw = img.as_raw();
  &raw[y * stride + x0..y * stride + x1]
}

/// 行是否为纯色（各采样点通道值相差 ≤2）。纯色行没有辨识度，不能当锚点，
/// 否则白底页面的空白行会到处匹配，产生错误重叠。
fn is_uniform_span(span: &[u8]) -> bool {
  let px = span.len() / 4;
  if px == 0 {
    return true;
  }
  let step = (px / 16).max(1);
  let first = &span[0..3];
  let mut i = 0;
  while i < px {
    let o = i * 4;
    if (span[o] as i16 - first[0] as i16).abs() > 2
      || (span[o + 1] as i16 - first[1] as i16).abs() > 2
      || (span[o + 2] as i16 - first[2] as i16).abs() > 2
    {
      return false;
    }
    i += step;
  }
  true
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

/// 在累积图 a 中定位新一屏 b 的追加点：返回 Some(行号) 表示 b 该行之前的内容
/// 都已在 a 中；Some(b.height()) 表示整屏重复（未滚动）；None 表示没有可靠匹配。
///
/// 锚点法：取 a 底部若干「特征行」（非纯色、与上一行指纹不同）在 b 中定位，
/// 从锚点向下必须整段对齐（a 的尾部都要出现在 b 中），向上连续扩展直到失配
/// （固定吸顶栏在扩展中自然被截断，不参与匹配），连续验证行数达标才算数。
/// 精确指纹优先，±2 容差兜底（抗锯齿噪声）。
pub(crate) fn find_append_point(a: &RgbaImage, b: &RgbaImage) -> Option<u32> {
  if a.width() != b.width() || a.width() < 16 || a.height() == 0 || b.height() == 0 {
    return None;
  }
  let ha = band_row_hashes(a);
  let hb = band_row_hashes(b);
  let h_a = a.height() as usize;
  let h_b = b.height() as usize;
  // 最小重叠行数：不少于 12 行且约一屏的 1/8，再小视为「滚太快」交给回滚恢复
  let min_overlap = 12usize.max(h_b / 8).min(h_a).min(h_b);

  // 锚点：自底向上取最多 3 个特征行
  let mut anchors: Vec<usize> = Vec::new();
  for i in (0..h_a).rev() {
    if anchors.len() >= 3 {
      break;
    }
    let above_differs = i == 0 || ha[i] != ha[i - 1];
    if above_differs && !is_uniform_span(band_row(a, i)) {
      anchors.push(i);
    }
  }
  if anchors.is_empty() {
    return None;
  }

  // 验证 a 锚点行 i ↔ b 行 r 的对齐：向下整段核对（a 尾部必须在 b 中），
  // 向上连续匹配到失配为止；连续核对行数 ≥ min_overlap 才接受。
  let verify = |i: usize, r: usize, fuzzy: bool| -> Option<usize> {
    let tail = h_a - i;
    let end_b = r + tail;
    if end_b > h_b {
      return None;
    }
    let rows_equal = |ya: usize, yb: usize| -> bool {
      if fuzzy {
        rows_fuzzy_equal(band_row(a, ya), band_row(b, yb), 2)
      } else {
        ha[ya] == hb[yb]
      }
    };
    // 向下：a 的 [i, h_a) 必须与 b 的 [r, end_b) 完全对齐
    for j in 0..tail {
      if !rows_equal(i + j, r + j) {
        return None;
      }
    }
    // 向上：连续扩展，首次失配即停（吸顶栏/动画内容在这里被截断）
    let mut up = 0usize;
    while up < i.min(r) && rows_equal(i - up - 1, r - up - 1) {
      up += 1;
    }
    if tail + up >= min_overlap {
      Some(end_b)
    } else {
      None
    }
  };

  for fuzzy in [false, true] {
    let mut best: Option<usize> = None;
    for &i in &anchors {
      let anchor_span = band_row(a, i);
      for r in 0..h_b {
        let hit = if fuzzy {
          rows_fuzzy_equal(anchor_span, band_row(b, r), 2)
        } else {
          hb[r] == ha[i]
        };
        if !hit {
          continue;
        }
        if let Some(end_b) = verify(i, r, fuzzy) {
          // 多个通过验证的对齐取最小追加点（最大重叠），宁可重复不可丢内容
          best = Some(best.map_or(end_b, |cur| cur.min(end_b)));
        }
      }
    }
    if let Some(end_b) = best {
      return Some(end_b as u32);
    }
  }
  None
}

/// 垂直拼接全部帧（单测 / 批量合并用）：相邻帧按追加点去重；
/// 无重叠时整屏直连并记录警告码，绝不静默丢帧。
/// 生产路径走 scroll_append_to_session 的增量拼接，此函数由单测锁定语义。
#[cfg(test)]
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
    match find_append_point(&out, b) {
      Some(append_from) if append_from < b.height() => {
        let tail_height = b.height() - append_from;
        let mut merged = RgbaImage::new(width, out.height() + tail_height);
        image::imageops::overlay(&mut merged, &out, 0, 0);
        let tail = image::imageops::crop_imm(b, 0, append_from, width, tail_height).to_image();
        image::imageops::overlay(&mut merged, &tail, 0, out.height() as i64);
        out = merged;
      }
      // 整屏重复（未滚动）：跳过
      Some(_) => {}
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

/// 读取某个显示器的冻结帧信息。
///
/// 必须是 async：同步命令在**主线程**执行，而覆盖窗/悬浮条在挂载时会立刻调用本命令——
/// 此刻主线程正卡在 `builder.build()` 里等这个窗口创建完成，命令排队等主线程 →
/// 双方互等，进程挂死（0.9.0 实测点「截长图」即 AppHang）。
#[tauri::command]
pub async fn screenshot_frame_info(
  app: AppHandle,
  monitor_index: usize,
) -> Result<MonitorFrameInfo, String> {
  let state = app.state::<ScreenshotState>();
  let guard = state.session.lock().unwrap();
  let frame = session_monitor(&guard, monitor_index)?;
  let session = guard.as_ref().unwrap();
  Ok(frame.info(session.cursor, session.focus_index))
}

/// 只有完成输出才结束会话；取消文件对话框或写入失败可继续使用同一选区。
#[tauri::command]
pub async fn screenshot_finish(
  app: AppHandle,
  window: tauri::WebviewWindow,
  output: CaptureOutput,
) -> Result<Option<ScreenshotOutcome>, String> {
  let CaptureOutput {
    monitor_index,
    x,
    y,
    width,
    height,
    action,
    annotated_png,
    save_title,
  } = output;
  if !matches!(action.as_str(), "copy" | "save" | "chat" | "ocr" | "pin") {
    return Err(format!("未知输出动作: {}", action));
  }
  let state = app.state::<ScreenshotState>();
  let _operation = state
    .capture_operation
    .try_lock()
    .map_err(|_| "截图操作进行中")?;
  let _output = state
    .capture_output
    .try_lock()
    .map_err(|_| "截图操作进行中")?;
  let frame = session_monitor(&state.session.lock().unwrap(), monitor_index)?.clone();
  if frame.label != window.label() || width == 0 || height == 0 {
    return Err("无效的截图选区".into());
  }

  let (cx, cy, cw, ch) = clamp_crop_rect(frame.width, frame.height, x, y, width, height);
  let frame_path = frame.frame_path.clone();
  let (cropped, png_bytes) = tauri::async_runtime::spawn_blocking(move || {
    let cropped = if let Some(data_url) = annotated_png {
      decode_annotated_png(&data_url, cw, ch)?
    } else {
      let image = image::open(&frame_path).map_err(|e| format!("读取冻结帧失败: {}", e))?;
      image.crop_imm(cx, cy, cw, ch).into_rgba8()
    };
    let mut png = Vec::new();
    cropped
      .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
      .map_err(|e| format!("PNG 编码失败: {}", e))?;
    Ok::<_, String>((cropped, png))
  })
  .await
  .map_err(|e| format!("裁剪任务异常: {}", e))??;

  let filename = timestamped_filename("png");
  let export_path = if action == "save" {
    use tauri_plugin_dialog::DialogExt;
    let dialog = app
      .dialog()
      .file()
      .set_parent(&window)
      .add_filter("PNG", &["png"])
      .set_title(save_title.unwrap_or_else(|| "Save screenshot".into()))
      .set_file_name(&filename);
    let (tx, rx) = tokio::sync::oneshot::channel();
    dialog.save_file(move |path| {
      let _ = tx.send(path);
    });
    let Some(path) = rx.await.map_err(|_| "无法打开保存对话框")? else {
      return Ok(None);
    };
    Some(path.into_path().map_err(|e| e.to_string())?)
  } else {
    None
  };

  let gallery_app = app.clone();
  let (gallery_path, saved_path) = tauri::async_runtime::spawn_blocking(move || {
    let gallery_dir = screenshots_dir(&gallery_app);
    if let Some(path) = export_path {
      fs::write(&path, &png_bytes).map_err(|e| format!("保存截图失败: {}", e))?;
      // 另存到外部时仍保留一份归档；归档失败不影响已成功写入的文件。
      let library = gallery_dir.ok().and_then(|dir| {
        if path.parent().and_then(|parent| parent.canonicalize().ok()) == dir.canonicalize().ok() {
          return Some(path.clone());
        }
        let archived = dir.join(filename);
        fs::write(&archived, &png_bytes).ok().map(|()| archived)
      });
      Ok((library, path))
    } else {
      let path = gallery_dir?.join(filename);
      fs::write(&path, &png_bytes).map_err(|e| format!("保存截图失败: {}", e))?;
      Ok::<_, String>((Some(path.clone()), path))
    }
  })
  .await
  .map_err(|e| format!("保存任务异常: {}", e))??;

  let path = saved_path.to_string_lossy().into_owned();
  if let Some(gallery_path) = gallery_path {
    let _ = app.emit_to("main", "screenshot-saved", gallery_path.to_string_lossy());
  }

  match action.as_str() {
    "copy" => {
      use tauri_plugin_clipboard_manager::ClipboardExt;
      let image = tauri::image::Image::new(cropped.as_raw(), cw, ch);
      app
        .clipboard()
        .write_image(&image)
        .map_err(|e| format!("写入剪贴板失败: {}", e))?;
    }
    "chat" | "ocr" => {
      let event = if action == "chat" {
        "screenshot-captured"
      } else {
        "screenshot-ocr"
      };
      app
        .emit_to("main", event, &path)
        .map_err(|e| format!("通知主窗口失败: {}", e))?;
    }
    "pin" => spawn_pin_window(&app, &path, Some(pin_position(&frame, cx, cy))).await?,
    _ => {}
  }
  teardown_all(&app);
  Ok(Some(ScreenshotOutcome {
    action,
    saved_path: Some(path),
    warnings: Vec::new(),
  }))
}

#[tauri::command]
pub fn screenshot_cancel(app: AppHandle) -> Result<(), String> {
  let state = app.state::<ScreenshotState>();
  let _output = state
    .capture_output
    .try_lock()
    .map_err(|_| "截图操作进行中")?;
  if crate::api::screenshot_record::recording_snapshot(&app).is_some() {
    return Err("请使用录屏控制条结束录制".into());
  }
  // 前端在黑屏或解码失败时也需要取消，即使窗口刚创建完成。
  teardown_all(&app);
  Ok(())
}

// ---- 滚动截长图命令 ----

/// 进入滚动截取模式：固定选区、捕获第一屏、保留取景框并启动自动捕获看守线程。
/// 用户直接滚动页面，画面变化时自动捕获并增量拼接（PixPin 手动滚动模式）。
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
  let _operation = state
    .capture_operation
    .try_lock()
    .map_err(|_| "截图操作进行中")?;
  let _output = state
    .capture_output
    .try_lock()
    .map_err(|_| "截图操作进行中")?;
  let geometry = session_monitor_geometry(&state, monitor_index)?;
  let (mx, my, frame_w, frame_h) = geometry;
  let (rx, ry, rw, rh) = clamp_crop_rect(frame_w, frame_h, x, y, width, height);

  if state.scroll.lock().unwrap().is_some() {
    return Err("已有滚动截取进行中".to_string());
  }

  // 第一屏直接取自冻结帧：用户框选时看到的内容，且不会把覆盖窗自身的
  // 边框/控件捕获进画面（此刻覆盖窗尚未隐藏）。
  let frozen = session_monitor(&state.session.lock().unwrap(), monitor_index)?.clone();
  let frame = tauri::async_runtime::spawn_blocking(move || -> Result<image::RgbaImage, String> {
    let img = image::open(&frozen.frame_path).map_err(|e| format!("读取冻结帧失败: {}", e))?;
    drop(frozen); // 阻塞读图完成前保留临时文件所有权，包括调用任务被取消的情况。
    Ok(image::imageops::crop_imm(&img, rx, ry, rw, rh).to_image())
  })
  .await
  .map_err(|e| format!("捕获任务异常: {}", e))??;

  let session = ScrollSession {
    monitor_index,
    monitor_origin: (mx, my),
    x: rx,
    y: ry,
    width: rw,
    height: rh,
    stitched: frame,
    pending: None,
    frame_count: 1,
    match_ok: true,
    stop: Arc::new(AtomicBool::new(false)),
  };
  let status = scroll_status_of(&session);
  let stop = session.stop.clone();
  *state.scroll.lock().unwrap() = Some(session);

  // 保留选区框（点击穿透 + 排除捕获），让用户看得见截取范围并自由滚动页面
  keep_selection_overlay(&app, monitor_index);

  // 先返回，让覆盖窗的 await 立刻拿到结果。建悬浮条窗口会阻塞主线程，
  // 若与「命令响应的脚本投递」同时发生，两边互等 → 进程挂死（AppHang）。
  // 所以「建窗 + 启动看守线程」整体推到后台：窗口建好之后才开始 emit，
  // 避免看守线程的 emit 撞上正在创建的 webview。
  let deferred_app = app.clone();
  let region = (rx, ry, rw, rh);
  std::thread::spawn(move || {
    std::thread::sleep(Duration::from_millis(350));
    if stop.load(Ordering::Acquire) {
      return;
    }
    show_toolbar(&deferred_app, geometry, region);
    std::thread::sleep(Duration::from_millis(350));
    scroll_watcher(deferred_app, stop);
  });
  Ok(status)
}

/// 看守线程：轮询选区画面，与上次捕获的降采样基准做差分；
/// 变化超阈值且距上次追加 ≥250ms 时捕获整帧并增量拼接，状态广播给悬浮条与取景框。
/// 每一步都核对 `Arc::ptr_eq(&session.stop, stop)`：本线程可能比会话活得久
/// （detached 不 join），teardown 后新开的滚动会话绝不能被旧线程写入。
fn scroll_watcher(app: AppHandle, stop: Arc<AtomicBool>) {
  const POLL: Duration = Duration::from_millis(150);
  const MIN_GAP: Duration = Duration::from_millis(250);
  const DIFF_THRESHOLD: f32 = 3.0;
  let mut base: Option<RgbaImage> = None;
  // 起始时允许立即捕获第一次变化（基准为空时直接捕获）
  let mut last_append = Instant::now()
    .checked_sub(MIN_GAP)
    .unwrap_or_else(Instant::now);

  while !stop.load(Ordering::Acquire) {
    std::thread::sleep(POLL);
    if stop.load(Ordering::Acquire) {
      break;
    }
    if last_append.elapsed() < MIN_GAP {
      continue;
    }
    let params = {
      let state = app.state::<ScreenshotState>();
      let guard = state.scroll.lock().unwrap();
      let params = guard
        .as_ref()
        .filter(|s| Arc::ptr_eq(&s.stop, &stop))
        .map(|s| {
          (
            s.monitor_index,
            s.monitor_origin,
            s.x,
            s.y,
            s.width,
            s.height,
          )
        });
      params
    };
    let Some((monitor_index, origin, x, y, w, h)) = params else {
      break;
    };
    let frame = match capture_live_region_at(monitor_index, origin, x, y, w, h) {
      Ok(frame) => frame,
      Err(_) => continue, // 偶发捕获失败：等下一轮轮询
    };
    let small = downsample(&frame, 8);
    if let Some(b) = &base {
      if mean_abs_diff(&small, b) < DIFF_THRESHOLD {
        continue; // 画面没滚动，跳过
      }
    }
    base = Some(small);
    last_append = Instant::now();

    let status = {
      let state = app.state::<ScreenshotState>();
      let mut guard = state.scroll.lock().unwrap();
      match guard.as_mut() {
        Some(session) if Arc::ptr_eq(&session.stop, &stop) => {
          scroll_append_to_session(session, frame);
          scroll_status_of(session)
        }
        _ => break,
      }
    };
    let _ = app.emit_to(TOOLBAR_LABEL, "scroll-frame", &status);
    let _ = app.emit_to(
      format!("{}{}", OVERLAY_LABEL_PREFIX, monitor_index),
      "scroll-frame",
      &status,
    );
  }
}

/// 累积长图高度上限（px）：超过后停止追加，防止超长滚动把内存吃爆
const MAX_STITCH_HEIGHT: u32 = 30_000;

/// 把一帧追加进滚动会话：匹配成功则增量拼接；失败（滚太快/含固定元素）
/// 挂起为 pending，等用户回滚后重新匹配时丢弃（回滚路径会重新捕获）。
fn scroll_append_to_session(session: &mut ScrollSession, frame: RgbaImage) {
  if session.stitched.height() >= MAX_STITCH_HEIGHT {
    // 已达上限：不再追加（防内存失控），会话保持可用，完成后照常入库
    session.match_ok = true;
    return;
  }
  match find_append_point(&session.stitched, &frame) {
    Some(append_from) if append_from < frame.height() => {
      session.pending = None;
      session.match_ok = true;
      let width = session.stitched.width();
      let tail_height = frame.height() - append_from;
      let mut merged = RgbaImage::new(width, session.stitched.height() + tail_height);
      image::imageops::overlay(&mut merged, &session.stitched, 0, 0);
      let tail = image::imageops::crop_imm(&frame, 0, append_from, width, tail_height).to_image();
      image::imageops::overlay(&mut merged, &tail, 0, session.stitched.height() as i64);
      session.stitched = merged;
      session.frame_count += 1;
    }
    Some(_) => {
      // 整屏重复（未滚动）：不计帧
      session.match_ok = true;
    }
    None => {
      session.pending = Some(frame);
      session.match_ok = false;
      session.frame_count += 1;
    }
  }
}

/// 手动补拍一屏：自动捕获漏拍（动画/惯性滚动）时的兜底入口。
#[tauri::command]
pub async fn screenshot_scroll_capture(app: AppHandle) -> Result<ScrollStatus, String> {
  let (monitor_index, origin, x, y, w, h) = {
    let state = app.state::<ScreenshotState>();
    let guard = state.scroll.lock().unwrap();
    let session = guard
      .as_ref()
      .ok_or_else(|| "没有正在进行的滚动截取".to_string())?;
    (
      session.monitor_index,
      session.monitor_origin,
      session.x,
      session.y,
      session.width,
      session.height,
    )
  };

  let frame = tauri::async_runtime::spawn_blocking(move || {
    capture_live_region_at(monitor_index, origin, x, y, w, h)
  })
  .await
  .map_err(|e| format!("捕获任务异常: {}", e))??;

  let status = {
    let state = app.state::<ScreenshotState>();
    let mut guard = state.scroll.lock().unwrap();
    let session = guard
      .as_mut()
      .ok_or_else(|| "没有正在进行的滚动截取".to_string())?;
    scroll_append_to_session(session, frame);
    scroll_status_of(session)
  };
  let _ = app.emit_to(TOOLBAR_LABEL, "scroll-frame", &status);
  let _ = app.emit_to(
    format!("{}{}", OVERLAY_LABEL_PREFIX, monitor_index),
    "scroll-frame",
    &status,
  );
  Ok(status)
}

/// 完成滚动截取：停看守线程，提交未匹配帧（直连+警告），长图入库。
#[tauri::command]
pub async fn screenshot_scroll_finish(app: AppHandle) -> Result<ScreenshotOutcome, String> {
  let state = app.state::<ScreenshotState>();
  let _operation = state
    .capture_operation
    .try_lock()
    .map_err(|_| "截图操作进行中")?;
  let _output = state
    .capture_output
    .try_lock()
    .map_err(|_| "截图操作进行中")?;
  stop_scroll_watcher(&app);
  let mut session = state
    .scroll
    .lock()
    .unwrap()
    .take()
    .ok_or_else(|| "没有正在进行的滚动截取".to_string())?;

  // 匹配失败残留的帧：不静默丢弃，直连到末尾并给出警告
  let mut warnings = Vec::new();
  if let Some(pending) = session.pending.take() {
    let width = session.stitched.width();
    let mut merged = RgbaImage::new(width, session.stitched.height() + pending.height());
    image::imageops::overlay(&mut merged, &session.stitched, 0, 0);
    image::imageops::overlay(&mut merged, &pending, 0, session.stitched.height() as i64);
    session.stitched = merged;
    warnings.push("stitch_fallback".to_string());
  }

  let stitched = session.stitched;
  let png_bytes = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<u8>, String> {
    let mut png = Vec::new();
    stitched
      .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
      .map_err(|e| format!("PNG 编码失败: {}", e))?;
    Ok(png)
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

/// 取消滚动截取：丢弃已拼内容，不产出文件。
#[tauri::command]
pub fn screenshot_scroll_cancel(app: AppHandle) -> Result<(), String> {
  let state = app.state::<ScreenshotState>();
  let _output = state
    .capture_output
    .try_lock()
    .map_err(|_| "截图操作进行中")?;
  teardown_all(&app);
  Ok(())
}

/// 悬浮控制条挂载时查询当前模式与进度；无活动会话时返回错误以便其自关闭。
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolbarInfo {
  /// scroll | record
  pub mode: String,
  pub frame_count: u32,
  /// 录屏最长时长（毫秒），滚动模式为 0
  pub max_duration_ms: u64,
  /// 录制开始时刻（Unix 毫秒，供悬浮条对表计时），滚动模式为 0
  pub started_at_ms: u64,
  /// 录制当前是否处于暂停态
  #[serde(default)]
  pub paused: bool,
}

/// 悬浮控制条挂载时读取会话状态。
///
/// 必须是 async：同 `screenshot_frame_info`，同步命令排在主线程上，
/// 而窗口正在主线程的 `build()` 中创建 → 死锁。
#[tauri::command]
pub async fn screenshot_toolbar_info(app: AppHandle) -> Result<ToolbarInfo, String> {
  let state = app.state::<ScreenshotState>();
  if let Some(scroll) = state.scroll.lock().unwrap().as_ref() {
    return Ok(ToolbarInfo {
      mode: "scroll".into(),
      frame_count: scroll.frame_count,
      max_duration_ms: 0,
      started_at_ms: 0,
      paused: false,
    });
  }
  if let Some((max_ms, started_ms, paused)) =
    crate::api::screenshot_record::recording_snapshot(&app)
  {
    return Ok(ToolbarInfo {
      mode: "record".into(),
      frame_count: 0,
      max_duration_ms: max_ms,
      started_at_ms: started_ms,
      paused,
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

/// 弹出系统文件选择器，把选中的图片导入截图库（统一转存为 PNG）。
/// 用户取消时返回空字符串。
#[tauri::command]
pub async fn screenshot_import(app: AppHandle) -> Result<String, String> {
  let dialog_app = app.clone();
  let picked = tauri::async_runtime::spawn_blocking(move || -> Option<std::path::PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    dialog_app
      .dialog()
      .file()
      .add_filter(
        "图片 / Image",
        &["png", "jpg", "jpeg", "webp", "gif", "bmp"],
      )
      .blocking_pick_file()
      .and_then(|p| p.as_path().map(|path| path.to_path_buf()))
  })
  .await
  .map_err(|e| format!("选择任务异常: {}", e))?;

  let Some(picked) = picked else {
    return Ok(String::new());
  };

  let png_bytes = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<u8>, String> {
    let mut reader =
      image::ImageReader::open(&picked).map_err(|e| format!("读取图片失败: {}", e))?;
    reader = reader.with_guessed_format().map_err(|e| e.to_string())?;
    reader.limits(image::Limits::default());
    let img = reader
      .decode()
      .map_err(|e| format!("图片格式无效: {}", e))?
      .to_rgba8();
    let mut png = Vec::new();
    img
      .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
      .map_err(|e| format!("PNG 编码失败: {}", e))?;
    Ok(png)
  })
  .await
  .map_err(|e| format!("导入任务异常: {}", e))??;

  let path = screenshots_dir(&app)?.join(format!("Import_{}", timestamped_filename("png")));
  fs::write(&path, &png_bytes).map_err(|e| format!("保存导入图片失败: {}", e))?;
  let path_str = path.to_string_lossy().into_owned();
  let _ = app.emit_to("main", "screenshot-saved", &path_str);
  Ok(path_str)
}

/// 把截图库中的图片复制到系统剪贴板（快速预览「复制图片」用）。
#[tauri::command]
pub async fn screenshot_copy_file(app: AppHandle, path: String) -> Result<(), String> {
  let img = tauri::async_runtime::spawn_blocking(move || -> Result<RgbaImage, String> {
    let mut reader = image::ImageReader::open(&path).map_err(|e| format!("读取图片失败: {}", e))?;
    reader = reader.with_guessed_format().map_err(|e| e.to_string())?;
    reader.limits(image::Limits::default());
    reader
      .decode()
      .map(|dyn_img| dyn_img.to_rgba8())
      .map_err(|e| format!("解码图片失败: {}", e))
  })
  .await
  .map_err(|e| format!("读取任务异常: {}", e))??;
  use tauri_plugin_clipboard_manager::ClipboardExt;
  let (width, height) = (img.width(), img.height());
  let raw = img.into_raw();
  let image = tauri::image::Image::new(&raw, width, height);
  app
    .clipboard()
    .write_image(&image)
    .map_err(|e| format!("写入剪贴板失败: {}", e))
}

/// 把一张图片贴到屏幕上：置顶无边框悬浮窗展示，可拖动 / 滚轮缩放 / Esc 收起。
#[tauri::command]
pub async fn screenshot_pin_file(app: AppHandle, path: String) -> Result<(), String> {
  let canon = fs::canonicalize(&path).map_err(|e| format!("文件不存在: {}", e))?;
  if canon.is_dir() {
    return Err("目标是目录".into());
  }
  spawn_pin_window(&app, &canon.to_string_lossy(), None).await
}

/// 贴图窗口挂载时领取自己的图片路径（由 spawn_pin_window 预先登记）。
#[tauri::command]
pub fn screenshot_pin_take(app: AppHandle, window: tauri::WebviewWindow) -> Result<String, String> {
  let state = app.state::<ScreenshotState>();
  let path = state.pin_paths.lock().unwrap().remove(window.label());
  path.ok_or_else(|| "没有待贴图的图片".to_string())
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
    clamp_crop_rect, decode_annotated_png, find_append_point, stitch_frames, MonitorFrame,
    MonitorFrameInfo, ScreenshotItem, ScreenshotOutcome, WindowRect,
  };
  use image::{Rgba, RgbaImage};
  use std::sync::Arc;

  #[test]
  fn annotated_png_must_match_selection_dimensions() {
    use base64::Engine;
    let image = RgbaImage::from_pixel(4, 3, Rgba([255, 50, 0, 255]));
    let mut png = Vec::new();
    image
      .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
      .unwrap();
    let url = format!(
      "data:image/png;base64,{}",
      base64::engine::general_purpose::STANDARD.encode(png)
    );
    assert_eq!(decode_annotated_png(&url, 4, 3).unwrap(), image);
    assert!(decode_annotated_png(&url, 5, 3).is_err());
    assert!(decode_annotated_png(&url, 3, 3).is_err());
    assert!(decode_annotated_png("data:image/jpeg;base64,AAAA", 4, 3).is_err());
    assert!(decode_annotated_png("data:image/png;base64,broken", 4, 3).is_err());
  }

  #[test]
  fn frozen_frame_lives_until_last_operation_reference_is_dropped() {
    let file = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    let path = file.to_path_buf();
    let frame = MonitorFrame {
      index: 0,
      label: "screenshot-overlay-0".into(),
      frame_path: path.clone(),
      _frame_file: Arc::new(file),
      bounds: WindowRect {
        x: -1440,
        y: 0,
        width: 1440,
        height: 900,
      },
      width: 2880,
      height: 1800,
      scale: 2.0,
      window_targets: vec![WindowRect {
        x: 10,
        y: 20,
        width: 100,
        height: 80,
      }],
    };
    let output_frame = frame.clone();
    let info = frame.info(Some((-1425.0, 25.0)), 0);
    assert_eq!(
      info.window_targets,
      vec![WindowRect {
        x: 20,
        y: 40,
        width: 200,
        height: 160
      }]
    );
    assert_eq!(info.cursor, Some((30.0, 50.0)));
    drop(frame);
    assert!(path.exists());
    drop(output_frame);
    assert!(!path.exists());
  }

  /// 行列共同决定颜色的合成帧：像素 RGB = (x/8 + y + offset)。
  /// 列方向必须有差异——整行纯色的行会被锚点选择拒绝。
  fn frame(rows: u32, offset: u32) -> RgbaImage {
    RgbaImage::from_fn(64, rows, |x, y| {
      let v = (x / 8 + y + offset) as u8;
      Rgba([v, v, v, 255])
    })
  }

  /// 带固定吸顶栏的合成帧：前 header_h 行永不变化，其下内容随 offset 滚动。
  fn frame_with_header(rows: u32, offset: u32, header_h: u32) -> RgbaImage {
    RgbaImage::from_fn(64, rows, |x, y| {
      let v = if y < header_h {
        (200 + x / 4) as u8
      } else {
        (x / 8 + (y - header_h) + offset) as u8
      };
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
      window_targets: Vec::new(),
      cursor: None,
      focused: true,
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
  fn finds_append_point_for_normal_scroll() {
    let a = frame(30, 0);
    // 向下滚 10 行：b 前 20 行已在 a 中 → 追加点 20
    let b = frame(30, 10);
    assert_eq!(find_append_point(&a, &b), Some(20));
    // 未滚动：整屏重复（追加点 = b 高度）
    assert_eq!(find_append_point(&a, &a), Some(30));
    // 滚动超过一屏（剩余重叠 4 行 < 最小 12 行）→ 无可靠匹配
    let c = frame(30, 26);
    assert_eq!(find_append_point(&a, &c), None);
  }

  #[test]
  fn handles_sticky_header_frames() {
    // 固定吸顶栏：内容对齐不受头部不变条带影响
    let a = frame_with_header(30, 0, 5);
    let b = frame_with_header(30, 10, 5);
    assert_eq!(find_append_point(&a, &b), Some(20));
    let (img, warnings) = stitch_frames(&[a, b]).unwrap();
    assert_eq!(img.height(), 40);
    assert!(warnings.is_empty());
  }

  #[test]
  fn uniform_rows_do_not_create_false_anchors() {
    // 白底 + 单行内容：纯色行不允许成为锚点（旧行哈希匹配会拼出错乱长图）
    let mut a = RgbaImage::from_pixel(64, 40, Rgba([255, 255, 255, 255]));
    for x in 0..64 {
      a.put_pixel(x, 30, Rgba([10, 20, 30, 255]));
    }
    let mut b = RgbaImage::from_pixel(64, 40, Rgba([255, 255, 255, 255]));
    for x in 0..64 {
      b.put_pixel(x, 32, Rgba([10, 20, 30, 255]));
    }
    assert_eq!(find_append_point(&a, &b), None);
  }

  #[test]
  fn white_background_document_scroll_matches() {
    // 白底文档：空白行多但文本行足以锚定（每条文本行颜色随行号变化，无周期性）
    fn doc(offset: u32) -> RgbaImage {
      RgbaImage::from_fn(64, 40, |x, y| {
        let pos = y + offset;
        let line = pos / 4;
        let on = pos % 4 < 2 && x % 8 < 5;
        let v = if on && line % 3 != 0 {
          (40 + line * 7) as u8
        } else {
          255
        };
        Rgba([v, v, v, 255])
      })
    }
    let a = doc(0);
    let b = doc(10);
    assert_eq!(find_append_point(&a, &b), Some(30));
  }

  #[test]
  fn detects_fuzzy_overlap_with_noise() {
    // 行距 4 的陡梯度 + ±2 抗锯齿噪声：正确对齐能容差匹配；
    // 相邻错位（整体差 4）超过容差，不会通过——恒定小梯度的合成帧
    // 会让 ±2 容差把错位也判成相同，真实内容不存在这种病态。
    fn steep(rows: u32, offset: u32) -> RgbaImage {
      RgbaImage::from_fn(64, rows, |x, y| {
        let v = (x / 8 + y * 4 + offset) as u8;
        Rgba([v, v, v, 255])
      })
    }
    let a = steep(30, 0);
    let mut b = steep(30, 40); // 向下滚 10 行（10 × 4）
    for (i, pixel) in b.pixels_mut().enumerate() {
      let jitter = if i % 3 == 0 { 2 } else { 0 };
      pixel[0] = pixel[0].saturating_add(jitter);
    }
    assert_eq!(find_append_point(&a, &b), Some(20));
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
