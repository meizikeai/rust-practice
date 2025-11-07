// src/repository/database.rs
use crate::router::AppState;
use chrono::Utc;
use serde_json::Map;
use serde_json::Value;
use sqlx::{Error, Row};

pub struct Database {}

impl Database {
  pub async fn get_test(state: &AppState, uid: u64) -> Result<Value, Error> {
    let mut pool = state.db_slave.acquire().await?;
    let row = sqlx::query("SELECT content FROM settings WHERE uid = ? LIMIT 1").bind(uid).fetch_one(&mut *pool).await?;

    Ok(row.get("content"))
  }

  pub async fn add_test(state: &AppState, uid: u64, fields: Value) -> Result<u64, Error> {
    let mut pool = state.db_master.acquire().await?;

    let mut change = vec![];
    let mut temporary: Map<String, Value> = Map::new();
    let now = Utc::now().timestamp() as u64;

    if let Some(obj) = fields.as_object() {
      for (key, val) in obj {
        temporary.insert(key.to_string(), Value::String(val.to_string()));

        change.push(format!("'$.{}'", key));
        change.push(val.to_string());
      }
    }
    // println!("{:?},{:?}", change, temporary);

    if change.len() == 0 {
      return Ok(0);
    }

    let content = serde_json::to_string(&temporary).unwrap_or_else(|_| "{}".to_string());
    let sql = format!(
      "INSERT INTO settings (uid, content, event_time) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE content = JSON_SET(content, {}), event_time = ?",
      change.join(", ")
    );
    println!("{},{},{},{}", sql, uid, content, now);

    let result = sqlx::query(&sql).bind(uid as i64).bind(content).bind(now).bind(now).execute(&mut *pool).await?;

    Ok(result.rows_affected())
  }
}
