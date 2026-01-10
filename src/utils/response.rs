// src/utils/response.rs
use axum::{Json, http::StatusCode};
use serde::Serialize;
use serde_json::{Value, json};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Code {
  Ok,
  BadRequest,
  Unauthorized,
  Forbidden,
  NotFound,
  MethodNotAllowed,
  UnprocessableEntity,
  InternalServerError,

  // custom
  DbError,
}

#[allow(dead_code)]
impl Code {
  pub fn as_u32(&self) -> u32 {
    match self {
      Code::Ok => 200,
      Code::BadRequest => 400,
      Code::Unauthorized => 401,
      Code::Forbidden => 403,
      Code::NotFound => 404,
      Code::MethodNotAllowed => 405,
      Code::UnprocessableEntity => 422,
      Code::InternalServerError => 500,

      Code::DbError => 403_001,
    }
  }

  pub fn message(&self) -> &'static str {
    match self {
      Code::Ok => "OK",
      Code::BadRequest => "Bad Request",
      Code::Unauthorized => "Unauthorized",
      Code::Forbidden => "Forbidden",
      Code::NotFound => "Not Found",
      Code::MethodNotAllowed => "Method Not Allowed",
      Code::UnprocessableEntity => "Unprocessable Entity",
      Code::InternalServerError => "Internal Server Error",

      Code::DbError => "DB Error",
    }
  }

  pub fn http_status(&self) -> StatusCode {
    match self {
      Code::Ok => StatusCode::OK,
      Code::BadRequest => StatusCode::BAD_REQUEST,
      Code::Unauthorized => StatusCode::UNAUTHORIZED,
      Code::Forbidden => StatusCode::FORBIDDEN,
      Code::NotFound => StatusCode::NOT_FOUND,
      Code::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
      Code::UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY,
      Code::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,

      // 自定义错误映射到 4xx / 5xx
      Code::DbError => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
}

#[allow(dead_code)]
pub fn respond<T: Serialize>(code: Code, data: Option<T>) -> (StatusCode, Json<Value>) {
  let body = json!({
    "code": code.as_u32(),
    "message": code.message(),
    "data": data
  });

  (code.http_status(), Json(body))
}

#[allow(dead_code)]
pub fn success<T: Serialize>(data: T) -> (StatusCode, Json<Value>) {
  respond(Code::Ok, Some(data))
}

#[allow(dead_code)]
pub fn failure(code: Code) -> (StatusCode, Json<Value>) {
  respond::<Value>(code, None)
}
