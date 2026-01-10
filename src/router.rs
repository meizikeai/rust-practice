// src/router.rs
use crate::{handler::common, model::domain::AppState, utils::prometheus};
use axum::{
  Router, middleware,
  routing::{get, post},
};
use std::sync::Arc;

pub async fn init(state: Arc<AppState>) -> Router {
  Router::new()
    .route("/", get(common::ok))
    .route("/metrics", get(prometheus::prometheus_handler))
    .route("/get/{uid}/something", get(common::get_something))
    .route("/del/{uid}/something", post(common::del_something))
    .layer(middleware::from_fn_with_state(state.clone(), prometheus::metrics_middleware))
    .fallback(common::not_found)
    .with_state(state)
}
