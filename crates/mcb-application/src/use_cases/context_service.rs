//! Context Service Use Case
//!
//! Application service for code intelligence and semantic operations.
//! Orchestrates embeddings, vector storage, and caching for semantic code understanding.

use std::collections::HashMap;
use std::sync::Arc;

use mcb_domain::constants::keys::{
    METADATA_KEY_END_LINE, METADATA_KEY_FILE_PATH, METADATA_KEY_START_LINE,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::{CacheEntryConfig, EmbeddingProvider, VectorStoreProvider};
use mcb_domain::ports::services::ContextServiceInterface;
use mcb_domain::value_objects::{CollectionId, Embedding, SearchResult};
use serde_json::json;

/// Cache key helpers for collection management
mod cache_keys {
    #[inline]
    pub fn collection(name: &str) -> String {
        format!("collection:{name}")
    }

    #[inline]
    pub fn collection_meta(name: &str) -> String {
        format!("collection:{name}:meta")
    }
}

/// Build metadata map from a code chunk
fn build_chunk_metadata(chunk: &CodeChunk) -> HashMap<String, serde_json::Value> {
    HashMap::from([
        ("id".to_string(), json!(chunk.id)),
        (METADATA_KEY_FILE_PATH.to_string(), json!(chunk.file_path)),
        ("content".to_string(), json!(chunk.content)),
        (METADATA_KEY_START_LINE.to_string(), json!(chunk.start_line)),
        (METADATA_KEY_END_LINE.to_string(), json!(chunk.end_line)),
        ("language".to_string(), json!(chunk.language)),
    ])
}

/// Context service implementation - manages embeddings and vector storage
pub struct ContextServiceImpl {
    cache: Arc<dyn mcb_domain::ports::providers::cache::CacheProvider>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextServiceImpl {
    /// Create new context service with injected dependencies
    pub fn new(
        cache: Arc<dyn mcb_domain::ports::providers::cache::CacheProvider>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            cache,
            embedding_provider,
            vector_store_provider,
        }
    }

    /// Check if collection exists in vector store
    async fn collection_exists(&self, collection: &CollectionId) -> Result<bool> {
        self.vector_store_provider
            .collection_exists(collection)
            .await
    }

    /// Set a cache value with default config
    async fn cache_set(&self, key: &str, value: &str) -> Result<()> {
        self.cache
            .set_json(key, value, CacheEntryConfig::default())
            .await
    }
}

#[async_trait::async_trait]
impl ContextServiceInterface for ContextServiceImpl {
    async fn initialize(&self, collection: &CollectionId) -> Result<()> {
        let name = collection.as_str();
        // Create collection if it doesn't exist
        if !self.collection_exists(collection).await? {
            let dimensions = self.embedding_provider.dimensions();
            self.vector_store_provider
                .create_collection(collection, dimensions)
                .await?;
        }

        // Track initialization in cache
        self.cache_set(&cache_keys::collection(name), "\"initialized\"")
            .await
    }

    async fn store_chunks(&self, collection: &CollectionId, chunks: &[CodeChunk]) -> Result<()> {
        let name = collection.as_str();
        // Generate embeddings for each chunk
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding_provider.embed_batch(&texts).await?;

        // Build metadata for each chunk
        let metadata: Vec<_> = chunks.iter().map(build_chunk_metadata).collect();

        // Insert into vector store
        self.vector_store_provider
            .insert_vectors(collection, &embeddings, metadata)
            .await?;

        // Update collection metadata in cache
        self.cache_set(
            &cache_keys::collection_meta(name),
            &chunks.len().to_string(),
        )
        .await
    }

    async fn search_similar(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedding_provider.embed(query).await?;
        self.vector_store_provider
            .search_similar(collection, &query_embedding.vector, limit, None)
            .await
    }

    async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }

    async fn clear_collection(&self, collection: &CollectionId) -> Result<()> {
        let name = collection.as_str();
        // Delete collection from vector store if it exists
        if self.collection_exists(collection).await? {
            self.vector_store_provider
                .delete_collection(collection)
                .await?;
        }

        // Clear cache metadata
        self.cache.delete(&cache_keys::collection(name)).await?;
        self.cache
            .delete(&cache_keys::collection_meta(name))
            .await?;
        Ok(())
    }

    async fn get_stats(&self) -> Result<(i64, i64)> {
        Ok((0, 0))
    }

    fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }
}
