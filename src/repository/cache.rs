// src/repository/cache.rs
use crate::model::domain::AppState;
use deadpool_redis::redis::AsyncCommands;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub struct Cache {}

impl Cache {
  pub async fn get_test(state: &AppState, uid: u64) -> Result<Value, String> {
    let mut conn = state.cache.profile.get().await.map_err(|e| format!("Redis pool error: {}", e))?;

    let key = format!("u:{}:setting", uid);
    let data: HashMap<String, String> = conn.hgetall(&key).await.map_err(|e| format!("Redis hgetall error: {}", e))?;
    let result = hashmap_to_serde_map(data);
    // println!("get_setting -> {:?}", result);

    Ok(Value::Object(result))
  }

  pub async fn add_test(state: &AppState, uid: u64, data: Value) -> Result<Value, String> {
    let mut conn = state.cache.profile.get().await.map_err(|e| format!("Redis pool error: {}", e))?;

    let key = format!("u:{}:setting", uid);
    let obj = match data.as_object() {
      Some(map) => map,
      None => return Ok(Value::Null),
    };

    let mut args: Vec<(String, String)> = Vec::new();

    for (field, val) in obj {
      let val_str = match val {
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        _ => val.to_string(),
      };
      args.push((field.clone(), val_str));
    }

    let _: () = conn.hset_multiple(&key, &args).await.map_err(|e| format!("Redis hset error: {}", e))?;

    Ok(Value::Null)
  }
}

fn hashmap_to_serde_map<T>(input: HashMap<String, T>) -> Map<String, Value>
where
  T: Into<Value>,
{
  input.into_iter().map(|(k, v)| (k, v.into())).collect()
}
