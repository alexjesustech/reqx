// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! HTTP client implementation

use super::{HttpConfig, Response};
use crate::parser::{BodySection, ReqxFile};
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};

pub struct Client {
    inner: reqwest::Client,
    timeout: Duration,
    retries: u32,
    retry_delay: Duration,
}

impl Client {
    pub fn new(timeout_ms: u64, retries: u32, retry_delay_ms: u64, config: HttpConfig) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .connect_timeout(Duration::from_secs(10));

        if config.follow_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::limited(config.max_redirects));
        } else {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        }

        if let Some(proxy_url) = &config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .context("Invalid proxy URL")?;
            builder = builder.proxy(proxy);
        }

        if config.insecure {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let inner = builder.build().context("Failed to create HTTP client")?;

        Ok(Self {
            inner,
            timeout: Duration::from_millis(timeout_ms),
            retries,
            retry_delay: Duration::from_millis(retry_delay_ms),
        })
    }

    pub async fn execute(&self, reqx_file: &ReqxFile) -> Result<Response> {
        let mut last_error = None;

        for attempt in 0..=self.retries {
            if attempt > 0 {
                tokio::time::sleep(self.retry_delay).await;
            }

            match self.execute_once(reqx_file).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // Only retry on network errors, not on HTTP errors
                    if e.is_network_error() {
                        last_error = Some(e);
                        continue;
                    }
                    return Err(e.into());
                }
            }
        }

        Err(last_error.unwrap().into())
    }

    async fn execute_once(&self, reqx_file: &ReqxFile) -> Result<Response, RequestError> {
        let start = Instant::now();

        // Build URL with query parameters
        let mut url = reqx_file.request.url.clone();
        if !reqx_file.query.is_empty() {
            let query_string: String = reqx_file
                .query
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");

            if url.contains('?') {
                url = format!("{}&{}", url, query_string);
            } else {
                url = format!("{}?{}", url, query_string);
            }
        }

        // Build headers
        let mut headers = HeaderMap::new();
        for (key, value) in &reqx_file.headers {
            let header_name = HeaderName::from_str(key)
                .map_err(|_| RequestError::InvalidHeader(key.clone()))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|_| RequestError::InvalidHeader(key.clone()))?;
            headers.insert(header_name, header_value);
        }

        // Build request
        let method = reqwest::Method::from_str(&reqx_file.request.method)
            .map_err(|_| RequestError::InvalidMethod(reqx_file.request.method.clone()))?;

        let mut request = self.inner.request(method, &url).headers(headers);

        // Add body
        if let Some(body) = &reqx_file.body {
            request = match body {
                BodySection::Json(json) => request.json(json),
                BodySection::Raw(raw) => request.body(raw.clone()),
                BodySection::FormData(form) => request.form(form),
            };
        }

        // Execute request
        let response = request
            .send()
            .await
            .map_err(|e| RequestError::Network(e.to_string()))?;

        let status = response.status().as_u16();

        // Parse response headers
        let response_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_string(),
                    v.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect();

        // Parse response body
        let body_text = response
            .text()
            .await
            .map_err(|e| RequestError::Network(e.to_string()))?;

        let body: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_else(|_| {
            serde_json::Value::String(body_text)
        });

        let duration = start.elapsed();

        Ok(Response {
            status,
            headers: response_headers,
            body,
            duration,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Timeout")]
    Timeout,
}

impl RequestError {
    pub fn is_network_error(&self) -> bool {
        matches!(self, Self::Network(_) | Self::Timeout)
    }
}

impl From<RequestError> for anyhow::Error {
    fn from(err: RequestError) -> Self {
        anyhow::anyhow!("{}", err)
    }
}
