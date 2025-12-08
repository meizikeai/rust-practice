// src/libs/prometheus.rs
use crate::router::AppState;
use axum::{
  body::{Body, HttpBody},
  extract::{Request, State},
  middleware::Next,
  response::Response,
};
use http_body::Frame;
use lazy_static::lazy_static;
use pin_project::pin_project;
use prometheus::{
  Encoder, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGaugeVec, Opts, TextEncoder, register_histogram_vec, register_int_counter, register_int_counter_vec,
  register_int_gauge_vec,
};
use regex::Regex;
use std::convert::AsRef;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Instant,
};
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

  // 请求大小（用 Histogram 替代 Summary）
  static ref HTTP_REQ_SIZE: HistogramVec =
    register_histogram_vec!(HistogramOpts::new("http_request_size_bytes", "HTTP request sizes in bytes")
      .namespace(NAMESPACE)
      .buckets(vec![100.0, 500.0, 1_000.0, 5_000.0, 10_000.0, 50_000.0]),
      &["status", "endpoint", "method"]).unwrap();

  // 响应大小（用 Histogram 替代 Summary）
  static ref HTTP_RESP_SIZE: HistogramVec =
    register_histogram_vec!(HistogramOpts::new("http_response_size_bytes", "HTTP response sizes in bytes")
      .namespace(NAMESPACE)
      .buckets(vec![100.0, 500.0, 1_000.0, 5_000.0, 10_000.0, 50_000.0]),
      &["status", "endpoint", "method"]).unwrap();

  // 调用第三方监控
  static ref CLIENT_REQ_DURATION: HistogramVec =
    register_histogram_vec!(HistogramOpts::new("client_request_duration_seconds", "服务请求第三方的耗时")
      .buckets(vec![0.1, 0.25, 0.5, 1.0, 5.0]),
      &["status", "destination_service"]).unwrap();

  static ref CLIENT_REQ_ERROR: IntGaugeVec =
    register_int_gauge_vec!(Opts::new("client_request_error_count", "客户端请求错误数"), &["err_name"]).unwrap();
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
      endpoint_label_fn: Arc::new(|req| req.uri().path().to_string()),
    }
  }

  pub fn check_label(&self, label: &str, regex: &Option<Regex>) -> bool {
    match regex {
      Some(r) => !r.is_match(label),
      None => true,
    }
  }
}

// 导出 Prometheus 指标文本
pub async fn prometheus_handler() -> String {
  let encoder = TextEncoder::new();
  let metric_families = prometheus::gather();
  let mut buffer = Vec::new();
  encoder.encode(&metric_families, &mut buffer).unwrap();
  String::from_utf8(buffer).unwrap()
}

// 主体中间件：采集 HTTP 请求指标
pub async fn metrics_middleware(State(state): State<Arc<AppState>>, mut req: Request, next: Next) -> Response {
  let start = Instant::now();
  let method = req.method().clone();
  let opts = state.prom_opts.clone();
  let endpoint = (opts.endpoint_label_fn)(&req);

  // 请求大小
  // 注意：Axum 的 Request 消耗后无法再访问 Body，因此这里只能在上游 clone 或提前计算
  // 为简单起见，这里可提前在 extractor 层计算或直接使用 0
  // let req_size = 0.0;

  // 近似值
  // let req_size = calc_request_size(&req) as f64;

  // 真实值
  let byte_counter = ByteCounter::new();
  req.extensions_mut().insert(byte_counter.clone());
  let (parts, body) = req.into_parts();
  let counting_body = CountingBody::new(body, byte_counter.clone());
  let req_with_body = Request::from_parts(parts, Body::new(counting_body));
  let response = next.run(req_with_body).await;
  let req_size = byte_counter.load() as f64;

  let status = response.status().as_u16().to_string();
  let elapsed = start.elapsed().as_secs_f64();

  if !(opts.check_label(&status, &opts.exclude_regex_status)
    && opts.check_label(&endpoint, &opts.exclude_regex_endpoint)
    && opts.check_label(method.as_str(), &opts.exclude_regex_method))
  {
    return response;
  }

  // 响应大小
  let resp_size = response.body().size_hint().upper().unwrap_or(0) as f64;

  let labels = [&status, &endpoint, method.as_str()];

  HTTP_REQ_COUNT.with_label_values(&labels).inc();
  HTTP_REQ_DURATION.with_label_values(&labels).observe(elapsed);
  HTTP_REQ_SIZE.with_label_values(&labels).observe(req_size);
  HTTP_RESP_SIZE.with_label_values(&labels).observe(resp_size);

  response
}

// 启动 uptime 定时任务
pub fn start_record_uptime() {
  std::thread::spawn(move || {
    let rt = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .expect("Failed to build uptime tokio runtime");

    rt.block_on(async move {
      let mut ticker = interval(Duration::from_secs(1));
      loop {
        ticker.tick().await;
        if let Err(e) = std::panic::catch_unwind(|| {
          UPTIME.inc();
        }) {
          println!("record_uptime panicked: {:?}", e);
        }
      }
    });
  });
}

// 初始化默认的 Prometheus 配置
pub fn init_prometheus_opts() -> Arc<PromOpts> {
  Arc::new(PromOpts::new())
}

// 第三方错误计数上报
#[allow(dead_code)]
pub fn record_error_count(name: &str) {
  CLIENT_REQ_ERROR.with_label_values(&[name]).inc();
}

// 不使用的方法
// // 版本字符串（用于计算请求大小）
// #[allow(dead_code)]
// fn version_to_str(version: &http::Version) -> &'static str {
//   match version {
//     &http::Version::HTTP_09 => "HTTP/0.9",
//     &http::Version::HTTP_10 => "HTTP/1.0",
//     &http::Version::HTTP_11 => "HTTP/1.1",
//     &http::Version::HTTP_2 => "HTTP/2",
//     &http::Version::HTTP_3 => "HTTP/3",
//     _ => "UNKNOWN",
//   }
// }

// // 计算请求大小
// #[allow(dead_code)]
// fn calc_request_size(req: &Request<Body>) -> usize {
//   let mut size = 0;
//   size += req.uri().to_string().len();
//   size += req.method().as_str().len();
//   size += version_to_str(&req.version()).len();

//   for (name, value) in req.headers() {
//     size += name.as_str().len();
//     size += value.len();
//   }

//   // host 可能为空
//   if let Some(host) = req.uri().host() {
//     size += host.len();
//   }

//   // content-length
//   if let Some(len) = req.headers().get("content-length") {
//     if let Ok(s) = len.to_str() {
//       if let Ok(n) = s.parse::<usize>() {
//         size += n;
//       }
//     }
//   }

//   size
// }

// body 的近似值
#[derive(Debug, Clone)]
pub struct ByteCounter(Arc<AtomicUsize>);

impl ByteCounter {
  pub fn new() -> Self {
    Self(Arc::new(AtomicUsize::new(0)))
  }

  pub fn load(&self) -> usize {
    self.0.load(Ordering::Relaxed)
  }

  pub fn inc(&self, amount: usize) {
    self.0.fetch_add(amount, Ordering::Relaxed);
  }
}

#[pin_project]
#[derive(Debug)]
pub struct CountingBody<B> {
  #[pin]
  inner: B,
  counter: ByteCounter,
}

impl<B> CountingBody<B> {
  pub fn new(inner: B, counter: ByteCounter) -> Self {
    Self { inner, counter }
  }

  #[allow(dead_code)]
  pub fn counter(&self) -> &ByteCounter {
    &self.counter
  }
}

impl<B> HttpBody for CountingBody<B>
where
  B: HttpBody,
  B::Data: AsRef<[u8]>,
{
  type Data = B::Data;
  type Error = B::Error;

  fn poll_frame(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
    let mut this = self.project();
    let frame = this.inner.as_mut().poll_frame(cx);

    if let Poll::Ready(Some(Ok(frame))) = &frame {
      if let Some(data) = frame.data_ref() {
        this.counter.inc(data.as_ref().len());
      }
    }

    frame
  }
}

impl CountingBody<Body> {
  #[allow(dead_code)]
  pub fn into_body(self) -> Body {
    Body::new(self)
  }
}
