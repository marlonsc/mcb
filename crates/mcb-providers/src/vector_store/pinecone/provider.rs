//! Provider operations for Pinecone vector store provider.

use std::collections::HashMap;

use async_trait::async_trait;
use mcb_domain::error::Error;
use mcb_domain::error::Result;
use mcb_domain::ports::VectorStoreProvider;
use mcb_domain::utils::id;
use mcb_domain::value_objects::{CollectionId, Embedding, SearchResult};
use serde_json::Value;

use super::PineconeVectorStoreProvider;

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
        self.request(
            reqwest::Method::POST,
            "/vectors/delete",
            Some(serde_json::json!({ "deleteAll": true, "namespace": &name_str })),
        )
        .await?;

        self.collections.remove(&name_str);
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
        if vectors.len() != metadata.len() {
            return Err(Error::vector_db(format!(
                "Vectors/metadata length mismatch: vectors={}, metadata={}",
                vectors.len(),
                metadata.len()
            )));
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
                self.request(
                    reqwest::Method::POST,
                    "/vectors/upsert",
                    Some(serde_json::json!({
                        "vectors": pinecone_vectors,
                        "namespace": collection_str
                    })),
                )
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

        self.query_match_results(payload).await
    }

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let collection_str = collection.to_string();

        self.request(
            reqwest::Method::POST,
            "/vectors/delete",
            Some(serde_json::json!({ "ids": ids, "namespace": collection_str })),
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
            mcb_domain::warn!("pinecone", "get_vectors_by_ids called with empty ids array");
            return Err(Error::vector_db(
                "Cannot fetch vectors with empty ids array".to_owned(),
            ));
        }
        let collection_str = collection.to_string();

        let response = self
            .request(
                reqwest::Method::POST,
                "/vectors/fetch",
                Some(serde_json::json!({ "ids": ids, "namespace": collection_str })),
            )
            .await?;

        let vectors_obj =
            Self::extract_json_field(&response, "vectors", Value::as_object, "response", "object")?;

        let results = vectors_obj
            .iter()
            .map(|(id, data)| {
                let metadata = Self::extract_json_field(
                    data,
                    "metadata",
                    Some,
                    &format!("vector '{id}'"),
                    "value",
                )?;
                Ok(
                    crate::utils::vector_store::search_result_from_json_metadata(
                        id.clone(),
                        metadata,
                        1.0,
                    ),
                )
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
        let dimensions = self.collection_dimensions(&collection_str);

        let zero_vector = vec![0.0f32; dimensions];
        self.search_similar(collection, &zero_vector, limit, None)
            .await
    }
}
