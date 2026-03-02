//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#vector-store-providers)
//!
//! Pinecone Vector Store Provider
//!
//! Implements the `VectorStoreProvider` using Pinecone's cloud vector database REST API.
//!
//! Pinecone is a managed vector database optimized for machine learning applications.
//! This provider communicates via Pinecone's REST API using the reqwest HTTP client.

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use mcb_domain::constants::http::CONTENT_TYPE_JSON;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::VectorStoreProvider;

use mcb_domain::value_objects::SearchResult;
use reqwest::Client;
use serde_json::Value;

use crate::constants::{
    EDGEVEC_DEFAULT_DIMENSIONS, HTTP_HEADER_CONTENT_TYPE, PINECONE_API_KEY_HEADER,
    PROVIDER_RETRY_BACKOFF_MS, PROVIDER_RETRY_COUNT,
};
use crate::utils::http::{VectorDbRequestParams, send_vector_db_request};
use crate::utils::vector_store::search_result_from_json_metadata;

mod admin;
mod browser;
mod provider;

/// Pinecone vector store provider
///
/// Implements the vector store domain ports using Pinecone's cloud REST API.
/// Supports index management, vector upsert, search, and metadata filtering.
pub struct PineconeVectorStoreProvider {
    api_key: String,
    host: String,
    timeout: Duration,
    http_client: Client,
    /// Track collections (namespaces) locally with their dimensions
    collections: Arc<DashMap<String, usize>>,
}

impl PineconeVectorStoreProvider {
    /// Create a new Pinecone vector store provider
    ///
    /// # Arguments
    /// * `api_key` - Pinecone API key
    /// * `host` - Pinecone index host URL
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    #[must_use]
    pub fn new(api_key: &str, host: &str, timeout: Duration, http_client: Client) -> Self {
        Self {
            api_key: api_key.trim().to_owned(),
            host: host.trim_end_matches('/').to_owned(),
            timeout,
            http_client,
            collections: Arc::new(DashMap::new()),
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

    pub(crate) fn collection_dimensions(&self, collection: &str) -> usize {
        self.collections
            .get(collection)
            .map_or(EDGEVEC_DEFAULT_DIMENSIONS, |d| *d.value())
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

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use mcb_domain::registry::vector_store::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

/// Factory function for creating Pinecone vector store provider instances.
///
/// # Errors
///
/// Returns `Err` if required configuration (API key, host) is missing.
pub fn pinecone_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    use crate::utils::http::{DEFAULT_HTTP_TIMEOUT, create_default_client};

    let api_key = config
        .api_key
        .clone()
        .ok_or_else(|| "Pinecone requires api_key".to_owned())?;
    let host = config
        .uri
        .clone()
        .ok_or_else(|| "Pinecone requires uri (index host URL)".to_owned())?;
    let http_client = create_default_client()?;

    Ok(Arc::new(PineconeVectorStoreProvider::new(
        &api_key,
        &host,
        DEFAULT_HTTP_TIMEOUT,
        http_client,
    )))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static PINECONE_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "pinecone",
    description: "Pinecone cloud vector database (managed, serverless)",
    build: pinecone_factory,
};
