// src/main.rs
mod controller;
mod libs;
mod repository;
mod router;

use axum::serve;
use libs::{config::AppConfig, connect::AppConnect, logger, prometheus, prometheus::PromOpts};
use router::AppState;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
  let config = AppConfig::init();
  let _ = logger::init(config.path).expect("Failed to initialize log");

  let db_master = AppConnect::create_db_pool(&config.db_master).await;
  let db_slave = AppConnect::create_db_pool(&config.db_slave).await;
  let cache = AppConnect::create_redis_pool(&config.cache).await;
  let prom_opts: Arc<PromOpts> = prometheus::init_prometheus_opts();

  let state = Arc::new(AppState {
    env: config.env.clone(),
    db_master,
    db_slave,
    cache,
    prom_opts,
  });
  println!("→ The current environment is {}", config.env);

  // Metrics record uptime
  prometheus::start_record_uptime();

  // The main thread starts the HTTP service
  let app = router::init(state).await;
  let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse().expect("Invalid server address");
  let listener = TcpListener::bind(addr).await.expect("Failed to bind server");

  println!("→ The TCP Server running on http://{}", addr);

  serve(listener, app).await.expect("Server crashed");
}
