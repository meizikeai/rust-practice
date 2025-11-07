// src/controller/default.rs
use crate::{
  repository::{mysql::Database, redis::Cache},
  router::AppState,
};
use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
};
use serde_json::{Value, json};
use std::sync::Arc;

fn api_response<T: serde::Serialize>(code: u32, data: T) -> (StatusCode, Json<Value>) {
  let status = StatusCode::from_u16(code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
  (
    status,
    Json(json!({
      "code": code,
      "message": status.canonical_reason().unwrap_or("Unknown"),
      "data": data
    })),
  )
}

pub async fn ok() -> String {
  "OK".to_string()
}

pub async fn not_found() -> impl IntoResponse {
  (StatusCode::NOT_FOUND, "Not Found")
}

// test
pub async fn get_something(State(state): State<Arc<AppState>>, Path(uid): Path<u64>) -> (StatusCode, Json<Value>) {
  let mut data = json!({});

  if uid == 0 {
    return api_response(422, data);
  }

  if state.env == "test" {
    data = Database::get_test(&state, uid).await.unwrap_or(json!({}))
  } else {
    data = Cache::get_test(&state, uid).await.unwrap_or(json!({}))
  };
  // println!("LocalCache -> {}", data);

  api_response(200, data)
}

pub async fn del_something(State(state): State<Arc<AppState>>, Path(uid): Path<u64>, Json(payload): Json<Value>) -> impl IntoResponse {
  if uid <= 0 {
    return api_response(422, json!({}));
  }

  if let Value::Object(obj) = &payload {
    if obj.is_empty() {
      return api_response(422, json!({}));
    }
  }

  if state.env == "test" {
    let _ = Database::add_test(&state, uid, payload).await;
  } else {
    let _ = Cache::add_test(&state, uid, payload).await;
  }

  api_response(200, json!({}))
}
