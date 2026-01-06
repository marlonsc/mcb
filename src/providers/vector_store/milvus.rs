//! Milvus vector store provider implementation

use crate::error::{Error, Result};
use crate::providers::vector_store::VectorStoreProvider;
use crate::types::{Embedding, SearchResult};
use async_trait::async_trait;
use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::search::{SearchIter, SearchResult as MilvusSearchResult};
use std::collections::HashMap;
use std::sync::Arc;

/// Milvus vector store provider
pub struct MilvusVectorStoreProvider {
    client: Arc<Client>,
}

impl MilvusVectorStoreProvider {
    /// Create a new Milvus vector store provider
    pub async fn new(address: String, token: Option<String>) -> Result<Self> {
        let mut client = Client::new(&address)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to connect to Milvus: {}", e)))?;

        if let Some(token) = token {
            // Set authentication if token provided
            client.set_token(token);
        }

        Ok(Self {
            client: Arc::new(client),
        })
    }
}

#[async_trait]
impl VectorStoreProvider for MilvusVectorStoreProvider {
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()> {
        // Create schema for the collection
        let schema = CollectionSchemaBuilder::new(name, "Code embeddings collection")
            .add_field(FieldSchema::new_primary_field("id", milvus::schema::DataType::VarChar, 64))
            .add_field(FieldSchema::new_vector_field("vector", milvus::schema::DataType::FloatVector, dimensions))
            .add_field(FieldSchema::new_field("content", milvus::schema::DataType::VarChar, 65535))
            .add_field(FieldSchema::new_field("file_path", milvus::schema::DataType::VarChar, 1024))
            .add_field(FieldSchema::new_field("start_line", milvus::schema::DataType::Int64))
            .add_field(FieldSchema::new_field("end_line", milvus::schema::DataType::Int64))
            .build()
            .map_err(|e| Error::vector_db(format!("Failed to build schema: {}", e)))?;

        self.client
            .create_collection(schema, None)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to create collection: {}", e)))?;

        // Create index on vector field
        self.client
            .create_index(name, "vector", milvus::index::IndexType::IvfFlat, Some(1024))
            .await
            .map_err(|e| Error::vector_db(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.client
            .drop_collection(name)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to delete collection: {}", e)))?;
        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        match self.client.describe_collection(name).await {
            Ok(_) => Ok(true),
            Err(milvus::error::Error::CollectionNotExists(_)) => Ok(false),
            Err(e) => Err(Error::vector_db(format!("Failed to check collection: {}", e))),
        }
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        if vectors.len() != metadata.len() {
            return Err(Error::vector_db("Vectors and metadata count mismatch"));
        }

        let mut ids = Vec::new();
        let mut vector_data = Vec::new();
        let mut content_data = Vec::new();
        let mut file_path_data = Vec::new();
        let mut start_line_data = Vec::new();
        let mut end_line_data = Vec::new();

        for (i, (vector, meta)) in vectors.iter().zip(metadata).enumerate() {
            let id = format!("{}_{}", collection, i);
            ids.push(id.clone());

            vector_data.push(vector.vector.clone());
            content_data.push(meta.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string());
            file_path_data.push(meta.get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string());
            start_line_data.push(meta.get("start_line")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as i64);
            end_line_data.push(meta.get("end_line")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as i64);
        }

        // Insert data
        let mut columns = Vec::new();
        columns.push(FieldColumn::new("id", ids));
        columns.push(FieldColumn::new("vector", vector_data));
        columns.push(FieldColumn::new("content", content_data));
        columns.push(FieldColumn::new("file_path", file_path_data));
        columns.push(FieldColumn::new("start_line", start_line_data));
        columns.push(FieldColumn::new("end_line", end_line_data));

        self.client
            .insert(collection, "", columns)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to insert vectors: {}", e)))?;

        // Flush to make data searchable
        self.client
            .flush(collection)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to flush: {}", e)))?;

        Ok((0..vectors.len()).map(|i| format!("{}_{}", collection, i)).collect())
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let search_request = milvus::search::SearchRequestBuilder::new(collection, "vector", query_vector.to_vec())
            .with_limit(limit as u64)
            .with_output_fields(vec!["content", "file_path", "start_line", "end_line"])
            .build();

        let results = self.client
            .search(search_request)
            .await
            .map_err(|e| Error::vector_db(format!("Search failed: {}", e)))?;

        let search_results = results
            .into_iter()
            .map(|result| {
                SearchResult {
                    file_path: result.entity.get("file_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    line_number: result.entity.get("start_line")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as u32,
                    content: result.entity.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    score: result.score,
                    metadata: result.entity,
                }
            })
            .collect();

        Ok(search_results)
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        // Delete by expression (ID in list)
        let expr = format!("id in {}", serde_json::to_string(ids)
            .map_err(|e| Error::vector_db(format!("Failed to serialize IDs: {}", e)))?);

        self.client
            .delete(collection, &expr)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to delete vectors: {}", e)))?;

        Ok(())
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        let stats = self.client
            .get_collection_statistics(collection)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to get stats: {}", e)))?;

        let mut result = HashMap::new();
        result.insert("count".to_string(), serde_json::json!(stats.row_count));
        result.insert("segments".to_string(), serde_json::json!(stats.segment_count));

        Ok(result)
    }

    async fn flush(&self, collection: &str) -> Result<()> {
        self.client
            .flush(collection)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to flush: {}", e)))?;
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "milvus"
    }
}