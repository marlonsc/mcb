//! Pinecone Vector Store Provider
//!
//! Implements the VectorStoreProvider, VectorStoreAdmin, and VectorStoreBrowser ports
//! using Pinecone's cloud vector database REST API.
//!
//! Pinecone is a managed vector database optimized for machine learning applications.
//! This provider communicates via Pinecone's REST API using the reqwest HTTP client.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use dashmap::DashMap;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};
use reqwest::Client;
use serde_json::Value;

use crate::constants::CONTENT_TYPE_JSON;
use crate::provider_utils::{JsonRequestParams, send_json_request};
use crate::utils::http::RequestErrorKind;

/// Pinecone vector store provider
///
/// Implements the vector store domain ports using Pinecone's cloud REST API.
/// Supports index management, vector upsert, search, and metadata filtering.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_providers::vector_store::PineconeVectorStoreProvider;
/// use reqwest::Client;
/// use std::time::Duration;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::builder()
///         .timeout(Duration::from_secs(30))
///         .build()?;
///     let provider = PineconeVectorStoreProvider::new(
///         "your-api-key".to_string(),
///         "https://your-index-abcdef.svc.aped-1234.pinecone.io".to_string(),
///         Duration::from_secs(30),
///         client,
///     );
///     Ok(())
/// }
/// ```
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
    pub fn new(api_key: String, host: String, timeout: Duration, http_client: Client) -> Self {
        Self {
            api_key: api_key.trim().to_string(),
            host: host.trim_end_matches('/').to_string(),
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
            ("Api-Key", self.api_key.clone()),
            ("Content-Type", CONTENT_TYPE_JSON.to_string()),
        ];

        send_json_request(JsonRequestParams {
            client: &self.http_client,
            method,
            url: self.api_url(path),
            timeout: self.timeout,
            provider: "Pinecone",
            operation: path,
            kind: RequestErrorKind::VectorDb,
            headers: &headers,
            body: body.as_ref(),
        })
        .await
    }

    /// Convert Pinecone match result to domain SearchResult
    fn match_to_search_result(item: &Value, score: f64) -> SearchResult {
        let id = item["id"].as_str().unwrap_or("").to_string();
        let metadata = item
            .get("metadata")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        SearchResult {
            id,
            file_path: metadata
                .get("file_path")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            start_line: metadata
                .get("start_line")
                .and_then(Value::as_u64)
                .or_else(|| metadata.get("line_number").and_then(Value::as_u64))
                .unwrap_or(0) as u32,
            content: metadata
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            score,
            language: metadata
                .get("language")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_owned(),
        }
    }
}

#[async_trait]
impl VectorStoreAdmin for PineconeVectorStoreProvider {
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
            "collection".to_string(),
            serde_json::json!(collection.to_string()),
        );
        stats.insert(
            "provider".to_string(),
            serde_json::json!(self.provider_name()),
        );

        match response {
            Ok(data) => {
                if let Some(namespaces) = data.get("namespaces")
                    && let Some(ns) = namespaces.get(&collection_str)
                    && let Some(count) = ns.get("vectorCount")
                {
                    stats.insert("vectors_count".to_string(), count.clone());
                }
                stats.insert("status".to_string(), serde_json::json!("active"));
            }
            Err(_) => {
                stats.insert("status".to_string(), serde_json::json!("unknown"));
                stats.insert("vectors_count".to_string(), serde_json::json!(0));
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
impl VectorStoreProvider for PineconeVectorStoreProvider {
    async fn create_collection(&self, name: &CollectionId, dimensions: usize) -> Result<()> {
        let name_str = name.to_string();
        if self.collections.contains_key(&name_str) {
            return Err(Error::vector_db(format!(
                "Collection '{}' already exists",
                name
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
            return Ok(Vec::new());
        }
        let collection_str = collection.to_string();

        let mut ids = Vec::with_capacity(vectors.len());
        let mut pinecone_vectors = Vec::with_capacity(vectors.len());
        let batch_size = 100;

        for (i, (embedding, meta)) in vectors.iter().zip(metadata.iter()).enumerate() {
            let id = format!("vec_{}", uuid::Uuid::new_v4());
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
            Error::vector_db("Invalid Pinecone response: missing matches array".to_string())
        })?;

        let results = matches
            .iter()
            .map(|m| {
                let score = m["score"].as_f64().unwrap_or(0.0);
                Self::match_to_search_result(m, score)
            })
            .collect();

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
            return Ok(Vec::new());
        }
        let collection_str = collection.to_string();

        let payload = serde_json::json!({
            "ids": ids,
            "namespace": collection_str
        });

        let response = self
            .request(reqwest::Method::GET, "/vectors/fetch", Some(payload))
            .await?;

        let results = response["vectors"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(id, data)| {
                        let metadata = data
                            .get("metadata")
                            .cloned()
                            .unwrap_or(serde_json::json!({}));
                        SearchResult {
                            id: id.clone(),
                            file_path: metadata
                                .get("file_path")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_owned(),
                            start_line: metadata
                                .get("start_line")
                                .and_then(Value::as_u64)
                                .unwrap_or(0) as u32,
                            content: metadata
                                .get("content")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_owned(),
                            score: 1.0,
                            language: metadata
                                .get("language")
                                .and_then(Value::as_str)
                                .unwrap_or("unknown")
                                .to_owned(),
                        }
                    })
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
        // Pinecone doesn't support listing; use zero vector search as workaround
        let collection_str = collection.to_string();
        let dimensions = self
            .collections
            .get(&collection_str)
            .map(|d| *d.value())
            .unwrap_or(1536);

        let zero_vector = vec![0.0f32; dimensions];
        self.search_similar(collection, &zero_vector, limit, None)
            .await
    }
}

#[async_trait]
impl VectorStoreBrowser for PineconeVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let collections: Vec<CollectionInfo> = self
            .collections
            .iter()
            .map(|entry| {
                CollectionInfo::new(
                    CollectionId::from_name(entry.key()),
                    0,
                    0,
                    None,
                    self.provider_name(),
                )
            })
            .collect();
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
        let filter = serde_json::json!({
            "file_path": { "$eq": file_path }
        });

        let collection_str = collection.to_string();
        let dimensions = self
            .collections
            .get(&collection_str)
            .map(|d| *d.value())
            .unwrap_or(1536);

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

        let mut results: Vec<SearchResult> = response["matches"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|m| {
                        let score = m["score"].as_f64().unwrap_or(0.0);
                        Self::match_to_search_result(m, score)
                    })
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

/// Factory function for creating Pinecone vector store provider instances.
fn pinecone_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    use crate::utils::http::{DEFAULT_HTTP_TIMEOUT, create_default_client};

    let api_key = config
        .api_key
        .clone()
        .ok_or_else(|| "Pinecone requires api_key".to_string())?;
    let host = config
        .uri
        .clone()
        .ok_or_else(|| "Pinecone requires uri (index host URL)".to_string())?;
    let http_client = create_default_client()?;

    Ok(Arc::new(PineconeVectorStoreProvider::new(
        api_key,
        host,
        DEFAULT_HTTP_TIMEOUT,
        http_client,
    )))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static PINECONE_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "pinecone",
    description: "Pinecone cloud vector database (managed, serverless)",
    factory: pinecone_factory,
};
