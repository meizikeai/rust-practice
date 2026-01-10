// src/utils/connect.rs
use crate::{
  config,
  model::domain::{StateCache, StateDB, StateDbPool},
};
use deadpool_redis::{Config, Pool, PoolConfig, Runtime, Timeouts};
use sqlx::{MySql, mysql::MySqlPoolOptions};
use std::time::Duration;

pub struct Connect;

impl Connect {
  pub async fn create_db_pool(database_url: &str) -> sqlx::Pool<MySql> {
    if database_url.is_empty() {
      panic!("Database URL is empty");
    }

    MySqlPoolOptions::new()
      .min_connections(100)
      .max_connections(200)
      .connect(database_url)
      .await
      .expect(format!("Failed to connect {}", database_url).as_str())
  }

  pub async fn create_redis_pool(redis_url: &str) -> Pool {
    if redis_url.is_empty() {
      panic!("Redis URL is empty");
    }

    let mut cfg = Config::from_url(redis_url);

    cfg.pool = Some(PoolConfig {
      max_size: 40,
      timeouts: Timeouts {
        wait: Some(Duration::from_secs(1)),
        create: Some(Duration::from_secs(1)),
        recycle: Some(Duration::from_secs(1)),
      },
      ..Default::default()
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1)).expect("Failed to connect Redis pool");

    // test the connection
    pool.get().await.expect(format!("Failed to connect {}", redis_url).as_str());

    pool
  }

  pub async fn new(config: config::Config) -> (StateDB, StateCache) {
    let relation_master = Self::create_db_pool(&config.db.relation.master).await;
    let relation_slave = Self::create_db_pool(&config.db.relation.slave).await;
    let profile = Self::create_redis_pool(&config.cache.profile).await;

    (
      StateDB {
        relation: StateDbPool {
          master: relation_master,
          slave: relation_slave,
        },
      },
      StateCache { profile },
    )
  }
}
