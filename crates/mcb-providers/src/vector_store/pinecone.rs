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
use reqwest::Client;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionInfo, Embedding, FileInfo, SearchResult};
use serde_json::Value;

use crate::constants::CONTENT_TYPE_JSON;
use crate::utils::{HttpResponseUtils, JsonExt};
use crate::vector_store::helpers::handle_vector_request_error;

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
        let mut builder = self
            .http_client
            .request(method, self.api_url(path))
            .header("Api-Key", &self.api_key)
            .header("Content-Type", CONTENT_TYPE_JSON)
            .timeout(self.timeout);

        if let Some(payload) = body {
            builder = builder.json(&payload);
        }

        let response = builder
            .send()
            .await
            .map_err(|e| handle_vector_request_error(e, self.timeout, "Pinecone", path))?;

        HttpResponseUtils::check_and_parse(response, "Pinecone").await
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
            file_path: metadata.string_or("file_path", ""),
            start_line: metadata
                .opt_u64("start_line")
                .or_else(|| metadata.opt_u64("line_number"))
                .unwrap_or(0) as u32,
            content: metadata.string_or("content", ""),
            score,
            language: metadata.string_or("language", "unknown"),
        }
    }
}

#[async_trait]
impl VectorStoreAdmin for PineconeVectorStoreProvider {
    async fn collection_exists(&self, name: &str) -> Result<bool> {
        Ok(self.collections.contains_key(name))
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, Value>> {
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
        stats.insert("collection".to_string(), serde_json::json!(collection));
        stats.insert(
            "provider".to_string(),
            serde_json::json!(self.provider_name()),
        );

        match response {
            Ok(data) => {
                if let Some(namespaces) = data.get("namespaces") {
                    if let Some(ns) = namespaces.get(collection) {
                        if let Some(count) = ns.get("vectorCount") {
                            stats.insert("vectors_count".to_string(), count.clone());
                        }
                    }
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

    async fn flush(&self, _collection: &str) -> Result<()> {
        // Pinecone writes are immediately consistent
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "pinecone"
    }
}

#[async_trait]
impl VectorStoreProvider for PineconeVectorStoreProvider {
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()> {
        if self.collections.contains_key(name) {
            return Err(Error::vector_db(format!(
                "Collection '{}' already exists",
                name
            )));
        }
        // Pinecone uses namespaces within an index; creation is implicit on first upsert
        self.collections.insert(name.to_string(), dimensions);
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let payload = serde_json::json!({
            "deleteAll": true,
            "namespace": name
        });

        self.request(reqwest::Method::POST, "/vectors/delete", Some(payload))
            .await?;

        self.collections.remove(name);
        Ok(())
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }

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
                    "namespace": collection
                });

                self.request(reqwest::Method::POST, "/vectors/upsert", Some(payload))
                    .await?;

                pinecone_vectors = Vec::new();
            }
        }

        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let mut payload = serde_json::json!({
            "vector": query_vector,
            "topK": limit,
            "namespace": collection,
            "includeMetadata": true
        });

        if let Some(filter_str) = filter {
            if let Ok(filter_val) = serde_json::from_str::<Value>(filter_str) {
                payload["filter"] = filter_val;
            }
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

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let payload = serde_json::json!({
            "ids": ids,
            "namespace": collection
        });

        self.request(reqwest::Method::POST, "/vectors/delete", Some(payload))
            .await?;

        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let payload = serde_json::json!({
            "ids": ids,
            "namespace": collection
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
                            file_path: metadata.string_or("file_path", ""),
                            start_line: metadata.opt_u64("start_line").unwrap_or(0) as u32,
                            content: metadata.string_or("content", ""),
                            score: 1.0,
                            language: metadata.string_or("language", "unknown"),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }

    async fn list_vectors(&self, collection: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Pinecone doesn't support listing; use zero vector search as workaround
        let dimensions = self
            .collections
            .get(collection)
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
            .map(|entry| CollectionInfo::new(entry.key().clone(), 0, 0, None, self.provider_name()))
            .collect();
        Ok(collections)
    }

    async fn list_file_paths(&self, collection: &str, limit: usize) -> Result<Vec<FileInfo>> {
        let results = self.list_vectors(collection, limit).await?;

        let mut file_map: HashMap<String, (u32, String)> = HashMap::new();
        for result in &results {
            let entry = file_map
                .entry(result.file_path.clone())
                .or_insert((0, result.language.clone()));
            entry.0 += 1;
        }

        let files = file_map
            .into_iter()
            .map(|(path, (chunk_count, language))| FileInfo::new(path, chunk_count, language, None))
            .collect();

        Ok(files)
    }

    async fn get_chunks_by_file(
        &self,
        collection: &str,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let filter = serde_json::json!({
            "file_path": { "$eq": file_path }
        });

        let dimensions = self
            .collections
            .get(collection)
            .map(|d| *d.value())
            .unwrap_or(1536);

        let zero_vector = vec![0.0f32; dimensions];

        let payload = serde_json::json!({
            "vector": zero_vector,
            "topK": 100,
            "namespace": collection,
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

use mcb_application::ports::registry::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

/// Factory function for creating Pinecone vector store provider instances.
fn pinecone_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    use crate::embedding::helpers::{DEFAULT_EMBEDDING_TIMEOUT, http::create_default_client};

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
        DEFAULT_EMBEDDING_TIMEOUT,
        http_client,
    )))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static PINECONE_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "pinecone",
    description: "Pinecone cloud vector database (managed, serverless)",
    factory: pinecone_factory,
};
