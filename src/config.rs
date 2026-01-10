// src/config.rs
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct Config {
  pub cache: RedisConfig,
  pub db: MysqlConfig,
  pub env: String,
  pub log_path: String,
  pub port: String,
}

#[derive(Deserialize, Clone)]
pub struct RedisConfig {
  pub profile: String,
}

#[derive(Deserialize, Clone)]
pub struct MysqlConfig {
  pub relation: DbConf,
}

#[derive(Deserialize, Clone)]
pub struct DbConf {
  pub master: String,
  pub slave: String,
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
#[allow(dead_code)]
pub struct ConfRedis {
  pub host: &'static str,
  pub password: &'static str,
}

impl Config {
  pub fn init() -> Self {
    let cache = Self::get_redis_config()["default"].clone();
    let db = Self::get_mysql_config()["default"].clone();

    let profile = Self::create_redis_uri(cache.host.to_string());
    let relation = Self::create_mysql_uri(
      db.username.to_string(),
      db.password.to_string(),
      db.master.to_string(),
      db.slave.to_string(),
      db.database.to_string(),
    );

    Self {
      cache: RedisConfig { profile },
      db: MysqlConfig { relation },
      env: Self::get_mode(),
      log_path: std::env::var("LOG_DIR").unwrap_or_else(|_| "/data/logs/rust-practice".into()),
      port: std::env::var("KS_PORT").unwrap_or_else(|_| "8887".into()),
    }
  }

  fn get_mode() -> String {
    match std::env::var("RP_MODE") {
      Ok(v) if v == "release" || v == "test" => v,
      _ => "test".to_string(),
    }
  }

  fn create_mysql_uri(user: String, password: String, master: String, slave: String, dbname: String) -> DbConf {
    if user.is_empty() || password.is_empty() || master.is_empty() || slave.is_empty() || dbname.is_empty() {
      return DbConf {
        master: String::new(),
        slave: String::new(),
      };
    }

    return DbConf {
      master: format!("mysql://{}:{}@{}/{}", user, password, master, dbname),
      slave: format!("mysql://{}:{}@{}/{}", user, password, slave, dbname),
    };
  }

  fn create_redis_uri(path: String) -> String {
    if path.is_empty() {
      return String::new();
    }

    format!("redis://{}", path)
  }

  #[allow(dead_code)]
  fn create_redis_uri_with_password(path: &str, password: &str) -> String {
    if path.is_empty() {
      return String::new();
    }

    match password {
      "" => format!("redis://{}", path),
      _ => format!("redis://:{}@{}", password, path),
    }
  }

  fn get_mysql_config() -> HashMap<String, ConfMySQL> {
    let mode = Self::get_mode();
    let key = format!("default-{}", mode);

    let mut result = HashMap::new();
    result.insert(
      "default".to_string(),
      match key.as_str() {
        "default-release" => ConfMySQL {
          master: "127.0.0.1",
          slave: "127.0.0.1",
          username: "test",
          password: "test@123",
          database: "test",
        },
        _ => ConfMySQL {
          master: "127.0.0.1",
          slave: "127.0.0.1",
          username: "test",
          password: "test@123",
          database: "test",
        },
      },
    );
    result
  }

  fn get_redis_config() -> HashMap<String, ConfRedis> {
    let mode = Self::get_mode();
    let key = format!("default-{}", mode);

    let mut result = HashMap::new();
    result.insert(
      "default".to_string(),
      match key.as_str() {
        "default-release" => ConfRedis {
          host: "127.0.0.1",
          password: "",
        },
        _ => ConfRedis {
          host: "127.0.0.1",
          password: "",
        },
      },
    );
    result
  }
}
