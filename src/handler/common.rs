// src/handler/common.rs
use crate::{
  model::domain::AppState,
  repository::{cache::Cache, db::Database},
  utils::response::{Code, failure, success},
};
use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
};
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn ok() -> String {
  "OK".to_string()
}

pub async fn not_found() -> impl IntoResponse {
  (StatusCode::NOT_FOUND, "Not Found")
}

// test
pub async fn get_something(State(state): State<Arc<AppState>>, Path(uid): Path<u64>) -> (StatusCode, Json<Value>) {
  if uid == 0 {
    return failure(Code::UnprocessableEntity);
  }

  let mut data = json!({});
  if state.env == "test" {
    data = Database::get_test(&state, uid).await.unwrap_or(json!({}))
  } else {
    data = Cache::get_test(&state, uid).await.unwrap_or(json!({}))
  };
  // println!("LocalCache -> {}", data);

  success(data)
}

pub async fn del_something(State(state): State<Arc<AppState>>, Path(uid): Path<u64>, Json(payload): Json<Value>) -> impl IntoResponse {
  if uid <= 0 {
    return failure(Code::UnprocessableEntity);
  }

  if let Value::Object(obj) = &payload {
    if obj.is_empty() {
      return failure(Code::UnprocessableEntity);
    }
  }

  if state.env == "test" {
    let _ = Database::add_test(&state, uid, payload).await;
  } else {
    let _ = Cache::add_test(&state, uid, payload).await;
  }

  success(json!({}))
}
