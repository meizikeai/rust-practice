// src/model/domain.rs
use crate::utils::prometheus::PromOpts;
use deadpool_redis::Pool as RedisPool;
use sqlx::{MySql, Pool as MysqlPool};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
  pub cache: StateCache,
  pub db: StateDB,
  pub env: String,
  pub prom: Arc<PromOpts>,
}

#[derive(Clone)]
pub struct StateDB {
  pub relation: StateDbPool,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct StateDbPool {
  pub master: MysqlPool<MySql>,
  pub slave: MysqlPool<MySql>,
}

#[derive(Clone)]
pub struct StateCache {
  pub profile: RedisPool,
}
