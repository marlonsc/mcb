//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#vector-store-providers)
//!
//! Qdrant Vector Store Provider
//!
//! Implements the `VectorStoreProvider` port using Qdrant's cloud and self-hosted
//! vector database REST API.
//!
//! Qdrant is an open-source vector search engine with rich filtering and payload support.
//! This provider communicates via Qdrant's REST API using the reqwest HTTP client.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use dashmap::DashMap;
use mcb_domain::constants::http::CONTENT_TYPE_JSON;
use mcb_domain::error::{Error, Result};
use mcb_domain::utils::id;

use crate::constants::{
    HTTP_HEADER_CONTENT_TYPE, STATS_FIELD_COLLECTION, STATS_FIELD_PROVIDER, STATS_FIELD_STATUS,
    STATS_FIELD_VECTORS_COUNT, STATUS_UNKNOWN, VECTOR_FIELD_FILE_PATH,
    VECTOR_STORE_RETRY_BACKOFF_SECS, VECTOR_STORE_RETRY_COUNT,
};
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};
use reqwest::Client;
use serde_json::Value;

use crate::utils::http::{VectorDbRequestParams, send_vector_db_request};
use crate::utils::vector_store::search_result_from_json_metadata;

/// Qdrant vector store provider
///
/// Implements the vector store domain ports using Qdrant's REST API.
/// Supports collection management, vector upsert, similarity search,
/// and advanced filtering via Qdrant's payload-based query system.
pub struct QdrantVectorStoreProvider {
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
    http_client: Client,
    /// Track collection dimensions locally
    collections: Arc<DashMap<String, usize>>,
}

impl fmt::Debug for QdrantVectorStoreProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QdrantVectorStoreProvider")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "REDACTED"))
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl QdrantVectorStoreProvider {
    /// Create a new Qdrant vector store provider
    ///
    /// # Arguments
    /// * `base_url` - Qdrant server URL (e.g., "<http://localhost:6333>")
    /// * `api_key` - Optional API key for Qdrant Cloud
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    #[must_use]
    pub fn new(
        base_url: &str,
        api_key: Option<String>,
        timeout: Duration,
        http_client: Client,
    ) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key: api_key.map(|k| k.trim().to_owned()),
            timeout,
            http_client,
            collections: Arc::new(DashMap::new()),
        }
    }

    /// Build a URL for the Qdrant API
    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Make an authenticated request to Qdrant
    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value> {
        let mut headers = vec![(HTTP_HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON.to_owned())];

        if let Some(ref key) = self.api_key {
            headers.push(("api-key", key.clone()));
        }

        send_vector_db_request(VectorDbRequestParams {
            client: &self.http_client,
            method,
            url: self.api_url(path),
            timeout: self.timeout,
            provider: "Qdrant",
            operation: path,
            headers: &headers,
            body: body.as_ref(),
            retry_attempts: VECTOR_STORE_RETRY_COUNT,
            retry_backoff_secs: VECTOR_STORE_RETRY_BACKOFF_SECS,
        })
        .await
    }

    /// Convert Qdrant point result to domain `SearchResult`
    fn point_to_search_result(item: &Value, score: f64) -> SearchResult {
        let id = match &item["id"] {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => String::new(),
        };
        let default_payload = serde_json::Value::Object(Default::default());
        let payload = item.get("payload").unwrap_or(&default_payload);
        search_result_from_json_metadata(id, payload, score)
    }
}

#[async_trait]
impl VectorStoreAdmin for QdrantVectorStoreProvider {
    // --- Admin Methods ---

    async fn collection_exists(&self, name: &CollectionId) -> Result<bool> {
        let response = self
            .request(reqwest::Method::GET, &format!("/collections/{name}"), None)
            .await;

        match response {
            Ok(_) => Ok(true),
            Err(e) if e.to_string().contains("(404)") => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn get_stats(&self, collection: &CollectionId) -> Result<HashMap<String, Value>> {
        let mut stats = HashMap::new();
        stats.insert(
            STATS_FIELD_COLLECTION.to_owned(),
            serde_json::json!(collection.to_string()),
        );
        stats.insert(
            STATS_FIELD_PROVIDER.to_owned(),
            serde_json::json!(self.provider_name()),
        );

        match self
            .request(
                reqwest::Method::GET,
                &format!("/collections/{collection}"),
                None,
            )
            .await
        {
            Ok(data) => {
                if let Some(result) = data.get("result") {
                    if let Some(count) = result.get(STATS_FIELD_VECTORS_COUNT) {
                        stats.insert(STATS_FIELD_VECTORS_COUNT.to_owned(), count.clone());
                    }
                    if let Some(status) = result.get("status") {
                        stats.insert(STATS_FIELD_STATUS.to_owned(), status.clone());
                    }
                }
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
        // Qdrant handles persistence automatically
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "qdrant"
    }
}

#[async_trait]
impl VectorStoreBrowser for QdrantVectorStoreProvider {
    // --- Browser Methods ---

    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let response = self
            .request(reqwest::Method::GET, "/collections", None)
            .await?;

        let collections = response["result"]["collections"]
            .as_array()
            .ok_or_else(|| {
                Error::vector_db(
                    "Qdrant list_collections: malformed response, missing collections array",
                )
            })?
            .iter()
            .map(|item| {
                let name = item["name"]
                    .as_str()
                    .ok_or_else(|| {
                        Error::vector_db("Qdrant list_collections: missing collection name")
                    })?
                    .to_owned();
                Ok(CollectionInfo::new(name, 0, 0, None, self.provider_name()))
            })
            .collect::<Result<Vec<_>>>()?;

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
        let payload = serde_json::json!({
            "filter": {
                "must": [{
                    "key": VECTOR_FIELD_FILE_PATH,
                    "match": { "value": file_path }
                }]
            },
            "limit": 100,
            "with_payload": true
        });

        let response = self
            .request(
                reqwest::Method::POST,
                &format!("/collections/{collection}/points/scroll"),
                Some(payload),
            )
            .await?;

        let mut results: Vec<SearchResult> = response["result"]["points"].as_array().map_or_else(
            || {
                mcb_domain::warn!(
                    "qdrant",
                    "payload missing or malformed, using empty default",
                    &"search_result.payload"
                );
                Vec::new()
            },
            |arr| {
                arr.iter()
                    .map(|item| Self::point_to_search_result(item, 1.0))
                    .collect()
            },
        );

        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}

#[async_trait]
impl VectorStoreProvider for QdrantVectorStoreProvider {
    // --- Provider Methods ---

    async fn create_collection(&self, name: &CollectionId, dimensions: usize) -> Result<()> {
        let payload = serde_json::json!({
            "vectors": {
                "size": dimensions,
                "distance": crate::constants::QDRANT_DISTANCE_METRIC
            }
        });

        self.request(
            reqwest::Method::PUT,
            &format!("/collections/{name}"),
            Some(payload),
        )
        .await?;

        self.collections.insert(name.to_string(), dimensions);
        Ok(())
    }

    async fn delete_collection(&self, name: &CollectionId) -> Result<()> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/collections/{name}"),
            None,
        )
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
            return Ok(Vec::new());
        }

        let mut ids = Vec::with_capacity(vectors.len());
        let mut points = Vec::with_capacity(vectors.len());

        for (embedding, meta) in vectors.iter().zip(metadata.iter()) {
            let id = id::generate().to_string();
            points.push(serde_json::json!({
                "id": id,
                "vector": embedding.vector,
                "payload": meta
            }));
            ids.push(id);
        }

        let payload = serde_json::json!({
            "points": points
        });

        self.request(
            reqwest::Method::PUT,
            &format!("/collections/{collection}/points"),
            Some(payload),
        )
        .await?;

        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let mut payload = serde_json::json!({
            "vector": query_vector,
            "limit": limit,
            "with_payload": true
        });

        if let Some(filter_str) = filter
            && let Ok(filter_val) = serde_json::from_str::<Value>(filter_str)
        {
            payload["filter"] = filter_val;
        }

        let response = self
            .request(
                reqwest::Method::POST,
                &format!("/collections/{collection}/points/search"),
                Some(payload),
            )
            .await?;

        let results = response["result"]
            .as_array()
            .ok_or_else(|| {
                Error::vector_db("Qdrant search: malformed response, missing result array")
            })?
            .iter()
            .map(|item| {
                let score = item["score"]
                    .as_f64()
                    .ok_or_else(|| Error::vector_db("Qdrant search: missing score in result"))?;
                Ok(Self::point_to_search_result(item, score))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(results)
    }

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let payload = serde_json::json!({
            "points": ids
        });

        self.request(
            reqwest::Method::POST,
            &format!("/collections/{collection}/points/delete"),
            Some(payload),
        )
        .await?;

        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &CollectionId,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let payload = serde_json::json!({
            "ids": ids,
            "with_payload": true
        });

        let response = self
            .request(
                reqwest::Method::POST,
                &format!("/collections/{collection}/points"),
                Some(payload),
            )
            .await?;

        let results = response["result"].as_array().map_or_else(
            || {
                mcb_domain::warn!(
                    "qdrant",
                    "vectors field missing or malformed, using empty default",
                    &"search_result.vectors"
                );
                Vec::new()
            },
            |arr| {
                arr.iter()
                    .map(|item| Self::point_to_search_result(item, 1.0))
                    .collect()
            },
        );

        Ok(results)
    }

    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let payload = serde_json::json!({
            "limit": limit,
            "with_payload": true
        });

        let response = self
            .request(
                reqwest::Method::POST,
                &format!("/collections/{collection}/points/scroll"),
                Some(payload),
            )
            .await?;

        let results = response["result"]["points"].as_array().map_or_else(
            || {
                mcb_domain::warn!(
                    "qdrant",
                    "ID extraction failed, using empty default",
                    &"search_result.id"
                );
                Vec::new()
            },
            |arr| {
                arr.iter()
                    .map(|item| Self::point_to_search_result(item, 1.0))
                    .collect()
            },
        );

        Ok(results)
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use crate::constants::QDRANT_DEFAULT_PORT;

crate::register_vector_store_provider!(
    qdrant_factory,
    config,
    QDRANT_PROVIDER,
    "qdrant",
    "Qdrant vector search engine (open-source, cloud and self-hosted)",
    {
        use crate::utils::http::{DEFAULT_HTTP_TIMEOUT, create_default_client};

        let base_url = config
            .uri
            .clone()
            .unwrap_or_else(|| format!("http://localhost:{QDRANT_DEFAULT_PORT}"));
        let api_key = config.api_key.clone();
        let http_client = create_default_client()?;

        Ok(std::sync::Arc::new(QdrantVectorStoreProvider::new(
            &base_url,
            api_key,
            DEFAULT_HTTP_TIMEOUT,
            http_client,
        )))
    }
);
