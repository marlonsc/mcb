use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use dashmap::DashMap;
use reqwest::Client;
use serde_json::Value;

use mcb_domain::constants::http::CONTENT_TYPE_JSON;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::utils::id;
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};

use crate::constants::{
    HTTP_HEADER_CONTENT_TYPE, PROVIDER_RETRY_BACKOFF_MS, PROVIDER_RETRY_COUNT,
    STATS_FIELD_COLLECTION, STATS_FIELD_PROVIDER, STATS_FIELD_STATUS, STATS_FIELD_VECTORS_COUNT,
    STATUS_UNKNOWN, VECTOR_FIELD_FILE_PATH,
};
use crate::utils::http::{VectorDbRequestParams, send_vector_db_request};
use crate::utils::vector_store::search_result_from_json_metadata;
/// Qdrant vector search engine client.
pub struct QdrantVectorStoreProvider {
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
    http_client: Client,
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
    /// Create a new Qdrant vector store provider.
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

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn collection_path(collection: &CollectionId) -> String {
        format!("/collections/{collection}")
    }

    fn collection_points_path(collection: &CollectionId, operation: &str) -> String {
        format!("{}/points/{operation}", Self::collection_path(collection))
    }

    async fn request_collection(
        &self,
        method: reqwest::Method,
        collection: &CollectionId,
        body: Option<Value>,
    ) -> Result<Value> {
        self.request(method, &Self::collection_path(collection), body)
            .await
    }

    async fn request_points(
        &self,
        method: reqwest::Method,
        collection: &CollectionId,
        body: Option<Value>,
    ) -> Result<Value> {
        self.request(
            method,
            &format!("{}/points", Self::collection_path(collection)),
            body,
        )
        .await
    }

    async fn request_points_operation(
        &self,
        method: reqwest::Method,
        collection: &CollectionId,
        operation: &str,
        body: Option<Value>,
    ) -> Result<Value> {
        self.request(
            method,
            &Self::collection_points_path(collection, operation),
            body,
        )
        .await
    }

    fn map_result_items(
        items: &Value,
        warn_message: &'static str,
        warn_field: &'static str,
    ) -> Result<Vec<SearchResult>> {
        items
            .as_array()
            .ok_or_else(|| Error::vector_db(format!("Qdrant: {warn_message} ({warn_field})")))?
            .iter()
            .map(|item| Ok(Self::point_to_search_result(item, 1.0)))
            .collect()
    }

    fn map_scored_search_results(response: &Value) -> Result<Vec<SearchResult>> {
        response["result"]
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
            .collect()
    }

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
            retry_attempts: PROVIDER_RETRY_COUNT,
            retry_backoff_ms: PROVIDER_RETRY_BACKOFF_MS,
        })
        .await
    }

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
    async fn collection_exists(&self, name: &CollectionId) -> Result<bool> {
        match self
            .request_collection(reqwest::Method::GET, name, None)
            .await
        {
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
            .request_collection(reqwest::Method::GET, collection, None)
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
        let response = self
            .request_points_operation(
                reqwest::Method::POST,
                collection,
                "scroll",
                Some(serde_json::json!({
                    "filter": {
                        "must": [{
                            "key": VECTOR_FIELD_FILE_PATH,
                            "match": { "value": file_path }
                        }]
                    },
                    "limit": 100,
                    "with_payload": true
                })),
            )
            .await?;

        let mut results = Self::map_result_items(
            &response["result"]["points"],
            "payload missing or malformed, using empty default",
            "search_result.payload",
        )?;

        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}

#[async_trait]
impl VectorStoreProvider for QdrantVectorStoreProvider {
    async fn create_collection(&self, name: &CollectionId, dimensions: usize) -> Result<()> {
        self.request_collection(
            reqwest::Method::PUT,
            name,
            Some(serde_json::json!({
                "vectors": {
                    "size": dimensions,
                    "distance": crate::constants::QDRANT_DISTANCE_METRIC
                }
            })),
        )
        .await?;

        self.collections.insert(name.to_string(), dimensions);
        Ok(())
    }

    async fn delete_collection(&self, name: &CollectionId) -> Result<()> {
        self.request_collection(reqwest::Method::DELETE, name, None)
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

        self.request_points(
            reqwest::Method::PUT,
            collection,
            Some(serde_json::json!({ "points": points })),
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
            .request_points_operation(reqwest::Method::POST, collection, "search", Some(payload))
            .await?;

        let results = Self::map_scored_search_results(&response)?;

        Ok(results)
    }

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        self.request_points_operation(
            reqwest::Method::POST,
            collection,
            "delete",
            Some(serde_json::json!({ "points": ids })),
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

        let response = self
            .request_points(
                reqwest::Method::POST,
                collection,
                Some(serde_json::json!({
                    "ids": ids,
                    "with_payload": true
                })),
            )
            .await?;

        let results = Self::map_result_items(
            &response["result"],
            "vectors field missing or malformed, using empty default",
            "search_result.vectors",
        )?;

        Ok(results)
    }

    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let response = self
            .request_points_operation(
                reqwest::Method::POST,
                collection,
                "scroll",
                Some(serde_json::json!({
                    "limit": limit,
                    "with_payload": true
                })),
            )
            .await?;

        let results = Self::map_result_items(
            &response["result"]["points"],
            "ID extraction failed, using empty default",
            "search_result.id",
        )?;

        Ok(results)
    }
}

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
