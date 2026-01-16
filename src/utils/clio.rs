// src/utils/clio.rs
use std::path::PathBuf;
use std::process::{Command, Output};

pub struct ClioTool {
    command: String,              // such as "get_conf", "get_host", "get_allhost", "get_batch_keys"
    path: PathBuf,                // such as "/demo/conf"
    binary_path: Option<PathBuf>, // such as "qconf", default "clio-tool"
}

impl ClioTool {
    pub fn new(command: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self { command: command.into(), path: path.into(), binary_path: None }
    }

    #[allow(dead_code)]
    pub fn binary(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary_path = Some(path.into());
        self
    }

    pub fn execute(&self) -> Result<Output, Box<dyn std::error::Error>> {
        let output = Command::new(self.binary_path.as_deref().unwrap_or("clio-tool".as_ref()))
            .arg(&self.command)
            .arg(&self.path)
            .output()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("clio-tool failed (exit {code}): {stderr}"),
            )));
        }

        Ok(output)
    }

    pub fn get_output(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = self.execute()?;
        let stdout = String::from_utf8(output.stdout)?.trim_end().to_string();
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
    pub fn mget_conf(paths: &[&str]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        paths.iter().map(|path| ClioTool::get_conf(*path).get_output()).collect::<Result<Vec<_>, _>>()
    }
}
