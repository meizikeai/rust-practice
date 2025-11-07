// src/libs/config_clio.rs
use crate::libs::clio::ClioTool;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
  pub env: String,
  pub port: String,
  pub path: String,

  pub db_master: String,
  pub db_slave: String,
  pub cache: String,
}

const REDIS_PATH: &str = "/blued/backend/umem/ur-relation-setting";
const MYSQL_MASTER: &str = "/blued/backend/cynosdb/relationships/master";
const MYSQL_SLAVE: &str = "/blued/backend/cynosdb/relationships/slave";
const MYSQL_USERNAME: &str = "/blued/backend/cynosdb/relationships/username";
const MYSQL_PASSWORD: &str = "/blued/backend/cynosdb/relationships/password";
const MYSQL_DATABASE: &str = "/blued/backend/cynosdb/relationships/database";

const ENV_MODES: [&str; 2] = ["release", "test"];

static CACHED_MODE: Lazy<&'static str> = Lazy::new(|| {
  let mode = std::env::var("RP_MODE").unwrap_or_else(|_| "test".to_string());

  if ENV_MODES.contains(&mode.as_str()) {
    if mode == "release" { ENV_MODES[0] } else { ENV_MODES[1] }
  } else {
    ENV_MODES[1]
  }
});

impl AppConfig {
  pub fn init() -> Self {
    let cache = Self::handle_get_host(REDIS_PATH);
    let mysql_master = Self::handle_get_host(MYSQL_MASTER);
    let mysql_slave = Self::handle_get_host(MYSQL_SLAVE);
    let mysql_other = Self::handle_mget_conf(&[MYSQL_USERNAME, MYSQL_PASSWORD, MYSQL_DATABASE]);

    Self {
      env: Self::get_mode().to_string(),
      port: env::var("KS_PORT").unwrap_or_else(|_| "8887".into()),
      path: env::var("LOG_PATH").unwrap_or_else(|_| "/data/logs/rust-practice".into()),
      db_master: Self::create_mysql_uri(mysql_other[0].clone(), mysql_other[1].clone(), mysql_master, mysql_other[2].clone()),
      db_slave: Self::create_mysql_uri(mysql_other[0].clone(), mysql_other[1].clone(), mysql_slave, mysql_other[2].clone()),
      cache: Self::create_redis_uri(cache),
    }
  }

  pub fn get_mode() -> &'static str {
    *CACHED_MODE
  }

  fn create_mysql_uri(user: String, password: String, host: String, dbname: String) -> String {
    if user.is_empty() || password.is_empty() || host.is_empty() || dbname.is_empty() {
      return "".to_string();
    }

    format!("mysql://{}:{}@{}/{}", user, password, host, dbname)
  }

  fn create_redis_uri(host: String) -> String {
    if host.is_empty() {
      return "".to_string();
    }

    format!("redis://{}", host)
  }

  // fn handle_get_conf(path: &str) -> String {
  //   match ClioTool::get_conf(path).get_output() {
  //     Ok(v) => v,
  //     Err(e) => {
  //       panic!("Failed to get '{}' config: {}", path, e);
  //     }
  //   }
  // }

  fn handle_mget_conf(paths: &[&str]) -> Vec<String> {
    match ClioTool::mget_conf(paths) {
      Ok(v) => v,
      Err(e) => {
        panic!("Failed to get {:?} config: {}", paths, e);
      }
    }
  }

  fn handle_get_host(path: &str) -> String {
    match ClioTool::get_host(path).get_output() {
      Ok(v) => v,
      Err(e) => {
        panic!("Failed to get '{}' config: {}", path, e);
      }
    }
  }
}
