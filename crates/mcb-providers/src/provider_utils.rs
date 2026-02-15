use std::future::Future;
use std::time::Duration;

use mcb_domain::error::{Error, Result};
use reqwest::Client;
use serde_json::Value;

use crate::utils::http::{RequestErrorKind, handle_request_error_with_kind};
use crate::utils::http_response::HttpResponseUtils;

pub(crate) struct RetryConfig {
    pub max_attempts: usize,
    pub base_delay: Duration,
}

impl RetryConfig {
    pub const fn new(max_attempts: usize, base_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
        }
    }
}

pub(crate) async fn retry_with_backoff<T, E, F, Fut, P>(
    config: RetryConfig,
    mut operation: F,
    should_retry: P,
) -> std::result::Result<T, E>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = std::result::Result<T, E>>,
    P: Fn(&E) -> bool,
{
    let attempts = config.max_attempts.max(1);

    for attempt in 0..attempts {
        match operation(attempt).await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if attempt + 1 == attempts || !should_retry(&error) {
                    return Err(error);
                }

                tokio::time::sleep(config.base_delay.mul_f64((attempt + 1) as f64)).await;
            }
        }
    }

    unreachable!("retry loop must return success or error")
}

/// Parameters for [`send_json_request`].
pub(crate) struct JsonRequestParams<'a> {
    /// HTTP client to use.
    pub client: &'a Client,
    /// HTTP method (GET, POST, etc.).
    pub method: reqwest::Method,
    /// Target URL.
    pub url: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Provider name for error messages.
    pub provider: &'a str,
    /// Operation name for error messages.
    pub operation: &'a str,
    /// Error classification kind.
    pub kind: RequestErrorKind,
    /// Additional headers.
    pub headers: &'a [(&'a str, String)],
    /// Optional JSON body.
    pub body: Option<&'a Value>,
}

/// Send a JSON request with configurable parameters.
pub(crate) async fn send_json_request(params: JsonRequestParams<'_>) -> Result<Value> {
    let mut builder = params
        .client
        .request(params.method, params.url)
        .timeout(params.timeout);

    for (key, value) in params.headers {
        builder = builder.header(*key, value);
    }

    if let Some(payload) = params.body {
        builder = builder.json(payload);
    }

    let response = builder.send().await.map_err(|e| {
        handle_request_error_with_kind(
            e,
            params.timeout,
            params.provider,
            params.operation,
            params.kind,
        )
    })?;

    HttpResponseUtils::check_and_parse(response, params.provider).await
}

pub(crate) fn embedding_data_array(response_data: &Value, expected_len: usize) -> Result<&[Value]> {
    let data = response_data["data"].as_array().ok_or_else(|| {
        Error::embedding("Invalid response format: missing data array".to_owned())
    })?;

    if data.len() != expected_len {
        return Err(Error::embedding(format!(
            "Response data count mismatch: expected {}, got {}",
            expected_len,
            data.len()
        )));
    }

    Ok(data.as_slice())
}

pub(crate) fn parse_float_array_lossy(
    response_data: &Value,
    pointer: &str,
    missing_message: &str,
) -> Result<Vec<f32>> {
    let arr = response_data
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| Error::embedding(missing_message.to_owned()))?;

    Ok(arr
        .iter()
        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
        .collect())
}
