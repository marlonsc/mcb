//! In-memory vector store provider implementation

use crate::error::{Error, Result};
use crate::providers::vector_store::VectorStoreProvider;
use crate::types::{Embedding, SearchResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory vector store provider for development and testing
pub struct InMemoryVectorStoreProvider {
    collections: Mutex<HashMap<String, Vec<(Embedding, HashMap<String, serde_json::Value>)>>>,
}

impl InMemoryVectorStoreProvider {
    /// Create a new in-memory vector store provider
    pub fn new() -> Self {
        Self {
            collections: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryVectorStoreProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStoreProvider for InMemoryVectorStoreProvider {
    async fn create_collection(&self, name: &str, _dimensions: usize) -> Result<()> {
        let mut collections = self.collections.lock().unwrap();
        if collections.contains_key(name) {
            return Err(Error::vector_db(format!("Collection '{}' already exists", name)));
        }
        collections.insert(name.to_string(), Vec::new());
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.lock().unwrap();
        collections.remove(name);
        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let collections = self.collections.lock().unwrap();
        Ok(collections.contains_key(name))
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        let mut collections = self.collections.lock().unwrap();
        let coll = collections.get_mut(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{}' not found", collection)))?;

        let mut ids = Vec::new();
        for (vector, meta) in vectors.iter().zip(metadata) {
            let id = format!("{}_{}", collection, coll.len());
            coll.push((vector.clone(), meta));
            ids.push(id);
        }

        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let collections = self.collections.lock().unwrap();
        let coll = collections.get(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{}' not found", collection)))?;

        // Simple cosine similarity search
        let mut results: Vec<_> = coll
            .iter()
            .enumerate()
            .map(|(i, (embedding, metadata))| {
                let similarity = cosine_similarity(query_vector, &embedding.vector);
                (similarity, i, embedding, metadata)
            })
            .collect();

        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        let search_results = results
            .into_iter()
            .map(|(score, _i, embedding, metadata)| {
                SearchResult {
                    file_path: metadata.get("file_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    line_number: metadata.get("start_line")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    content: metadata.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    score,
                    metadata: metadata.clone(),
                }
            })
            .collect();

        Ok(search_results)
    }

    async fn delete_vectors(&self, _collection: &str, _ids: &[String]) -> Result<()> {
        // Simple implementation for in-memory provider
        Ok(())
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        let collections = self.collections.lock().unwrap();
        let count = collections.get(collection)
            .map(|coll| coll.len())
            .unwrap_or(0);

        let mut stats = HashMap::new();
        stats.insert("count".to_string(), serde_json::json!(count));
        Ok(stats)
    }

    async fn flush(&self, _collection: &str) -> Result<()> {
        // In-memory, no-op
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "in-memory"
    }
}

/// Cosine similarity calculation
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}