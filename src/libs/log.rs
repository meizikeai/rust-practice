// src/libs/log.rs
use std::fs;
use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer, filter::LevelFilter, fmt, prelude::*};

pub fn init<P: AsRef<Path>>(log_path: P) {
  let log_path = log_path.as_ref();

  if !log_path.exists() {
    if let Err(e) = fs::create_dir_all(log_path) {
      eprintln!("Failed to create log directory {}: {}", log_path.display(), e);
    }
  }

  let app_log = RollingFileAppender::new(Rotation::DAILY, log_path, "app.log");
  let error_log = RollingFileAppender::new(Rotation::DAILY, log_path, "error.log");

  let app_layer = fmt::layer().with_writer(app_log).with_ansi(false).with_target(false).with_filter(LevelFilter::INFO);
  let error_layer = fmt::layer().with_writer(error_log).with_ansi(false).with_target(false).with_filter(LevelFilter::ERROR);

  let console_layer = fmt::layer().with_ansi(true).with_target(false).with_filter(EnvFilter::from_default_env());

  tracing_subscriber::registry().with(app_layer).with(error_layer).with(console_layer).init();
}
