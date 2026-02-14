//! Qdrant Vector Store Provider
//!
//! Implements the VectorStoreProvider, VectorStoreAdmin, and VectorStoreBrowser ports
//! using Qdrant's cloud and self-hosted vector database REST API.
//!
//! Qdrant is an open-source vector search engine with rich filtering and payload support.
//! This provider communicates via Qdrant's REST API using the reqwest HTTP client.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use dashmap::DashMap;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};
use reqwest::Client;
use serde_json::Value;

use crate::constants::CONTENT_TYPE_JSON;
use crate::provider_utils::{JsonRequestParams, send_json_request};
use crate::utils::http::RequestErrorKind;

/// Qdrant vector store provider
///
/// Implements the vector store domain ports using Qdrant's REST API.
/// Supports collection management, vector upsert, similarity search,
/// and advanced filtering via Qdrant's payload-based query system.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_providers::vector_store::QdrantVectorStoreProvider;
/// use reqwest::Client;
/// use std::time::Duration;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::builder()
///         .timeout(Duration::from_secs(30))
///         .build()?;
///     let provider = QdrantVectorStoreProvider::new(
///         "http://localhost:6333".to_string(),
///         None,
///         Duration::from_secs(30),
///         client,
///     );
///     Ok(())
/// }
/// ```
pub struct QdrantVectorStoreProvider {
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
    http_client: Client,
    /// Track collection dimensions locally
    collections: Arc<DashMap<String, usize>>,
}

impl QdrantVectorStoreProvider {
    /// Create a new Qdrant vector store provider
    ///
    /// # Arguments
    /// * `base_url` - Qdrant server URL (e.g., "http://localhost:6333")
    /// * `api_key` - Optional API key for Qdrant Cloud
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    pub fn new(
        base_url: String,
        api_key: Option<String>,
        timeout: Duration,
        http_client: Client,
    ) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.map(|k| k.trim().to_string()),
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
        let mut headers = vec![("Content-Type", CONTENT_TYPE_JSON.to_string())];

        if let Some(ref key) = self.api_key {
            headers.push(("api-key", key.clone()));
        }

        send_json_request(JsonRequestParams {
            client: &self.http_client,
            method,
            url: self.api_url(path),
            timeout: self.timeout,
            provider: "Qdrant",
            operation: path,
            kind: RequestErrorKind::VectorDb,
            headers: &headers,
            body: body.as_ref(),
        })
        .await
    }

    /// Convert Qdrant point result to domain SearchResult
    fn point_to_search_result(item: &Value, score: f64) -> SearchResult {
        let id = match &item["id"] {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => String::new(),
        };

        let payload = item
            .get("payload")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        SearchResult {
            id,
            file_path: payload
                .get("file_path")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            start_line: payload
                .get("start_line")
                .and_then(Value::as_u64)
                .or_else(|| payload.get("line_number").and_then(Value::as_u64))
                .unwrap_or(0) as u32,
            content: payload
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            score,
            language: payload
                .get("language")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_owned(),
        }
    }
}

#[async_trait]
impl VectorStoreAdmin for QdrantVectorStoreProvider {
    async fn collection_exists(&self, name: &CollectionId) -> Result<bool> {
        let response = self
            .request(reqwest::Method::GET, &format!("/collections/{name}"), None)
            .await;

        match response {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_stats(&self, collection: &CollectionId) -> Result<HashMap<String, Value>> {
        let mut stats = HashMap::new();
        stats.insert(
            "collection".to_string(),
            serde_json::json!(collection.to_string()),
        );
        stats.insert(
            "provider".to_string(),
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
                    if let Some(count) = result.get("vectors_count") {
                        stats.insert("vectors_count".to_string(), count.clone());
                    }
                    if let Some(status) = result.get("status") {
                        stats.insert("status".to_string(), status.clone());
                    }
                }
            }
            Err(_) => {
                stats.insert("status".to_string(), serde_json::json!("unknown"));
                stats.insert("vectors_count".to_string(), serde_json::json!(0));
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
impl VectorStoreProvider for QdrantVectorStoreProvider {
    async fn create_collection(&self, name: &CollectionId, dimensions: usize) -> Result<()> {
        let payload = serde_json::json!({
            "vectors": {
                "size": dimensions,
                "distance": "Cosine"
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
            let id = uuid::Uuid::new_v4().to_string();
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
            .map(|arr| {
                arr.iter()
                    .map(|item| {
                        let score = item["score"].as_f64().unwrap_or(0.0);
                        Self::point_to_search_result(item, score)
                    })
                    .collect()
            })
            .unwrap_or_default();

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

        let results = response["result"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|item| Self::point_to_search_result(item, 1.0))
                    .collect()
            })
            .unwrap_or_default();

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

        let results = response["result"]["points"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|item| Self::point_to_search_result(item, 1.0))
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }
}

#[async_trait]
impl VectorStoreBrowser for QdrantVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let response = self
            .request(reqwest::Method::GET, "/collections", None)
            .await?;

        let collections = response["result"]["collections"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|item| {
                        let name = item["name"].as_str().unwrap_or("").to_string();
                        CollectionInfo::new(name, 0, 0, None, self.provider_name())
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(collections)
    }

    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>> {
        let results = self.list_vectors(collection, limit).await?;
        Ok(super::helpers::build_file_info_from_results(results))
    }

    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let payload = serde_json::json!({
            "filter": {
                "must": [{
                    "key": "file_path",
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

        let mut results: Vec<SearchResult> = response["result"]["points"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|item| Self::point_to_search_result(item, 1.0))
                    .collect()
            })
            .unwrap_or_default();

        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use mcb_domain::registry::vector_store::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

use crate::constants::QDRANT_DEFAULT_PORT;

/// Factory function for creating Qdrant vector store provider instances.
fn qdrant_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    use crate::utils::http::{DEFAULT_HTTP_TIMEOUT, create_default_client};

    let base_url = config
        .uri
        .clone()
        .unwrap_or_else(|| format!("http://localhost:{QDRANT_DEFAULT_PORT}"));
    let api_key = config.api_key.clone();
    let http_client = create_default_client()?;

    Ok(Arc::new(QdrantVectorStoreProvider::new(
        base_url,
        api_key,
        DEFAULT_HTTP_TIMEOUT,
        http_client,
    )))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static QDRANT_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "qdrant",
    description: "Qdrant vector search engine (open-source, cloud and self-hosted)",
    factory: qdrant_factory,
};
