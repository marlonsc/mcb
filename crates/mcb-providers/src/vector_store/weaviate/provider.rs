//! Provider operations for Weaviate vector store provider.

use std::collections::HashMap;

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::VectorStoreProvider;
use mcb_domain::value_objects::{CollectionId, Embedding, SearchResult};
use mcb_utils::constants::vector_store::{WEAVIATE_BATCH_SIZE, WEAVIATE_DISTANCE_METRIC};
use mcb_utils::utils::id;
use serde_json::Value;

use super::WeaviateVectorStoreProvider;
use crate::utils::vector_store::search_result_from_json_metadata;

#[async_trait]
impl VectorStoreProvider for WeaviateVectorStoreProvider {
    async fn create_collection(&self, name: &CollectionId, dimensions: usize) -> Result<()> {
        let name_str = name.to_string();
        if self.collections.contains_key(&name_str) {
            return Err(Error::vector_db(format!(
                "Collection '{name}' already exists"
            )));
        }
        let class = Self::class_name(name);
        let body = serde_json::json!({
            "class": class,
            "vectorizer": "none",
            "vectorIndexConfig": { "distance": WEAVIATE_DISTANCE_METRIC },
        });
        self.request(reqwest::Method::POST, "/v1/schema", Some(body))
            .await?;
        self.collections.insert(name_str, dimensions);
        Ok(())
    }

    async fn delete_collection(&self, name: &CollectionId) -> Result<()> {
        let class = Self::class_name(name);
        self.request(
            reqwest::Method::DELETE,
            &format!("/v1/schema/{class}"),
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
            mcb_domain::warn!("weaviate", "insert_vectors called with empty vectors array");
            return Err(Error::vector_db(
                "Cannot insert empty vectors array".to_owned(),
            ));
        }
        if vectors.len() != metadata.len() {
            return Err(Error::vector_db(format!(
                "Vectors/metadata length mismatch: vectors={}, metadata={}",
                vectors.len(),
                metadata.len()
            )));
        }
        let class = Self::class_name(collection);

        let mut ids = Vec::with_capacity(vectors.len());
        let mut objects = Vec::with_capacity(vectors.len());

        for (i, (embedding, meta)) in vectors.iter().zip(metadata.iter()).enumerate() {
            let object_id = id::generate().to_string();
            objects.push(serde_json::json!({
                "class": class,
                "id": object_id,
                "vector": embedding.vector,
                "properties": meta,
            }));
            ids.push(object_id);

            if objects.len() >= WEAVIATE_BATCH_SIZE || i == vectors.len() - 1 {
                self.request(
                    reqwest::Method::POST,
                    "/v1/batch/objects",
                    Some(serde_json::json!({ "objects": objects })),
                )
                .await?;
                objects.clear();
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
        let class = Self::class_name(collection);
        let where_clause = match filter {
            Some(f) => Some(Self::where_clause(f)?),
            None => None,
        };
        let query =
            Self::build_get_query(&class, Some(query_vector), limit, where_clause.as_deref())?;
        self.run_get_query(&class, query).await
    }

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let class = Self::class_name(collection);
        for object_id in ids {
            self.request(
                reqwest::Method::DELETE,
                &format!("/v1/objects/{class}/{object_id}"),
                None,
            )
            .await?;
        }
        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &CollectionId,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        if ids.is_empty() {
            mcb_domain::warn!("weaviate", "get_vectors_by_ids called with empty ids array");
            return Err(Error::vector_db(
                "Cannot fetch vectors with empty ids array".to_owned(),
            ));
        }
        let class = Self::class_name(collection);
        let mut results = Vec::with_capacity(ids.len());
        for object_id in ids {
            let response = self
                .request(
                    reqwest::Method::GET,
                    &format!("/v1/objects/{class}/{object_id}"),
                    None,
                )
                .await?;
            let empty = serde_json::Value::Object(serde_json::Map::new());
            let properties = response.get("properties").unwrap_or(&empty);
            let resolved_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(object_id)
                .to_owned();
            results.push(search_result_from_json_metadata(
                resolved_id,
                properties,
                1.0,
            ));
        }
        Ok(results)
    }

    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let class = Self::class_name(collection);
        let query = Self::build_get_query(&class, None, limit, None)?;
        self.run_get_query(&class, query).await
    }
}
