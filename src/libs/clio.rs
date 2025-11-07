// src/libs/clio.rs
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClioError {
  #[error("Command execution failed: {0}")]
  IoError(#[from] io::Error),
  #[error("Non-zero exit status (code={0}): {1}")]
  CommandFailed(i32, String),
  #[error("Invalid UTF-8 output")]
  InvalidUtf8,
}

pub struct ClioTool {
  command: String,              // such as "get_conf", "get_host", "get_allhost", "get_batch_keys"
  path: PathBuf,                // such as "/demo/conf"
  binary_path: Option<PathBuf>, // such as "qconf", default "clio-tool"
}

impl ClioTool {
  pub fn new(command: impl Into<String>, path: impl Into<PathBuf>) -> Self {
    Self {
      command: command.into(),
      path: path.into(),
      binary_path: None,
    }
  }

  #[allow(dead_code)]
  pub fn binary(mut self, path: impl Into<PathBuf>) -> Self {
    self.binary_path = Some(path.into());
    self
  }

  pub fn execute(&self) -> Result<Output, ClioError> {
    let output = Command::new(self.binary_path.as_deref().unwrap_or("clio-tool".as_ref()))
      .arg(&self.command)
      .arg(&self.path)
      .output()?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
      return Err(ClioError::CommandFailed(output.status.code().unwrap_or(-1), stderr));
    }

    // println!("{:?}", output);
    Ok(output)
  }

  pub fn get_output(&self) -> Result<String, ClioError> {
    let output = self.execute()?;
    // String::from_utf8(output.stderr).map_err(|_| ClioError::InvalidUtf8)
    let stdout = String::from_utf8(output.stdout).map_err(|_| ClioError::InvalidUtf8)?.trim_end().to_string();

    Ok(stdout)
  }

  #[allow(dead_code)]
  pub fn get_conf(path: impl Into<PathBuf>) -> Self {
    Self::new("get_conf", path)
  }

  #[allow(dead_code)]
  pub fn get_host(path: impl Into<PathBuf>) -> Self {
    Self::new("get_host", path)
  }

  #[allow(dead_code)]
  pub fn get_allhost(path: impl Into<PathBuf>) -> Self {
    Self::new("get_allhost", path)
  }

  #[allow(dead_code)]
  pub fn get_batch_keys(path: impl Into<PathBuf>) -> Self {
    Self::new("get_batch_keys", path)
  }

  #[allow(dead_code)]
  pub fn mget_conf(paths: &[&str]) -> Result<Vec<String>, ClioError> {
    paths.iter().map(|path| ClioTool::get_conf(*path).get_output()).collect::<Result<Vec<_>, _>>()
  }
}
