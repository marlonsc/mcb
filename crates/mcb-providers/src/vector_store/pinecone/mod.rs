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

    /// Convert Pinecone match result to domain `SearchResult`
    pub(crate) fn match_to_search_result(item: &Value, score: f64) -> Result<SearchResult> {
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
fn pinecone_factory(
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── match_to_search_result error propagation ──────────────────────

    #[test]
    fn test_match_to_search_result_missing_id_returns_error() {
        let item = serde_json::json!({ "metadata": {} });
        let result = PineconeVectorStoreProvider::match_to_search_result(&item, 0.9);
        let err = result.expect_err("match_to_search_result should fail when id is missing");
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("id"),
            "error should mention 'id': {err_msg}"
        );
    }

    #[test]
    fn test_match_to_search_result_non_string_id_returns_error() {
        let item = serde_json::json!({ "id": 42, "metadata": {} });
        let result = PineconeVectorStoreProvider::match_to_search_result(&item, 0.9);
        let err = result.expect_err("match_to_search_result should fail when id is not a string");
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("id"),
            "error should mention 'id': {err_msg}"
        );
    }

    #[test]
    fn test_match_to_search_result_missing_metadata_returns_error() {
        let item = serde_json::json!({ "id": "vec_123" });
        let result = PineconeVectorStoreProvider::match_to_search_result(&item, 0.9);
        let err = result.expect_err("match_to_search_result should fail when metadata is missing");
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("metadata"),
            "error should mention 'metadata': {err_msg}"
        );
    }

    #[test]
    fn test_match_to_search_result_valid_item_succeeds() {
        let item = serde_json::json!({
            "id": "vec_123",
            "metadata": {
                "file_path": "src/main.rs",
                "content": "fn main() {}",
                "start_line": 1,
                "language": "rust"
            }
        });
        let result = PineconeVectorStoreProvider::match_to_search_result(&item, 0.95);
        let sr = result.expect("match_to_search_result should succeed for valid item");
        assert_eq!(sr.id, "vec_123");
        assert!((sr.score - 0.95).abs() < f64::EPSILON);
    }

    // ── Factory tests ────────────────────────────────────────────────

    #[test]
    fn test_pinecone_factory_missing_api_key_returns_error() {
        let config = VectorStoreProviderConfig {
            provider: "pinecone".to_owned(),
            uri: Some("https://my-index.svc.pinecone.io".to_owned()),
            api_key: None,
            ..Default::default()
        };
        let result = pinecone_factory(&config);
        let err = result
            .map(|_| ())
            .expect_err("pinecone_factory should fail without api_key");
        assert!(
            err.contains("api_key"),
            "error should mention 'api_key': {err}"
        );
    }

    #[test]
    fn test_pinecone_factory_missing_uri_returns_error() {
        let config = VectorStoreProviderConfig {
            provider: "pinecone".to_owned(),
            api_key: Some("pk-test-key".to_owned()),
            uri: None,
            ..Default::default()
        };
        let result = pinecone_factory(&config);
        let err = result
            .map(|_| ())
            .expect_err("pinecone_factory should fail without uri");
        assert!(err.contains("uri"), "error should mention 'uri': {err}");
    }

    // ── insert_vectors error propagation ──────────────────────────────

    #[tokio::test]
    async fn test_insert_vectors_empty_vectors_returns_error() {
        let provider = PineconeVectorStoreProvider::new(
            "test-key",
            "https://test.pinecone.io",
            Duration::from_secs(5),
            reqwest::Client::new(),
        );
        let collection = CollectionId::from_name("test_collection");
        let vectors: Vec<Embedding> = vec![];
        let metadata: Vec<HashMap<String, Value>> = vec![];

        let result = provider
            .insert_vectors(&collection, &vectors, metadata)
            .await;
        let err = result.expect_err("insert_vectors should fail for empty vectors");
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("empty"),
            "error should mention 'empty': {err_msg}"
        );
    }

    // ── get_vectors_by_ids error propagation ──────────────────────────

    #[tokio::test]
    async fn test_get_vectors_by_ids_empty_ids_returns_error() {
        let provider = PineconeVectorStoreProvider::new(
            "test-key",
            "https://test.pinecone.io",
            Duration::from_secs(5),
            reqwest::Client::new(),
        );
        let collection = CollectionId::from_name("test_collection");
        let ids: Vec<String> = vec![];

        let result = provider.get_vectors_by_ids(&collection, &ids).await;
        let err = result.expect_err("get_vectors_by_ids should fail for empty ids");
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("empty"),
            "error should mention 'empty': {err_msg}"
        );
    }
}
