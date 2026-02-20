//! Context Service Use Case
//!
//! # Overview
//! The `ContextService` acts as the central orchestrator for all semantic code intelligence operations.
//! It bridges the gap between raw code assets and high-dimensional vector representations.
//!
//! # Key Features
//! - **Embedding Pipeline**: Tokenization and vector generation for code chunks.
//! - **Vector Lifecycle**: Managing collections in the underlying vector store (Qdrant, Chroma, etc.).
//! - **Semantic Search**: KNN retrieval with optional metadata filtering.
//!
//! # Architecture
//! Following Clean Architecture principles, this service implements the `ContextServiceInterface` port
//! and orchestrates interactions between:
//! - `EmbeddingProvider`: For generating vector representations.
//! - `VectorStoreProvider`: For persistent storage and retrieval of vectors.
//! - `CacheProvider`: For performance optimizations.

use std::collections::HashMap;
use std::path::{Component, Path};
use std::sync::Arc;

use mcb_domain::constants::keys::{
    METADATA_KEY_CONTENT, METADATA_KEY_END_LINE, METADATA_KEY_FILE_PATH, METADATA_KEY_LANGUAGE,
    METADATA_KEY_START_LINE,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::ports::{
    CacheEntryConfig, CacheProvider, ContextServiceInterface, EmbeddingProvider,
    VectorStoreProvider,
};
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
fn build_chunk_metadata(
    chunk: &CodeChunk,
    normalized_file_path: &str,
) -> HashMap<String, serde_json::Value> {
    HashMap::from([
        ("id".to_owned(), json!(chunk.id)),
        (
            METADATA_KEY_FILE_PATH.to_owned(),
            json!(normalized_file_path),
        ),
        (METADATA_KEY_CONTENT.to_owned(), json!(chunk.content)),
        (METADATA_KEY_START_LINE.to_owned(), json!(chunk.start_line)),
        (METADATA_KEY_END_LINE.to_owned(), json!(chunk.end_line)),
        (METADATA_KEY_LANGUAGE.to_owned(), json!(chunk.language)),
    ])
}

fn extract_vector_count(stats: &HashMap<String, serde_json::Value>) -> i64 {
    ["vectors_count", "row_count", "vector_count"]
        .iter()
        .find_map(|key| stats.get(*key).and_then(serde_json::Value::as_i64))
        .unwrap_or(0)
}

fn normalize_relative_file_path(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(mcb_domain::error::Error::invalid_argument(
            "chunk file_path cannot be empty".to_owned(),
        ));
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(mcb_domain::error::Error::invalid_argument(format!(
            "chunk file_path must be workspace-relative: '{trimmed}'"
        )));
    }

    let mut parts: Vec<String> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => parts.push(part.to_str().unwrap_or_default().to_owned()),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(mcb_domain::error::Error::invalid_argument(format!(
                    "chunk file_path must be normalized workspace-relative: '{trimmed}'"
                )));
            }
        }
    }

    if parts.is_empty() {
        return Err(mcb_domain::error::Error::invalid_argument(
            "chunk file_path cannot resolve to current directory".to_owned(),
        ));
    }

    Ok(parts.join("/"))
}

/// Implementation of the `ContextServiceInterface`.
///
/// Orchestrates `EmbeddingProvider`, `VectorStoreProvider`, and `CacheProvider` to deliver
/// low-latency semantic search capabilities.
pub struct ContextServiceImpl {
    cache: Arc<dyn CacheProvider>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextServiceImpl {
    /// Create new context service with injected dependencies
    pub fn new(
        cache: Arc<dyn CacheProvider>,
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
    ///
    /// # Errors
    ///
    /// Returns an error if the vector store query fails.
    async fn collection_exists(&self, collection: &CollectionId) -> Result<bool> {
        self.vector_store_provider
            .collection_exists(collection)
            .await
    }

    /// Set a cache value with default config
    ///
    /// # Errors
    ///
    /// Returns an error if the cache write fails.
    async fn cache_set(&self, key: &str, value: &str) -> Result<()> {
        self.cache
            .set_json(key, value, CacheEntryConfig::default())
            .await
    }
}

#[async_trait::async_trait]
impl ContextServiceInterface for ContextServiceImpl {
    /// # Errors
    ///
    /// Returns an error if the collection creation or cache initialization fails.
    async fn initialize(&self, collection: &CollectionId) -> Result<()> {
        let name = collection.to_string();
        // Create collection if it doesn't exist
        if !self.collection_exists(collection).await? {
            let dimensions = self.embedding_provider.dimensions();
            self.vector_store_provider
                .create_collection(collection, dimensions)
                .await?;
        }

        // Track initialization in cache
        self.cache_set(&cache_keys::collection(&name), "\"initialized\"")
            .await
    }

    /// # Errors
    ///
    /// Returns an error if embedding generation, vector insertion, or cache update fails.
    async fn store_chunks(&self, collection: &CollectionId, chunks: &[CodeChunk]) -> Result<()> {
        let name = collection.to_string();
        let mut texts = Vec::with_capacity(chunks.len());
        let mut metadata = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let normalized_file_path = normalize_relative_file_path(&chunk.file_path)?;
            texts.push(chunk.content.clone());
            metadata.push(build_chunk_metadata(chunk, &normalized_file_path));
        }

        // Generate embeddings for each chunk
        let embeddings = self.embedding_provider.embed_batch(&texts).await?;

        // Insert into vector store
        self.vector_store_provider
            .insert_vectors(collection, &embeddings, metadata)
            .await?;

        // Update collection metadata in cache
        self.cache_set(
            &cache_keys::collection_meta(&name),
            &chunks.len().to_string(),
        )
        .await
    }

    /// # Errors
    ///
    /// Returns an error if embedding generation or vector search fails.
    async fn search_similar(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Design: filters applied in-memory by SearchService. Push-down via metadata param when scaling.
        let query_embedding = self.embedding_provider.embed(query).await?;
        self.vector_store_provider
            .search_similar(collection, &query_embedding.vector, limit, None)
            .await
    }

    /// # Errors
    ///
    /// Returns an error if the embedding provider fails.
    async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }

    /// # Errors
    ///
    /// Returns an error if the vector store deletion or cache cleanup fails.
    async fn clear_collection(&self, collection: &CollectionId) -> Result<()> {
        let name = collection.to_string();
        // Delete collection from vector store if it exists
        if self.collection_exists(collection).await? {
            self.vector_store_provider
                .delete_collection(collection)
                .await?;
        }

        // Clear cache metadata
        self.cache.delete(&cache_keys::collection(&name)).await?;
        self.cache
            .delete(&cache_keys::collection_meta(&name))
            .await?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if listing collections or retrieving stats fails.
    async fn get_stats(&self) -> Result<(i64, i64)> {
        let collections = self.vector_store_provider.list_collections().await?;
        let collection_count = collections.len() as i64;

        let mut chunk_count = 0_i64;
        for collection in &collections {
            let stats = self.vector_store_provider.get_stats(&collection.id).await;
            chunk_count += match stats {
                Ok(stats) => {
                    let from_stats = extract_vector_count(&stats);
                    if from_stats > 0 {
                        from_stats
                    } else {
                        collection.vector_count as i64
                    }
                }
                Err(_) => collection.vector_count as i64,
            };
        }

        Ok((collection_count, chunk_count))
    }

    fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }
}
