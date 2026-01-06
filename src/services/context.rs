//! Context service for managing embeddings and vector storage

use crate::error::{Error, Result};
use crate::providers::{EmbeddingProvider, VectorStore, InMemoryVectorStore, MockEmbeddingProvider};
use crate::types::{CodeChunk, Embedding, SearchResult};
use std::collections::HashMap;
use std::sync::Arc;

/// Context service that orchestrates embedding and vector storage operations
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store: Arc<dyn VectorStore>,
}

impl ContextService {
    /// Create a new context service with default providers (MVP)
    pub fn new() -> Self {
        Self {
            embedding_provider: Arc::new(MockEmbeddingProvider::new()),
            vector_store: Arc::new(InMemoryVectorStore::new()),
        }
    }

    /// Generate embeddings for text
    pub async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }

    /// Generate embeddings for multiple texts
    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        self.embedding_provider.embed_batch(texts).await
    }

    /// Store code chunks in vector database
    pub async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embed_texts(&texts).await?;
        self.vector_store.store(collection, &embeddings).await
    }

    /// Search for similar code chunks
    pub async fn search_similar(&self, collection: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embed_text(query).await?;
        let results = self.vector_store.search(collection, &query_embedding.vector, limit).await?;

        let search_results = results.into_iter().map(|(score, _embedding)| {
            // For MVP, we return basic search results
            // In a real implementation, we'd have more metadata
            SearchResult {
                file_path: "unknown".to_string(),
                line_number: 0,
                content: query.to_string(),
                score,
                metadata: HashMap::new(),
            }
        }).collect();

        Ok(search_results)
    }

    /// Clear a collection
    pub async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.vector_store.clear(collection).await
    }

    /// Get embedding dimensions
    pub fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }
}

impl Default for ContextService {
    fn default() -> Self {
        Self::new()
    }
}