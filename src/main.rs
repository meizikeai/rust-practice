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
use repository::Repository;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use utils::{connect::Connect, fetch::Fetch, log, prometheus};

#[tokio::main]
async fn main() {
    let cfg = Config::init();

    // Log
    let _ = log::init(cfg.log.clone());
    // Connect to database
    let (db, cache) = Connect::new(cfg.clone()).await;
    // Create state
    let state = Arc::new(AppState {
        env: cfg.env.clone(),
        fetch: Fetch::new(),
        prometheus: prometheus::new(),
        repository: Repository::new(cache, db),
    });
    println!("→ Starting application in the {} environment", cfg.env.clone());

    // Metrics record uptime
    prometheus::start_record_uptime();

    // The main thread starts the HTTP service
    let app = router::init(state).await;
    let addr: SocketAddr = format!("0.0.0.0:{}", cfg.port.clone()).parse().expect("Invalid server address");
    let listener = TcpListener::bind(addr).await.expect("Failed to bind server");
    println!("→ Application started successfully. Listening on http://{}", addr);

    serve(listener, app).await.expect("Service crashed");
}
