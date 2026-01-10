// src/utils/prometheus.rs
use crate::model::domain::AppState;
use axum::{
  body::Body,
  extract::{MatchedPath, Request, State},
  http::{StatusCode, header},
  middleware::Next,
  response::{IntoResponse, Response},
};
use lazy_static::lazy_static;
use prometheus::{
  Encoder, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, Opts, TextEncoder, register_histogram_vec, register_int_counter,
  register_int_counter_vec,
};
use regex::Regex;
use std::{sync::Arc, time::Instant};
use tokio::time::{Duration, interval};

const NAMESPACE: &str = "service";

lazy_static! {
  // uptime
  static ref UPTIME: IntCounter =
    register_int_counter!(Opts::new("uptime", "HTTP service uptime.").namespace(NAMESPACE)).unwrap();

  // 请求计数
  static ref HTTP_REQ_COUNT: IntCounterVec =
    register_int_counter_vec!(Opts::new("http_request_count_total", "Total number of HTTP requests.")
      .namespace(NAMESPACE),
      &["status", "endpoint", "method"]).unwrap();

  // 请求耗时
  static ref HTTP_REQ_DURATION: HistogramVec =
    register_histogram_vec!(HistogramOpts::new("http_request_duration_seconds", "HTTP request latencies in seconds.")
      .namespace(NAMESPACE),
      &["status", "endpoint", "method"]).unwrap();

  // 请求大小
  static ref HTTP_REQ_SIZE: HistogramVec =
    register_histogram_vec!(HistogramOpts::new("http_request_size_bytes", "HTTP request sizes in bytes")
      .namespace(NAMESPACE)
      .buckets(vec![100.0, 500.0, 1_000.0, 5_000.0, 10_000.0, 50_000.0]),
      &["status", "endpoint", "method"]).unwrap();

  // 响应大小
  static ref HTTP_RESP_SIZE: HistogramVec =
    register_histogram_vec!(HistogramOpts::new("http_response_size_bytes", "HTTP response sizes in bytes")
      .namespace(NAMESPACE)
      .buckets(vec![100.0, 500.0, 1_000.0, 5_000.0, 10_000.0, 50_000.0]),
      &["status", "endpoint", "method"]).unwrap();

  // 定义自监控指标
  static ref PROM_SENSORS_REQUESTS: IntCounterVec =
    register_int_counter_vec!(Opts::new("promhttp_metric_handler_requests_total", "Total number of scrapes by HTTP status code."), &["code"]).unwrap();
}

#[derive(Clone)]
pub struct PromOpts {
  pub exclude_regex_status: Option<Regex>,
  pub exclude_regex_endpoint: Option<Regex>,
  pub exclude_regex_method: Option<Regex>,
  pub endpoint_label_fn: Arc<dyn Fn(&Request<Body>) -> String + Send + Sync>,
}

impl PromOpts {
  pub fn new() -> Self {
    Self {
      exclude_regex_status: None,
      exclude_regex_endpoint: None,
      exclude_regex_method: None,
      endpoint_label_fn: Arc::new(|req: &Request<Body>| {
        if let Some(path) = req.extensions().get::<MatchedPath>() {
          path.as_str().to_string()
        } else if req.uri().path() == "/metrics" {
          "/metrics".to_string()
        } else {
          "/unknown".to_string()
        }
      }),
    }
  }

  pub fn check_label(&self, label: &str, regex: &Option<Regex>) -> bool {
    match regex {
      Some(r) => !r.is_match(label),
      None => true,
    }
  }
}

pub async fn prometheus_handler() -> Response {
  let encoder = TextEncoder::new();
  let metric_families = prometheus::gather();
  let mut buffer = Vec::new();

  match encoder.encode(&metric_families, &mut buffer) {
    Ok(_) => {
      PROM_SENSORS_REQUESTS.with_label_values(&["200"]).inc();

      Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, encoder.format_type())
        .body(axum::body::Body::from(buffer))
        .unwrap()
    }
    Err(e) => {
      PROM_SENSORS_REQUESTS.with_label_values(&["500"]).inc();
      eprintln!("Prometheus metrics serialization failed: {}", e);

      (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
    }
  }
}

pub async fn metrics_middleware(State(state): State<Arc<AppState>>, req: Request, next: Next) -> Response {
  let start = Instant::now();
  let method = req.method().clone();
  let prom = state.prom.clone();

  let req_size = calc_approximate_request_size(&req);
  let endpoint = (prom.endpoint_label_fn)(&req);

  let response = next.run(req).await;

  let status = response.status().as_u16().to_string();
  let elapsed = start.elapsed().as_secs_f64();

  if !(prom.check_label(&status, &prom.exclude_regex_status)
    && prom.check_label(&endpoint, &prom.exclude_regex_endpoint)
    && prom.check_label(method.as_str(), &prom.exclude_regex_method))
  {
    return response;
  }

  let resp_size = response
    .headers()
    .get(http::header::CONTENT_LENGTH)
    .and_then(|v| v.to_str().ok())
    .and_then(|s| s.parse::<f64>().ok())
    .unwrap_or(0.0);

  let labels = [&status, &endpoint, method.as_str()];

  HTTP_REQ_COUNT.with_label_values(&labels).inc();
  HTTP_REQ_DURATION.with_label_values(&labels).observe(elapsed);
  HTTP_REQ_SIZE.with_label_values(&labels).observe(req_size);
  HTTP_RESP_SIZE.with_label_values(&labels).observe(resp_size);

  response
}

pub fn start_record_uptime() {
  tokio::spawn(async move {
    let mut ticker = interval(Duration::from_secs(1));
    loop {
      ticker.tick().await;
      UPTIME.inc();
    }
  });
}

pub fn init_prometheus_opts() -> Arc<PromOpts> {
  Arc::new(PromOpts::new())
}

fn calc_approximate_request_size(req: &Request<Body>) -> f64 {
  let mut size = 0;

  size += req.uri().to_string().len();
  size += req.method().as_str().len();
  size += format!("{:?}", req.version()).len();
  for (name, value) in req.headers() {
    size += name.as_str().len();
    size += value.as_bytes().len();
  }
  if let Some(host) = req.uri().host() {
    size += host.len();
  }
  if let Some(len) = req.headers().get(http::header::CONTENT_LENGTH) {
    if let Ok(s) = len.to_str() {
      if let Ok(n) = s.parse::<usize>() {
        size += n;
      }
    }
  }

  size as f64
}
