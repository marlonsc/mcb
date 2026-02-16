use std::future::Future;
use std::time::Duration;

use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::Embedding;
use reqwest::Client;
use serde_json::Value;

/// Shared HTTP client configuration for embedding providers.
///
/// Holds common state (API key, base URL, model, timeout, HTTP client)
/// used across all HTTP-based embedding provider implementations.
pub(crate) struct HttpEmbeddingClient {
    /// API key for authentication.
    pub(crate) api_key: String,
    /// Base URL for the embedding API.
    pub(crate) base_url: String,
    /// Model identifier (e.g., "voyage-code-3", "text-embedding-3-small").
    pub(crate) model: String,
    /// Request timeout duration.
    pub(crate) timeout: Duration,
    /// Shared reqwest HTTP client.
    pub(crate) client: Client,
}

impl HttpEmbeddingClient {
    /// Creates a new HTTP embedding client with the given configuration.
    #[must_use]
    pub(crate) fn new(
        api_key: &str,
        base_url: Option<String>,
        default_base_url: &str,
        model: String,
        timeout: Duration,
        client: Client,
    ) -> Self {
        Self {
            api_key: api_key.to_owned(),
            base_url: base_url.unwrap_or_else(|| default_base_url.to_owned()),
            model,
            timeout,
            client,
        }
    }
}

/// Extracts the `data` array from a standard embedding API response.
pub(crate) fn embedding_data_array(response_data: &Value) -> Result<&[Value]> {
    response_data
        .get("data")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| Error::embedding("Invalid response format: missing data array"))
}

/// Parses a JSON array at the given pointer path into a `Vec<f32>`,
/// tolerating both numeric and string representations.
pub(crate) fn parse_float_array_lossy(
    response_data: &Value,
    pointer: &str,
    missing_message: &str,
) -> Result<Vec<f32>> {
    let array = response_data
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| Error::embedding(missing_message))?;

    let mut values = Vec::with_capacity(array.len());
    for value in array {
        if let Some(n) = value.as_f64() {
            values.push(n as f32);
            continue;
        }
        if let Some(s) = value.as_str() {
            let parsed = s
                .parse::<f32>()
                .map_err(|_| Error::embedding("Invalid embedding value: expected float"))?;
            values.push(parsed);
            continue;
        }
        return Err(Error::embedding("Invalid embedding value: expected number"));
    }

    Ok(values)
}

/// Parse a standard embedding response item (OpenAI/Anthropic/VoyageAI format).
///
/// Extracts the embedding vector from `item[EMBEDDING_RESPONSE_FIELD]` and wraps
/// it in an `Embedding` value object with the given model name and dimensions.
pub(crate) fn parse_standard_embedding(
    model: &str,
    dimensions: usize,
    index: usize,
    item: &Value,
) -> Result<Embedding> {
    let embedding_vec = super::http::parse_embedding_vector(
        item,
        crate::constants::EMBEDDING_RESPONSE_FIELD,
        index,
    )?;
    Ok(Embedding {
        vector: embedding_vec,
        model: model.to_owned(),
        dimensions,
    })
}

/// Processes a batch of texts by fetching embeddings and parsing each result.
///
/// Returns an error if the response count doesn't match the input count.
pub(crate) async fn process_batch<Fut, Parser>(
    texts: &[String],
    fetch: Fut,
    mut parse_item: Parser,
) -> Result<Vec<Embedding>>
where
    Fut: Future<Output = Result<Value>>,
    Parser: FnMut(usize, &Value) -> Result<Embedding>,
{
    if texts.is_empty() {
        return Ok(Vec::new());
    }

    let response_data = fetch.await?;
    let data = embedding_data_array(&response_data)?;

    let mut embeddings = Vec::with_capacity(data.len());
    for (index, item) in data.iter().enumerate() {
        embeddings.push(parse_item(index, item)?);
    }

    if embeddings.len() != texts.len() {
        return Err(Error::embedding(format!(
            "Embedding count mismatch: expected {}, got {}",
            texts.len(),
            embeddings.len()
        )));
    }

    Ok(embeddings)
}
