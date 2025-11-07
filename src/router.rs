// src/router.rs
use crate::controller::default;
use crate::libs::prometheus::{self, PromOpts};
use axum::{
  Router, middleware,
  routing::{get, post},
};
use deadpool_redis::Pool as RedisPool;
use default::{not_found, ok};
use sqlx::{MySql, Pool as MysqlPool};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
  pub env: String,
  pub db_master: MysqlPool<MySql>,
  pub db_slave: MysqlPool<MySql>,
  pub cache: RedisPool,
  pub prom_opts: Arc<PromOpts>,
}

pub async fn init(state: Arc<AppState>) -> Router {
  Router::new()
    .route("/", get(ok))
    .route("/metrics", get(prometheus::prometheus_handler))
    .route("/get/{uid}/something", get(default::get_something))
    .route("/del/{uid}/something", post(default::del_something))
    .layer(middleware::from_fn_with_state(state.clone(), prometheus::metrics_middleware))
    .fallback(not_found)
    .with_state(state)
}
