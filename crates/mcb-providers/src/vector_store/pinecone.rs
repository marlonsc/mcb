//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#vector-store-providers)
//!
//! Pinecone Vector Store Provider
//!
//! Implements the `VectorStoreProvider` using Pinecone's cloud vector database REST API.
//!
//! Pinecone is a managed vector database optimized for machine learning applications.
//! This provider communicates via Pinecone's REST API using the reqwest HTTP client.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use dashmap::DashMap;
use mcb_domain::constants::http::CONTENT_TYPE_JSON;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::utils::id;
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};
use reqwest::Client;
use serde_json::Value;

use crate::constants::{
    EDGEVEC_DEFAULT_DIMENSIONS, HTTP_HEADER_CONTENT_TYPE, PINECONE_API_KEY_HEADER,
    STATS_FIELD_COLLECTION, STATS_FIELD_PROVIDER, STATS_FIELD_STATUS, STATS_FIELD_VECTORS_COUNT,
    STATUS_ACTIVE, STATUS_UNKNOWN, VECTOR_FIELD_FILE_PATH, VECTOR_STORE_RETRY_BACKOFF_SECS,
    VECTOR_STORE_RETRY_COUNT,
};
use crate::utils::http::{VectorDbRequestParams, send_vector_db_request};
use crate::utils::vector_store::search_result_from_json_metadata;

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
    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.host, path)
    }

    /// Make an authenticated request to Pinecone
    async fn request(
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
            retry_attempts: VECTOR_STORE_RETRY_COUNT,
            retry_backoff_secs: VECTOR_STORE_RETRY_BACKOFF_SECS,
        })
        .await
    }

    /// Convert Pinecone match result to domain `SearchResult`
    fn match_to_search_result(item: &Value, score: f64) -> Result<SearchResult> {
        let id = item["id"]
            .as_str()
            .ok_or_else(|| {
                Error::vector_db(
                    "Invalid Pinecone match: missing or non-string 'id' field".to_owned(),
                )
            })?
            .to_owned();
        let metadata = item.get("metadata").ok_or_else(|| {
            Error::vector_db("Invalid Pinecone match: missing 'metadata' field".to_owned())
        })?;
        Ok(search_result_from_json_metadata(id, metadata, score))
    }
}

#[async_trait]
impl VectorStoreAdmin for PineconeVectorStoreProvider {
    // --- Admin Methods ---

    async fn collection_exists(&self, name: &CollectionId) -> Result<bool> {
        let name_str = name.to_string();
        Ok(self.collections.contains_key(&name_str))
    }

    async fn get_stats(&self, collection: &CollectionId) -> Result<HashMap<String, Value>> {
        let collection_str = collection.to_string();
        let payload = serde_json::json!({
            "filter": {},
        });

        let response = self
            .request(
                reqwest::Method::POST,
                "/describe_index_stats",
                Some(payload),
            )
            .await;

        let mut stats = HashMap::new();
        stats.insert(
            STATS_FIELD_COLLECTION.to_owned(),
            serde_json::json!(collection.to_string()),
        );
        stats.insert(
            STATS_FIELD_PROVIDER.to_owned(),
            serde_json::json!(self.provider_name()),
        );

        match response {
            Ok(data) => {
                if let Some(namespaces) = data.get("namespaces")
                    && let Some(ns) = namespaces.get(&collection_str)
                    && let Some(count) = ns.get("vectorCount")
                {
                    stats.insert(STATS_FIELD_VECTORS_COUNT.to_owned(), count.clone());
                }
                stats.insert(
                    STATS_FIELD_STATUS.to_owned(),
                    serde_json::json!(STATUS_ACTIVE),
                );
            }
            Err(_) => {
                stats.insert(
                    STATS_FIELD_STATUS.to_owned(),
                    serde_json::json!(STATUS_UNKNOWN),
                );
                stats.insert(STATS_FIELD_VECTORS_COUNT.to_owned(), serde_json::json!(0));
            }
        }

        Ok(stats)
    }

    async fn flush(&self, _collection: &CollectionId) -> Result<()> {
        // Pinecone writes are immediately consistent
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "pinecone"
    }
}

#[async_trait]
impl VectorStoreBrowser for PineconeVectorStoreProvider {
    // --- Browser Methods ---

    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let collections: Vec<CollectionInfo> = self
            .collections
            .iter()
            .map(|entry| CollectionInfo::new(entry.key(), 0, 0, None, self.provider_name()))
            .collect();
        Ok(collections)
    }

    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>> {
        let results = self.list_vectors(collection, limit).await?;
        Ok(crate::utils::vector_store::build_file_info_from_results(
            results,
        ))
    }

    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let filter = serde_json::json!({
            (VECTOR_FIELD_FILE_PATH): { "$eq": file_path }
        });

        let collection_str = collection.to_string();
        let dimensions = self
            .collections
            .get(&collection_str)
            .map_or(EDGEVEC_DEFAULT_DIMENSIONS, |d| *d.value());

        let zero_vector = vec![0.0f32; dimensions];

        let payload = serde_json::json!({
            "vector": zero_vector,
            "topK": 100,
            "namespace": collection_str,
            "includeMetadata": true,
            "filter": filter
        });

        let response = self
            .request(reqwest::Method::POST, "/query", Some(payload))
            .await?;

        let matches = response["matches"].as_array().ok_or_else(|| {
            Error::vector_db("Invalid Pinecone response: missing matches array".to_owned())
        })?;

        let mut results: Vec<SearchResult> = matches
            .iter()
            .map(|m| {
                let score = m["score"].as_f64().ok_or_else(|| {
                    Error::vector_db(
                        "Invalid Pinecone match: missing or non-numeric 'score' field".to_owned(),
                    )
                })?;
                Self::match_to_search_result(m, score)
            })
            .collect::<Result<Vec<_>>>()?;

        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}

#[async_trait]
impl VectorStoreProvider for PineconeVectorStoreProvider {
    // --- Provider Methods ---

    async fn create_collection(&self, name: &CollectionId, dimensions: usize) -> Result<()> {
        let name_str = name.to_string();
        if self.collections.contains_key(&name_str) {
            return Err(Error::vector_db(format!(
                "Collection '{name}' already exists"
            )));
        }
        // Pinecone uses namespaces within an index; creation is implicit on first upsert
        self.collections.insert(name_str, dimensions);
        Ok(())
    }

    async fn delete_collection(&self, name: &CollectionId) -> Result<()> {
        let name_str = name.to_string();
        let payload = serde_json::json!({
            "deleteAll": true,
            "namespace": name_str
        });

        self.request(reqwest::Method::POST, "/vectors/delete", Some(payload))
            .await?;

        self.collections.remove(&name.to_string());
        Ok(())
    }

    async fn insert_vectors(
        &self,
        collection: &CollectionId,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>> {
        if vectors.is_empty() {
            mcb_domain::warn!("pinecone", "insert_vectors called with empty vectors array");
            return Err(Error::vector_db(
                "Cannot insert empty vectors array".to_owned(),
            ));
        }
        let collection_str = collection.to_string();

        let mut ids = Vec::with_capacity(vectors.len());
        let mut pinecone_vectors = Vec::with_capacity(vectors.len());
        let batch_size = crate::constants::PINECONE_UPSERT_BATCH_SIZE;

        for (i, (embedding, meta)) in vectors.iter().zip(metadata.iter()).enumerate() {
            let id = format!("vec_{}", id::generate());
            pinecone_vectors.push(serde_json::json!({
                "id": id,
                "values": embedding.vector,
                "metadata": meta
            }));
            ids.push(id);

            // Pinecone has a batch size limit; upsert in chunks
            if pinecone_vectors.len() >= batch_size || i == vectors.len() - 1 {
                let payload = serde_json::json!({
                    "vectors": pinecone_vectors,
                    "namespace": collection_str
                });

                self.request(reqwest::Method::POST, "/vectors/upsert", Some(payload))
                    .await?;

                pinecone_vectors.clear();
            }
        }

        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let collection_str = collection.to_string();
        let mut payload = serde_json::json!({
            "vector": query_vector,
            "topK": limit,
            "namespace": collection_str,
            "includeMetadata": true
        });

        if let Some(filter_str) = filter
            && let Ok(filter_val) = serde_json::from_str::<Value>(filter_str)
        {
            payload["filter"] = filter_val;
        }

        let response = self
            .request(reqwest::Method::POST, "/query", Some(payload))
            .await?;

        let matches = response["matches"].as_array().ok_or_else(|| {
            Error::vector_db("Invalid Pinecone response: missing matches array".to_owned())
        })?;

        let results = matches
            .iter()
            .map(|m| {
                let score = m["score"].as_f64().ok_or_else(|| {
                    Error::vector_db(
                        "Invalid Pinecone match: missing or non-numeric 'score' field".to_owned(),
                    )
                })?;
                Self::match_to_search_result(m, score)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(results)
    }

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let collection_str = collection.to_string();

        let payload = serde_json::json!({
            "ids": ids,
            "namespace": collection_str
        });

        self.request(reqwest::Method::POST, "/vectors/delete", Some(payload))
            .await?;

        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &CollectionId,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        if ids.is_empty() {
            mcb_domain::warn!("pinecone", "get_vectors_by_ids called with empty ids array");
            return Err(Error::vector_db(
                "Cannot fetch vectors with empty ids array".to_owned(),
            ));
        }
        let collection_str = collection.to_string();

        let payload = serde_json::json!({
            "ids": ids,
            "namespace": collection_str
        });

        let response = self
            .request(reqwest::Method::GET, "/vectors/fetch", Some(payload))
            .await?;

        let vectors_obj = response["vectors"].as_object().ok_or_else(|| {
            Error::vector_db("Invalid Pinecone response: missing vectors object".to_owned())
        })?;

        let results = vectors_obj
            .iter()
            .map(|(id, data)| {
                let metadata = data.get("metadata").ok_or_else(|| {
                    Error::vector_db(format!(
                        "Invalid Pinecone vector '{id}': missing 'metadata' field"
                    ))
                })?;
                Ok(search_result_from_json_metadata(id.clone(), metadata, 1.0))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(results)
    }

    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Pinecone doesn't support listing; use zero vector search as workaround
        let collection_str = collection.to_string();
        let dimensions = self
            .collections
            .get(&collection_str)
            .map_or(EDGEVEC_DEFAULT_DIMENSIONS, |d| *d.value());

        let zero_vector = vec![0.0f32; dimensions];
        self.search_similar(collection, &zero_vector, limit, None)
            .await
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
