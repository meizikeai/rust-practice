// src/libs/config.rs
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
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

#[derive(Clone)]
pub struct ConfMySQL {
  pub master: &'static str,
  pub slave: &'static str,
  pub username: &'static str,
  pub password: &'static str,
  pub database: &'static str,
}

#[derive(Clone)]
pub struct ConfRedis {
  pub master: &'static str,
}

const ENV_MODES: [&str; 2] = ["release", "test"];

static CACHED_MODE: Lazy<&'static str> = Lazy::new(|| {
  let mode = std::env::var("RP_MODE").unwrap_or_else(|_| "test".to_string());

  if ENV_MODES.contains(&mode.as_str()) {
    if mode == "release" { ENV_MODES[0] } else { ENV_MODES[1] }
  } else {
    ENV_MODES[1]
  }
});
static MYSQL_CONFIG_MAP: Lazy<HashMap<String, ConfMySQL>> = Lazy::new(|| {
  let mut result = HashMap::new();

  result.insert(
    "default-test".to_string(),
    ConfMySQL {
      master: "10.99.0.93",
      slave: "10.99.0.93",
      username: "blued",
      password: "g3bkshwqcj4wcSMr",
      database: "blued",
    },
  );
  result.insert(
    "default-release".to_string(),
    ConfMySQL {
      master: "127.0.0.1",
      slave: "127.0.0.1",
      username: "test",
      password: "test@123",
      database: "test",
    },
  );
  result
});
static REDIS_CONFIG_MAP: Lazy<HashMap<String, ConfRedis>> = Lazy::new(|| {
  let mut result = HashMap::new();

  result.insert("default-test".to_string(), ConfRedis { master: "10.99.0.6" });
  result.insert("default-release".to_string(), ConfRedis { master: "127.0.0.1" });

  result
});

impl AppConfig {
  pub fn init() -> Self {
    let cache = Self::get_redis_config()["default"].master.to_string();
    let db = Self::get_mysql_config()["default"].clone();

    Self {
      env: Self::get_mode().to_string(),
      port: env::var("KS_PORT").unwrap_or_else(|_| "8887".into()),
      path: env::var("LOG_PATH").unwrap_or_else(|_| "/data/logs/rust-practice".into()),
      db_master: Self::create_mysql_uri(db.username.to_string(), db.password.to_string(), db.master.to_string(), db.database.to_string()),
      db_slave: Self::create_mysql_uri(db.username.to_string(), db.password.to_string(), db.slave.to_string(), db.database.to_string()),
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

  fn get_key(k: &str) -> String {
    let mode = Self::get_mode();
    format!("{}-{}", k, mode)
  }

  fn get_mysql_config() -> HashMap<String, ConfMySQL> {
    let mysql_config = &*MYSQL_CONFIG_MAP;
    let mut result = HashMap::new();

    let data: &'static [&'static str] = &["default"];

    for v in data {
      let key = Self::get_key(v);

      if let Some(config) = mysql_config.get(&key) {
        result.insert(v.to_string(), config.clone());
      }
    }

    result
  }

  fn get_redis_config() -> HashMap<String, ConfRedis> {
    let redis_config = &*REDIS_CONFIG_MAP;
    let mut result = HashMap::new();

    let data: &'static [&'static str] = &["default"];

    for v in data {
      let key = Self::get_key(v);

      if let Some(config) = redis_config.get(&key) {
        result.insert(v.to_string(), config.clone());
      }
    }

    result
  }
}
