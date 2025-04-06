use std::io;
use time::macros::{format_description, offset};
use tracing_appender::rolling::daily;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, registry, EnvFilter, Layer};

#[allow(dead_code)]
pub struct LoggerGuard(tracing_appender::non_blocking::WorkerGuard);

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
  let file_appender = daily("logs", "app.log");
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

  LoggerGuard(guard)
}
