use std::fs;
use std::io;
use std::path::PathBuf;
use time::macros::{format_description, offset};
use tracing::info;
use tracing_appender::rolling::daily;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, registry, EnvFilter, Layer};

#[allow(dead_code)]
pub struct LoggerGuard(tracing_appender::non_blocking::WorkerGuard);

/// 日志目录必须用绝对路径：相对路径 "logs" 跟随进程 CWD，安装版启动时落进
/// 安装目录，从别的目录拉起时又会散落各处。日志初始化时还拿不到 Tauri
/// AppHandle，这里按 tauri.conf.json 的 identifier (io.jedi) 用同一套平台
/// 规则推导数据目录，推导失败再退回相对 "logs"。
fn resolve_log_dir() -> PathBuf {
  if let Some(data_dir) = dirs::data_dir() {
    let dir = data_dir.join("io.jedi").join("logs");
    if fs::create_dir_all(&dir).is_ok() {
      return dir;
    }
  }
  PathBuf::from("logs")
}

pub fn init() -> LoggerGuard {
  let env_filter =
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("jedi=info,error"));

  let time_fmt =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]");
  let timer = OffsetTime::new(offset!(+8), time_fmt);

  // 设置日志格式
  let format = fmt::format()
    .with_level(true)
    .with_target(true)
    .with_timer(timer);

  // 输出到控制台中
  let console_layer = if cfg!(debug_assertions) {
    Some(
      fmt::layer()
        .event_format(format.clone())
        .with_writer(io::stdout)
        .boxed(),
    )
  } else {
    None
  };

  // 输出到文件中
  let log_dir = resolve_log_dir();
  let file_appender = daily(&log_dir, "app.log");
  let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);

  let file_layer = fmt::layer()
    .with_ansi(false)
    .event_format(format.clone())
    .with_writer(non_blocking_appender)
    .boxed();

  // 注册
  registry()
    .with(env_filter)
    .with(console_layer)
    .with(file_layer)
    .with(ErrorLayer::default())
    .init();

  // boot 行是"进程活着"的最小证据：文件日志一建立就落一条，
  // 避免"目录里有个 0 字节日志、不知道进程到底起没起来"的排查困境
  info!(
    "jedi boot v{}, log_dir={}",
    env!("CARGO_PKG_VERSION"),
    log_dir.display()
  );

  LoggerGuard(guard)
}
