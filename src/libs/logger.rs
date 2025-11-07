// src/libs/log.rs
use chrono::Local;
use flexi_logger::{DeferredNow, Logger, Record, writers::LogWriter};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct LevelBasedWriter {
  log_path: PathBuf,
  keep_files: usize,

  current_day: Mutex<String>,
  trace_file: Mutex<File>,
  debug_file: Mutex<File>,
  info_file: Mutex<File>,
  warn_file: Mutex<File>,
  error_file: Mutex<File>,
}

pub fn init(log_path: String) -> Result<flexi_logger::LoggerHandle, Box<dyn std::error::Error>> {
  let handle = Logger::try_with_str("trace")?
    .log_to_writer(Box::new(LevelBasedWriter::new(&log_path, 15))) // retain log files for 15 days
    .start()?;

  Ok(handle)
}

impl LevelBasedWriter {
  pub fn new(log_path: &str, keep_files: usize) -> Self {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let path = PathBuf::from(log_path);

    fs::create_dir_all(&path).unwrap();

    let trace_file = Self::open_file(&path, "trace", &today);
    let debug_file = Self::open_file(&path, "debug", &today);
    let info_file = Self::open_file(&path, "info", &today);
    let warn_file = Self::open_file(&path, "warn", &today);
    let error_file = Self::open_file(&path, "error", &today);

    Self {
      log_path: path,
      keep_files,
      current_day: Mutex::new(today),
      trace_file: Mutex::new(trace_file),
      debug_file: Mutex::new(debug_file),
      info_file: Mutex::new(info_file),
      warn_file: Mutex::new(warn_file),
      error_file: Mutex::new(error_file),
    }
  }

  fn open_file(base: &PathBuf, level: &str, date: &str) -> File {
    let filename = format!("{}_{}.log", level, date);
    OpenOptions::new().create(true).append(true).open(base.join(filename)).unwrap()
  }

  // clean up old logs and keep only N
  fn cleanup_old_files(&self, level: &str) {
    let mut files: Vec<_> = fs::read_dir(&self.log_path)
      .unwrap()
      .filter_map(|e| e.ok())
      .filter(|e| e.file_type().unwrap().is_file() && e.file_name().to_string_lossy().starts_with(level))
      .collect();

    files.sort_by_key(|e| e.metadata().unwrap().modified().unwrap());

    while files.len() > self.keep_files {
      let old = files.remove(0);
      let _ = fs::remove_file(old.path());
    }
  }

  fn rotate_if_needed(&self) {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut current_day = self.current_day.lock().unwrap();

    if *current_day != today {
      *current_day = today.clone();

      *self.trace_file.lock().unwrap() = Self::open_file(&self.log_path, "trace", &today);
      *self.debug_file.lock().unwrap() = Self::open_file(&self.log_path, "debug", &today);
      *self.info_file.lock().unwrap() = Self::open_file(&self.log_path, "info", &today);
      *self.warn_file.lock().unwrap() = Self::open_file(&self.log_path, "warn", &today);
      *self.error_file.lock().unwrap() = Self::open_file(&self.log_path, "error", &today);

      self.cleanup_old_files("trace");
      self.cleanup_old_files("debug");
      self.cleanup_old_files("info");
      self.cleanup_old_files("warn");
      self.cleanup_old_files("error");
    }
  }
}

impl LogWriter for LevelBasedWriter {
  fn write(&self, now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
    self.rotate_if_needed();

    let log_line = format!(
      "[{}] {} [{}:{}] -> {}\n",
      now.now().format("%Y-%m-%d %H:%M:%S"),
      record.level(),
      record.file().unwrap_or("unknown"),
      record.line().map(|l| l.to_string()).unwrap_or_else(|| "?".to_string()),
      // record.module_path().unwrap_or("unknown"),
      &record.args()
    );

    match record.level() {
      log::Level::Trace => self.trace_file.lock().unwrap().write_all(log_line.as_bytes()),
      log::Level::Debug => self.debug_file.lock().unwrap().write_all(log_line.as_bytes()),
      log::Level::Info => self.info_file.lock().unwrap().write_all(log_line.as_bytes()),
      log::Level::Warn => self.warn_file.lock().unwrap().write_all(log_line.as_bytes()),
      log::Level::Error => self.error_file.lock().unwrap().write_all(log_line.as_bytes()),
    }
  }

  fn flush(&self) -> std::io::Result<()> {
    self.trace_file.lock().unwrap().flush()?;
    self.debug_file.lock().unwrap().flush()?;
    self.info_file.lock().unwrap().flush()?;
    self.warn_file.lock().unwrap().flush()?;
    self.error_file.lock().unwrap().flush()?;
    Ok(())
  }
}
