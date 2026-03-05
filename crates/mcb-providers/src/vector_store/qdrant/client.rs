//! Qdrant vector search engine client.
//!
//! This module provides a complete implementation of the vector store provider interface
//! for Qdrant, supporting collection management, vector operations, and semantic search.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use reqwest::Client;
use serde_json::Value;

use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::{CollectionId, SearchResult};
use mcb_utils::constants::http::CONTENT_TYPE_JSON;

use crate::utils::http::{VectorDbRequestParams, send_vector_db_request};
use crate::utils::vector_store::search_result_from_json_metadata;
use mcb_utils::constants::http::{
    HTTP_HEADER_CONTENT_TYPE, PROVIDER_RETRY_BACKOFF_MS, PROVIDER_RETRY_COUNT,
};

/// Qdrant vector search engine client.
pub struct QdrantVectorStoreProvider {
    pub(super) base_url: String,
    pub(super) api_key: Option<String>,
    pub(super) timeout: Duration,
    pub(super) http_client: Client,
    pub(super) collections: Arc<DashMap<String, usize>>,
}

impl fmt::Debug for QdrantVectorStoreProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QdrantVectorStoreProvider")
            .field("base_url", &self.base_url)
            .field(
                "api_key",
                &self
                    .api_key
                    .as_ref()
                    .map(|_| mcb_utils::constants::REDACTED),
            )
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

    pub(super) fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub(super) fn collection_path(collection: &CollectionId) -> String {
        format!("/collections/{collection}")
    }

    pub(super) fn collection_points_path(collection: &CollectionId, operation: &str) -> String {
        format!("{}/points/{operation}", Self::collection_path(collection))
    }

    pub(super) async fn request_collection(
        &self,
        method: reqwest::Method,
        collection: &CollectionId,
        body: Option<Value>,
    ) -> Result<Value> {
        self.request(method, &Self::collection_path(collection), body)
            .await
    }

    pub(super) async fn request_points(
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

    pub(super) async fn request_points_operation(
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

    pub(super) fn map_result_items(
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

    pub(super) fn map_scored_search_results(response: &Value) -> Result<Vec<SearchResult>> {
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

    pub(super) async fn request(
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

    pub(super) fn point_to_search_result(item: &Value, score: f64) -> SearchResult {
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
