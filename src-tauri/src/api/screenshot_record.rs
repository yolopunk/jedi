// 选区录屏：后台线程按 fps 捕获选区帧 → openh264 编码 H.264 → 纯 Rust mp4 封装
//
// 录制期间选区所在覆盖窗保持可见（点击穿透 + Windows 下排除出系统捕获），
// 悬浮控制条提供「停止」；达到最长时长自动停止；窗口关闭/应用退出时通过
// stop + join 干净收尾并产出文件，不泄漏线程。

use crate::api::screenshot::{
  clamp_crop_rect, keep_selection_overlay, screenshots_dir, session_monitor_geometry,
  show_toolbar, teardown_all, timestamped_filename, TOOLBAR_LABEL,
};
use serde::Serialize;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};

/// 录屏最长时长：到点自动停止并产出文件
const MAX_DURATION: Duration = Duration::from_secs(600);
const DEFAULT_FPS: u32 = 15;

pub struct RecordState {
  active: Mutex<Option<ActiveRecord>>,
}

struct ActiveRecord {
  stop: Arc<AtomicBool>,
  handle: Option<JoinHandle<()>>,
  result: Arc<Mutex<Option<Result<String, String>>>>,
}

impl Default for RecordState {
  fn default() -> Self {
    Self {
      active: Mutex::new(None),
    }
  }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordStartInfo {
  pub width: u32,
  pub height: u32,
  pub fps: u32,
  pub max_duration_ms: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordOutcome {
  pub path: String,
}

pub fn is_recording(app: &AppHandle) -> bool {
  app.state::<RecordState>().active.lock().unwrap().is_some()
}

/// 悬浮条查询录制会话时使用：录制中返回最长时长，否则 None。
pub fn recording_max_duration(app: &AppHandle) -> Option<u64> {
  if is_recording(app) {
    Some(MAX_DURATION.as_millis() as u64)
  } else {
    None
  }
}

/// H.264 YUV420 色度抽样要求宽高为偶数：向下取偶，最小 2。
pub(crate) fn evenize(v: u32) -> u32 {
  v.max(2) & !1
}

pub(crate) fn clamp_fps(v: Option<u32>) -> u32 {
  v.unwrap_or(DEFAULT_FPS).clamp(5, 30)
}

/// 屏幕内容的码率经验值：按像素量与帧率估算并夹紧。
pub(crate) fn bitrate_bps(w: u32, h: u32, fps: u32) -> u32 {
  (w * h * fps / 10).clamp(2_000_000, 12_000_000)
}

// ---- Annex-B → AVCC ----

/// 按 00 00 01 / 00 00 00 01 起始码切分 NAL 单元（保留 NAL 头字节）。
pub(crate) fn split_annexb(buf: &[u8]) -> Vec<&[u8]> {
  let mut starts: Vec<usize> = Vec::new();
  let mut i = 0usize;
  while i + 3 <= buf.len() {
    if buf[i] == 0 && buf[i + 1] == 0 && buf[i + 2] == 1 {
      starts.push(i + 3);
      i += 3;
    } else {
      i += 1;
    }
  }
  let mut nals = Vec::with_capacity(starts.len());
  for (idx, &start) in starts.iter().enumerate() {
    let mut end = if idx + 1 < starts.len() {
      starts[idx + 1] - 3
    } else {
      buf.len()
    };
    // 4 字节起始码的前导 0 与尾部对齐零不属于 NAL 内容
    while end > start && buf[end - 1] == 0 {
      end -= 1;
    }
    if end > start {
      nals.push(&buf[start..end]);
    }
  }
  nals
}

fn append_avcc(dst: &mut Vec<u8>, nal: &[u8]) {
  dst.extend_from_slice(&(nal.len() as u32).to_be_bytes());
  dst.extend_from_slice(nal);
}

/// 收集编码器输出：SPS/PPS 单独留存（进 avcC），VCL NAL 组装为 AVCC 样本。
pub(crate) struct H264Collector {
  sps: Vec<u8>,
  pps: Vec<u8>,
  samples: Vec<(Vec<u8>, bool)>,
}

impl H264Collector {
  pub fn new() -> Self {
    Self {
      sps: Vec::new(),
      pps: Vec::new(),
      samples: Vec::new(),
    }
  }

  pub fn push_packet(&mut self, packet: &[u8]) {
    let mut sample: Vec<u8> = Vec::new();
    let mut is_sync = false;
    for nal in split_annexb(packet) {
      if nal.is_empty() {
        continue;
      }
      let nal_type = nal[0] & 0x1F;
      match nal_type {
        7 => self.sps = nal.to_vec(),
        8 => self.pps = nal.to_vec(),
        5 => {
          is_sync = true;
          append_avcc(&mut sample, nal);
        }
        _ => append_avcc(&mut sample, nal),
      }
    }
    if !sample.is_empty() {
      self.samples.push((sample, is_sync));
    }
  }

  pub fn is_empty(&self) -> bool {
    self.samples.is_empty()
  }
}

/// 用 mp4 crate 封装：avc1 + avcC（真实 SPS/PPS），固定帧时长（timescale=1000，毫秒）。
fn write_mp4(
  path: &std::path::Path,
  width: u32,
  height: u32,
  collector: &H264Collector,
  frame_ms: u32,
) -> Result<(), String> {
  use mp4::{AvcConfig, MediaConfig, Mp4Config, Mp4Sample, Mp4Writer, TrackConfig, TrackType};
  use std::str::FromStr;
  if collector.sps.is_empty() || collector.pps.is_empty() {
    return Err("编码器未输出 SPS/PPS，无法封装".to_string());
  }
  if collector.is_empty() {
    return Err("没有可封装的视频帧".to_string());
  }
  let file = fs::File::create(path).map_err(|e| format!("创建视频文件失败: {}", e))?;
  let config = Mp4Config {
    major_brand: FromStr::from_str("isom").map_err(|e| format!("{}", e))?,
    minor_version: 512,
    compatible_brands: vec![
      FromStr::from_str("isom").map_err(|e| format!("{}", e))?,
      FromStr::from_str("iso2").map_err(|e| format!("{}", e))?,
      FromStr::from_str("avc1").map_err(|e| format!("{}", e))?,
      FromStr::from_str("mp41").map_err(|e| format!("{}", e))?,
    ],
    timescale: 1000,
  };
  let mut writer =
    Mp4Writer::write_start(file, &config).map_err(|e| format!("MP4 初始化失败: {}", e))?;
  writer
    .add_track(&TrackConfig {
      track_type: TrackType::Video,
      timescale: 1000,
      language: "und".to_string(),
      media_conf: MediaConfig::AvcConfig(AvcConfig {
        width: width as u16,
        height: height as u16,
        seq_param_set: collector.sps.clone(),
        pic_param_set: collector.pps.clone(),
      }),
    })
    .map_err(|e| format!("添加视频轨失败: {}", e))?;
  for (index, (bytes, is_sync)) in collector.samples.iter().enumerate() {
    writer
      .write_sample(
        1,
        &Mp4Sample {
          start_time: index as u64 * frame_ms as u64,
          duration: frame_ms,
          rendering_offset: 0,
          is_sync: *is_sync || index == 0,
          bytes: bytes::Bytes::copy_from_slice(bytes),
        },
      )
      .map_err(|e| format!("写入视频帧失败: {}", e))?;
  }
  writer
    .write_end()
    .map_err(|e| format!("MP4 收尾失败: {}", e))?;
  Ok(())
}

/// 录制线程主体：捕获 → 编码 → 封装，结果写回共享槽位。
fn record_thread(
  app: AppHandle,
  monitor_index: usize,
  x: u32,
  y: u32,
  width: u32,
  height: u32,
  fps: u32,
  stop: Arc<AtomicBool>,
  result: Arc<Mutex<Option<Result<String, String>>>>,
) {
  let outcome = (|| -> Result<String, String> {
    let monitors = xcap::Monitor::all().map_err(|e| format!("枚举显示器失败: {}", e))?;
    let monitor = monitors
      .into_iter()
      .nth(monitor_index)
      .ok_or_else(|| format!("显示器 {} 不存在", monitor_index))?;

    let config = openh264::encoder::EncoderConfig::new()
      .bitrate(openh264::encoder::BitRate::from_bps(bitrate_bps(
        width, height, fps,
      )))
      .max_frame_rate(openh264::encoder::FrameRate::from_hz(fps as f32))
      .usage_type(openh264::encoder::UsageType::ScreenContentRealTime)
      .intra_frame_period(openh264::encoder::IntraFramePeriod::from_num_frames(
        (fps * 2).max(1),
      ))
      .skip_frames(false);
    let mut encoder =
      openh264::encoder::Encoder::with_api_config(openh264::OpenH264API::from_source(), config)
        .map_err(|e| format!("初始化编码器失败: {}", e))?;

    let mut collector = H264Collector::new();
    let frame_ms = (1000 / fps).max(1);
    let frame_dur = Duration::from_millis(frame_ms as u64);
    let start = Instant::now();
    let mut next_tick = start;
    let mut out_w = width;
    let mut out_h = height;

    loop {
      if stop.load(Ordering::Acquire) || start.elapsed() >= MAX_DURATION {
        break;
      }
      let frame = monitor
        .capture_image()
        .map_err(|e| format!("捕获画面失败: {}", e))?;
      let (cx, cy, cw, ch) = clamp_crop_rect(
        frame.width(),
        frame.height(),
        x as i32,
        y as i32,
        width,
        height,
      );
      let region = image::imageops::crop_imm(&frame, cx, cy, cw, ch).to_image();
      out_w = evenize(region.width());
      out_h = evenize(region.height());
      let region = if out_w != region.width() || out_h != region.height() {
        image::imageops::crop_imm(&region, 0, 0, out_w, out_h).to_image()
      } else {
        region
      };
      let yuv =
        openh264::formats::YUVBuffer::from_rgb8_source(openh264::formats::RgbaSliceU8::new(
          region.as_raw(),
          (region.width() as usize, region.height() as usize),
        ));
      let packet = encoder
        .encode(&yuv)
        .map_err(|e| format!("编码失败: {}", e))?
        .to_vec();
      collector.push_packet(&packet);

      next_tick += frame_dur;
      let now = Instant::now();
      if next_tick > now {
        std::thread::sleep(next_tick - now);
      } else {
        // 捕获落后于节拍时重置基准，避免无限追赶
        next_tick = now;
      }
    }

    let dir = screenshots_dir(&app)?;
    let path = dir.join(format!("Record_{}", timestamped_filename("mp4")));
    write_mp4(&path, out_w, out_h, &collector, frame_ms)?;
    Ok(path.to_string_lossy().into_owned())
  })();

  let manual_stop = stop.load(Ordering::Acquire);
  match &outcome {
    Ok(path) => {
      let _ = app.emit_to(
        TOOLBAR_LABEL,
        "record-finished",
        &serde_json::json!({ "ok": true, "path": path }),
      );
      // 自动到时停止：没有 stop 命令兜底收尾，由线程负责通知与拆除
      if !manual_stop {
        let _ = app.emit_to("main", "screenshot-saved", path);
        teardown_all(&app);
      }
    }
    Err(message) => {
      let _ = app.emit_to(
        TOOLBAR_LABEL,
        "record-finished",
        &serde_json::json!({ "ok": false, "message": message }),
      );
      let _ = app.emit_to(
        "main",
        "screenshot-notice",
        &vec!["record_failed".to_string()],
      );
      if !manual_stop {
        teardown_all(&app);
      }
    }
  }
  *result.lock().unwrap() = Some(outcome);
}

// ---- Tauri commands ----

/// 开始选区录制：准备覆盖窗与悬浮条，启动后台捕获编码线程。
#[tauri::command]
pub async fn start_region_record(
  app: AppHandle,
  state: State<'_, RecordState>,
  monitor_index: usize,
  x: i32,
  y: i32,
  width: u32,
  height: u32,
  fps: Option<u32>,
) -> Result<RecordStartInfo, String> {
  let shot_state = app.state::<crate::api::screenshot::ScreenshotState>();
  let geometry = session_monitor_geometry(&shot_state, monitor_index)?;
  let (mx, my, mw, mh) = geometry;
  drop(shot_state);
  let (rx, ry, rw, rh) = clamp_crop_rect(mw, mh, x, y, width, height);
  let w = evenize(rw);
  let h = evenize(rh);
  let fps = clamp_fps(fps);

  let mut active = state.active.lock().unwrap();
  if active.is_some() {
    return Err("已有录制进行中".to_string());
  }

  keep_selection_overlay(&app, monitor_index);
  show_toolbar(&app, (mx, my, mw, mh), (rx, ry, w, h));

  let stop = Arc::new(AtomicBool::new(false));
  let result: Arc<Mutex<Option<Result<String, String>>>> = Arc::new(Mutex::new(None));
  let thread_app = app.clone();
  let thread_stop = stop.clone();
  let thread_result = result.clone();
  let handle = std::thread::spawn(move || {
    record_thread(
      thread_app,
      monitor_index,
      rx,
      ry,
      w,
      h,
      fps,
      thread_stop,
      thread_result,
    )
  });
  *active = Some(ActiveRecord {
    stop,
    handle: Some(handle),
    result,
  });
  Ok(RecordStartInfo {
    width: w,
    height: h,
    fps,
    max_duration_ms: MAX_DURATION.as_millis() as u64,
  })
}

/// 停止录制并等待封装完成，返回 MP4 路径。
#[tauri::command]
pub async fn stop_region_record(
  app: AppHandle,
  state: State<'_, RecordState>,
) -> Result<RecordOutcome, String> {
  let record = state
    .active
    .lock()
    .unwrap()
    .take()
    .ok_or_else(|| "没有正在进行的录制".to_string())?;
  let app_for_close = app.clone();
  tauri::async_runtime::spawn_blocking(move || -> Result<RecordOutcome, String> {
    record.stop.store(true, Ordering::Release);
    if let Some(handle) = record.handle {
      let _ = handle.join();
    }
    let outcome = record
      .result
      .lock()
      .unwrap()
      .take()
      .unwrap_or_else(|| Err("录制线程未产出结果".to_string()));
    teardown_all(&app_for_close);
    if let Ok(path) = &outcome {
      let _ = app_for_close.emit_to("main", "screenshot-saved", path);
    }
    outcome.map(|path| RecordOutcome { path })
  })
  .await
  .map_err(|e| format!("停止任务异常: {}", e))?
}

/// 应用退出前收尾：停止线程并产出文件（ExitRequested 调用）。
pub fn shutdown(app: &AppHandle) {
  let state = app.state::<RecordState>();
  let taken = state.active.lock().unwrap().take();
  if let Some(record) = taken {
    record.stop.store(true, Ordering::Release);
    if let Some(handle) = record.handle {
      let _ = handle.join();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{bitrate_bps, clamp_fps, evenize, split_annexb, H264Collector};

  #[test]
  fn evenize_rounds_down_to_even_minimum_two() {
    assert_eq!(evenize(101), 100);
    assert_eq!(evenize(100), 100);
    assert_eq!(evenize(1), 2);
    assert_eq!(evenize(0), 2);
  }

  #[test]
  fn fps_defaults_and_clamps() {
    assert_eq!(clamp_fps(None), 15);
    assert_eq!(clamp_fps(Some(1)), 5);
    assert_eq!(clamp_fps(Some(120)), 30);
    assert_eq!(clamp_fps(Some(24)), 24);
  }

  #[test]
  fn bitrate_stays_in_bounds() {
    assert_eq!(bitrate_bps(64, 64, 5), 2_000_000);
    assert_eq!(bitrate_bps(7680, 4320, 60), 12_000_000);
    assert!(bitrate_bps(1920, 1080, 15) > 2_000_000);
  }

  #[test]
  fn splits_three_and_four_byte_start_codes() {
    let buf = [
      0x00, 0x00, 0x00, 0x01, 0x67, 0xAA, // SPS（4 字节起始码）
      0x00, 0x00, 0x01, 0x68, 0xBB, // PPS（3 字节起始码）
      0x00, 0x00, 0x01, 0x65, 0xCC, 0x00, // IDR，尾部对齐零
    ];
    let nals = split_annexb(&buf);
    assert_eq!(nals.len(), 3);
    assert_eq!(nals[0], &[0x67, 0xAA]);
    assert_eq!(nals[1], &[0x68, 0xBB]);
    assert_eq!(nals[2], &[0x65, 0xCC]);
  }

  #[test]
  fn collector_extracts_params_and_builds_avcc() {
    let mut collector = H264Collector::new();
    let sps_packet = [
      0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80,
      0x00, 0x00, 0x01, 0x65, 0x88, 0x84, 0x00, 0x11, 0xFF,
    ];
    collector.push_packet(&sps_packet);
    assert_eq!(collector.sps, vec![0x67, 0x42, 0x00, 0x1E]);
    assert_eq!(collector.pps, vec![0x68, 0xCE, 0x3C, 0x80]);
    let (sample, is_sync) = &collector.samples[0];
    assert!(*is_sync);
    // IDR NAL（6 字节）+ 长度前缀（4 字节）
    assert_eq!(sample.len(), 4 + 6);
    assert_eq!(&sample[0..4], &[0, 0, 0, 6]);
    assert_eq!(&sample[4..], &[0x65, 0x88, 0x84, 0x00, 0x11, 0xFF]);
    assert!(!collector.is_empty());
  }

  /// 离线全链路：openh264 编码 5 帧 → mp4 封装 → Mp4Reader 回读校验结构。
  /// 不触碰屏幕捕获，可在任意环境跑。
  #[test]
  fn encodes_and_muxes_parseable_mp4() {
    use openh264::encoder::{BitRate, Encoder, EncoderConfig, FrameRate, UsageType};
    use openh264::formats::{RgbaSliceU8, YUVBuffer};

    let mut encoder = openh264::encoder::Encoder::with_api_config(
      openh264::OpenH264API::from_source(),
      EncoderConfig::new()
        .bitrate(BitRate::from_bps(2_000_000))
        .max_frame_rate(FrameRate::from_hz(15.0))
        .usage_type(UsageType::ScreenContentRealTime),
    )
    .unwrap();
    let mut collector = H264Collector::new();
    for i in 0..5u8 {
      let img = image::RgbaImage::from_fn(32, 32, |x, _| image::Rgba([x as u8, i * 10, 0x80, 255]));
      let yuv = YUVBuffer::from_rgb8_source(RgbaSliceU8::new(img.as_raw(), (32, 32)));
      let packet = encoder.encode(&yuv).unwrap().to_vec();
      collector.push_packet(&packet);
    }
    assert!(!collector.sps.is_empty(), "SPS 应随首帧输出");
    assert!(!collector.pps.is_empty(), "PPS 应随首帧输出");

    let dir = std::env::temp_dir().join(format!("jedi_record_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("out.mp4");
    super::write_mp4(&path, 32, 32, &collector, 66).unwrap();

    let file = std::fs::File::open(&path).unwrap();
    let size = std::fs::metadata(&path).unwrap().len();
    let reader = mp4::Mp4Reader::read_header(file, size);
    assert!(reader.is_ok(), "生成的 MP4 应可解析: {:?}", reader.err());
    let reader = reader.unwrap();
    assert_eq!(reader.tracks().len(), 1);
    assert_eq!(reader.sample_count(1).unwrap(), 5);
    std::fs::remove_file(&path).ok();
  }
}
