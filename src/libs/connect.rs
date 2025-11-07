// src/libs/connect.rs
use deadpool_redis::{Config, Pool, Runtime};
use sqlx::{MySql, mysql::MySqlPoolOptions};

pub struct AppConnect;

impl AppConnect {
  pub async fn create_db_pool(database_url: &str) -> sqlx::Pool<MySql> {
    if database_url.is_empty() {
      panic!("Database URL is empty");
    }

    MySqlPoolOptions::new()
      .min_connections(10)
      .max_connections(20)
      .connect(database_url)
      .await
      .expect(format!("Failed to connect {}", database_url).as_str())
  }

  pub async fn create_redis_pool(redis_url: &str) -> Pool {
    if redis_url.is_empty() {
      panic!("Redis URL is empty");
    }

    let cfg = Config::from_url(redis_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).expect("Failed to connect Redis pool");

    // test the connection
    pool.get().await.expect(format!("Failed to connect {}", redis_url).as_str());

    pool
  }
}
