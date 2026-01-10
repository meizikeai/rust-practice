// src/main.rs
mod config;
mod handler;
mod model;
mod repository;
mod router;
mod utils;

use axum::serve;
use config::Config;
use model::domain::AppState;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use utils::{connect::Connect, log, prometheus, prometheus::PromOpts};

#[tokio::main]
async fn main() {
  let cfg = Config::init();
  let cfg_clone = cfg.clone();

  // Log
  let _ = log::init(cfg.log_path);
  // Prometheus
  let prom: Arc<PromOpts> = prometheus::init_prometheus_opts();
  // Connect to database
  let (db, cache) = Connect::new(cfg_clone).await;
  // Create state
  let state = Arc::new(AppState {
    cache,
    db,
    env: cfg.env.clone(),
    prom,
  });
  println!("→ Starting application in the {} environment", cfg.env);

  // Metrics record uptime
  prometheus::start_record_uptime();

  // The main thread starts the HTTP service
  let app = router::init(state).await;
  let addr: SocketAddr = format!("0.0.0.0:{}", cfg.port).parse().expect("Invalid server address");
  let listener = TcpListener::bind(addr).await.expect("Failed to bind server");

  println!("→ Application started successfully. Listening on http://{}", addr);

  serve(listener, app).await.expect("Service crashed");
}
