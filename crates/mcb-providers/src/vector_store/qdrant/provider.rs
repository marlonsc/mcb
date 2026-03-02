//! `VectorStoreProvider` implementation for Qdrant.

use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;

use mcb_domain::error::Result;
use mcb_domain::ports::VectorStoreProvider;
use mcb_domain::utils::id;
use mcb_domain::value_objects::{CollectionId, Embedding, SearchResult};

use super::QdrantVectorStoreProvider;

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
