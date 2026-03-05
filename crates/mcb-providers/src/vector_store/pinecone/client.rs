//! Pinecone vector store client implementation.

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::VectorStoreProvider;
use mcb_utils::constants::http::CONTENT_TYPE_JSON;

use mcb_domain::value_objects::SearchResult;
use reqwest::Client;
use serde_json::Value;

use crate::utils::http::{VectorDbRequestParams, send_vector_db_request};
use crate::utils::vector_store::search_result_from_json_metadata;
use mcb_utils::constants::http::{
    HTTP_HEADER_CONTENT_TYPE, PINECONE_API_KEY_HEADER, PROVIDER_RETRY_BACKOFF_MS,
    PROVIDER_RETRY_COUNT,
};

/// Pinecone vector store provider
///
/// Implements the vector store domain ports using Pinecone's cloud REST API.
/// Supports index management, vector upsert, search, and metadata filtering.
pub struct PineconeVectorStoreProvider {
    pub(super) api_key: String,
    pub(super) host: String,
    pub(super) timeout: Duration,
    pub(super) http_client: Client,
    /// Track collections (namespaces) locally with their dimensions
    pub(super) collections: Arc<DashMap<String, usize>>,
    /// Default dimensions sourced from provider config (embedding model).
    /// Used when a collection's dimensions are unknown locally.
    pub(super) default_dimensions: Option<usize>,
}

impl PineconeVectorStoreProvider {
    /// Create a new Pinecone vector store provider
    ///
    /// # Arguments
    /// * `api_key` - Pinecone API key
    /// * `host` - Pinecone index host URL
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    /// * `default_dimensions` - Default vector dimensions from config (embedding model)
    #[must_use]
    pub fn new(
        api_key: &str,
        host: &str,
        timeout: Duration,
        http_client: Client,
        default_dimensions: Option<usize>,
    ) -> Self {
        Self {
            api_key: api_key.trim().to_owned(),
            host: host.trim_end_matches('/').to_owned(),
            timeout,
            http_client,
            collections: Arc::new(DashMap::new()),
            default_dimensions,
        }
    }

    /// Build a URL for the Pinecone API
    pub(crate) fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.host, path)
    }

    /// Make an authenticated request to Pinecone
    pub(crate) async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value> {
        let headers = vec![
            (PINECONE_API_KEY_HEADER, self.api_key.clone()),
            (HTTP_HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON.to_owned()),
        ];

        send_vector_db_request(VectorDbRequestParams {
            client: &self.http_client,
            method,
            url: self.api_url(path),
            timeout: self.timeout,
            provider: "Pinecone",
            operation: path,
            headers: &headers,
            body: body.as_ref(),
            retry_attempts: PROVIDER_RETRY_COUNT,
            retry_backoff_ms: PROVIDER_RETRY_BACKOFF_MS,
        })
        .await
    }

    pub(crate) fn extract_json_field<'a, T, F>(
        obj: &'a Value,
        field: &str,
        parser: F,
        context: &str,
        expected: &str,
    ) -> Result<T>
    where
        F: FnOnce(&'a Value) -> Option<T>,
    {
        obj.get(field).and_then(parser).ok_or_else(|| {
            Error::vector_db(format!(
                "Invalid Pinecone {context}: missing or non-{expected} '{field}' field"
            ))
        })
    }

    pub(crate) async fn query_match_results(&self, payload: Value) -> Result<Vec<SearchResult>> {
        let response = self
            .request(reqwest::Method::POST, "/query", Some(payload))
            .await?;
        let matches =
            Self::extract_json_field(&response, "matches", Value::as_array, "response", "array")?;
        matches
            .iter()
            .map(|m| {
                let score =
                    Self::extract_json_field(m, "score", Value::as_f64, "match", "numeric")?;
                Self::match_to_search_result(m, score)
            })
            .collect()
    }

    pub(crate) fn collection_dimensions(&self, collection: &str) -> Result<usize> {
        if let Some(d) = self.collections.get(collection) {
            return Ok(*d.value());
        }
        self.default_dimensions.ok_or_else(|| {
            Error::vector_db(format!(
                "Unknown dimensions for collection '{collection}': \
                 set `dimensions` in vector store config or call create_collection() first"
            ))
        })
    }

    /// Convert Pinecone match result to domain `SearchResult`.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing from the match result.
    pub fn match_to_search_result(item: &Value, score: f64) -> Result<SearchResult> {
        let id = Self::extract_json_field(item, "id", Value::as_str, "match", "string")?.to_owned();
        let metadata = Self::extract_json_field(item, "metadata", Some, "match", "value")?;
        Ok(search_result_from_json_metadata(id, metadata, score))
    }
}
