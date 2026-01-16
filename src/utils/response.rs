// src/utils/response.rs
use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Query, Request},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::{Value, json};
use std::borrow::Cow;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Code {
    Ok = 200,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    UnprocessableEntity = 422,
    InternalServerError = 500,

    DbError = 403001,
}

impl Code {
    pub fn as_u32(&self) -> u32 {
        *self as u32
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
            // Map error to 4xx / 5xx
            Code::DbError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize)]
struct ResponseBody<T> {
    code: u32,
    message: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

pub struct Success<T>(pub T);

#[allow(dead_code)]
impl Success<Value> {
    pub fn empty() -> Self {
        Self(json!({}))
    }
    pub fn null() -> Self {
        Self(Value::Null)
    }
}

impl<T: Serialize> IntoResponse for Success<T> {
    fn into_response(self) -> Response {
        let body =
            ResponseBody { code: Code::Ok.as_u32(), message: Cow::Borrowed(Code::Ok.message()), data: Some(self.0) };
        (StatusCode::OK, Json(json!(body))).into_response()
    }
}

#[allow(dead_code)]
pub enum AppError {
    Logic(Code),
    Custom(Code, String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, msg) = match self {
            AppError::Logic(c) => (c, Cow::Borrowed(c.message())),
            AppError::Custom(c, s) => (c, Cow::Owned(s)),
        };
        let body: ResponseBody<Value> = ResponseBody { code: code.as_u32(), message: msg, data: None };
        (code.http_status(), Json(json!(body))).into_response()
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            tracing::error!("External API Timeout: {}", e);
            return AppError::Logic(Code::InternalServerError);
        }

        if e.is_connect() {
            tracing::error!("External API Connection Error: {}", e);
            return AppError::Logic(Code::InternalServerError);
        }

        if let Some(status) = e.status() {
            tracing::error!("External API Status Error: {} - {}", status, e);
        }

        tracing::error!("HTTP Client Unknown Error: {}", e);
        AppError::Logic(Code::InternalServerError)
    }
}

pub type AppResult<T> = Result<Success<T>, AppError>;

#[allow(dead_code)]
pub struct SafeJson<T>(pub T);

impl<S, T> FromRequest<S> for SafeJson<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(SafeJson(value)),
            Err(e) => {
                tracing::warn!("JSON parsing failed -> {}", e.body_text());
                Err(AppError::Logic(Code::UnprocessableEntity))
            }
        }
    }
}

#[allow(dead_code)]
pub struct SafeQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for SafeQuery<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(value)) => Ok(SafeQuery(value)),
            Err(e) => {
                tracing::warn!("Query parsing failed -> {}", e.body_text());
                Err(AppError::Logic(Code::UnprocessableEntity))
            }
        }
    }
}
