// src/handler/common.rs
use crate::{
    model::domain::AppState,
    utils::response::{AppError, AppResult, Code, SafeJson, Success},
};
use axum::{
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
pub async fn get_something(State(state): State<Arc<AppState>>, Path(uid): Path<u64>) -> AppResult<Value> {
    if uid == 0 {
        return Err(AppError::Logic(Code::UnprocessableEntity));
    }

    let data = if state.env == "test" {
        state.repository.db.get_test(uid).await.unwrap_or(json!({}))
    } else {
        state.repository.cache.get_test(uid).await.unwrap_or(json!({}))
    };
    // println!("LocalCache -> {}", data);

    Ok(Success(data))
}

pub async fn set_something(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<u64>,
    SafeJson(payload): SafeJson<Value>,
) -> AppResult<Value> {
    if uid <= 0 {
        return Err(AppError::Logic(Code::UnprocessableEntity));
    }

    if let Value::Object(obj) = &payload {
        if obj.is_empty() {
            return Err(AppError::Logic(Code::UnprocessableEntity));
        }
    }

    if state.env == "test" {
        let _ = state.repository.db.add_test(uid, payload).await;
    } else {
        let _ = state.repository.cache.add_test(uid, payload).await;
    }

    Ok(Success::empty())
}
