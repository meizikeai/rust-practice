// src/utils/fetch.rs
use crate::utils::response::{AppError, Code};
use reqwest::{
    Client, Method,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Fetch {
    client: Client,
}

#[derive(Deserialize, Debug)]
pub struct ExternalResponse<T> {
    pub code: u32,
    pub message: String,
    pub data: Option<T>,
}

#[allow(dead_code)]
impl Fetch {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(2))
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .expect("Failed to create client");

        Self { client }
    }

    pub async fn request<T, B>(
        &self,
        method: Method,
        url: &str,
        body: Option<&B>,
        params: Option<&HashMap<String, String>>,
        headers: Option<HeaderMap>,
    ) -> Result<T, AppError>
    where
        T: DeserializeOwned + Default,
        B: Serialize + ?Sized,
    {
        let mut rb = self.client.request(method, url);
        if let Some(p) = params {
            rb = rb.query(p);
        }
        if let Some(h) = headers {
            rb = rb.headers(h);
        }
        if let Some(b) = body {
            rb = rb.json(b);
        }

        let resp = rb.send().await.map_err(|e| {
            tracing::error!("Request Transport Error: {}", e);
            AppError::Logic(Code::InternalServerError)
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_default();
            tracing::error!("External API HTTP Error: {} - {}", status, error_text);
            return Err(AppError::Logic(Code::InternalServerError));
        }

        let res = resp.json::<ExternalResponse<T>>().await.map_err(|e| {
            tracing::error!("JSON Decode Error: {}", e);
            AppError::Logic(Code::InternalServerError)
        })?;

        if res.code != 200 {
            tracing::error!("External API Business Error: code={}, msg={}", res.code, res.message);
            return Err(AppError::Logic(Code::InternalServerError));
        }

        Ok(res.data.unwrap_or_default())
    }

    pub async fn get<T: DeserializeOwned + Default>(&self, uri: &str) -> Result<T, AppError> {
        self.request::<T, ()>(Method::GET, uri, None, None, None).await
    }

    pub async fn post<T: DeserializeOwned + Default, B: Serialize>(&self, url: &str, body: &B) -> Result<T, AppError> {
        self.request(Method::POST, url, Some(body), None, None).await
    }

    pub async fn post_with_headers<T: DeserializeOwned + Default, B: Serialize>(
        &self,
        url: &str,
        body: &B,
        headers: HeaderMap,
    ) -> Result<T, AppError> {
        self.request(Method::POST, url, Some(body), None, Some(headers)).await
    }
}
