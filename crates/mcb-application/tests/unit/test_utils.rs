//! Shared test utilities and mocks for mcb-application tests

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::Result;
use mcb_domain::ports::providers::*;
use mcb_domain::value_objects::CollectionId;
use mcb_domain::value_objects::{CollectionInfo, Embedding, FileInfo, SearchResult};
use serde_json::Value;
use tokio::sync::Mutex;

/// Mock Cache Provider for testing
#[derive(Debug)]
pub struct TestCacheProvider;

impl Default for TestCacheProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TestCacheProvider {
    /// Create a new mock cache provider
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CacheProvider for TestCacheProvider {
    async fn get_json(&self, _key: &str) -> Result<Option<String>> {
        Ok(None)
    }
    async fn set_json(&self, _key: &str, _value: &str, _config: CacheEntryConfig) -> Result<()> {
        Ok(())
    }
    async fn delete(&self, _key: &str) -> Result<bool> {
        Ok(true)
    }
    async fn exists(&self, _key: &str) -> Result<bool> {
        Ok(false)
    }
    async fn clear(&self) -> Result<()> {
        Ok(())
    }
    async fn stats(&self) -> Result<CacheStats> {
        Ok(CacheStats::default())
    }
    async fn size(&self) -> Result<usize> {
        Ok(0)
    }
    fn provider_name(&self) -> &str {
        "mock-cache"
    }
}

/// Mock Embedding Provider for testing
#[derive(Debug)]
pub struct TestEmbeddingProvider {
    /// Dimensions for the embeddings
    pub dimensions: usize,
}

impl TestEmbeddingProvider {
    /// Create a new mock embedding provider with given dimensions
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }
}

#[async_trait]
impl EmbeddingProvider for TestEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let embeddings = texts
            .iter()
            .map(|_| Embedding {
                vector: vec![0.1; self.dimensions],
                model: "mock-model".to_string(),
                dimensions: self.dimensions,
            })
            .collect();
        Ok(embeddings)
    }
    fn dimensions(&self) -> usize {
        self.dimensions
    }
    fn provider_name(&self) -> &str {
        "mock-embedding"
    }
}

// Mock Vector Store Provider

type VectorData = (Embedding, HashMap<String, Value>);
type StorageMap = HashMap<String, Vec<VectorData>>;

/// Mock Vector Store Provider that can store data in memory or return fixed results
#[derive(Debug, Default)]
pub struct TestVectorStoreProvider {
    /// In-memory storage for simple retrieval validation
    pub storage: Arc<Mutex<StorageMap>>,
    /// Fixed results to return if provided
    pub override_results: Arc<Mutex<Option<Vec<SearchResult>>>>,
}

impl TestVectorStoreProvider {
    /// Create a new mock vector store provider
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new mock vector store provider that always returns the given results
    pub fn with_results(results: Vec<SearchResult>) -> Self {
        Self {
            override_results: Arc::new(Mutex::new(Some(results))),
            ..Default::default()
        }
    }
}

#[async_trait]
impl VectorStoreAdmin for TestVectorStoreProvider {
    async fn collection_exists(&self, _name: &CollectionId) -> Result<bool> {
        Ok(true)
    }
    async fn get_stats(&self, _collection: &CollectionId) -> Result<HashMap<String, Value>> {
        Ok(HashMap::new())
    }
    async fn flush(&self, _collection: &CollectionId) -> Result<()> {
        Ok(())
    }
    fn provider_name(&self) -> &str {
        "mock-vector-store"
    }
}

#[async_trait]
impl VectorStoreBrowser for TestVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        Ok(vec![])
    }
    async fn list_file_paths(
        &self,
        _collection: &CollectionId,
        _limit: usize,
    ) -> Result<Vec<FileInfo>> {
        Ok(vec![])
    }
    async fn get_chunks_by_file(
        &self,
        _collection: &CollectionId,
        _file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        Ok(vec![])
    }
}

#[async_trait]
impl VectorStoreProvider for TestVectorStoreProvider {
    async fn create_collection(&self, _name: &CollectionId, _dimensions: usize) -> Result<()> {
        Ok(())
    }
    async fn delete_collection(&self, name: &CollectionId) -> Result<()> {
        let mut store = self.storage.lock().await;
        store.remove(name.as_str());
        Ok(())
    }
    async fn insert_vectors(
        &self,
        collection: &CollectionId,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>> {
        let mut store = self.storage.lock().await;
        let entry = store.entry(collection.to_string()).or_insert_with(Vec::new);
        for (v, m) in vectors.iter().zip(metadata.into_iter()) {
            entry.push((v.clone(), m));
        }
        let ids = (0..vectors.len()).map(|_| "mock-id".to_string()).collect();
        Ok(ids)
    }
    async fn search_similar(
        &self,
        collection: &CollectionId,
        _query_vector: &[f32],
        _limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let override_results = self.override_results.lock().await;
        if let Some(results) = &*override_results {
            return Ok(results.clone());
        }

        let store = self.storage.lock().await;
        if let Some(entries) = store.get(collection.as_str()) {
            let results = entries
                .iter()
                .map(|(_, meta)| SearchResult {
                    id: "mock-result".to_string(),
                    score: 0.9,
                    content: meta
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    file_path: meta
                        .get("file_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    start_line: meta.get("start_line").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    language: "rust".to_string(),
                })
                .collect();
            return Ok(results);
        }
        Ok(vec![])
    }
    async fn delete_vectors(&self, _collection: &CollectionId, _ids: &[String]) -> Result<()> {
        Ok(())
    }
    async fn get_vectors_by_ids(
        &self,
        _collection: &CollectionId,
        _ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        Ok(vec![])
    }
    async fn list_vectors(
        &self,
        _collection: &CollectionId,
        _limit: usize,
    ) -> Result<Vec<SearchResult>> {
        Ok(vec![])
    }
}
