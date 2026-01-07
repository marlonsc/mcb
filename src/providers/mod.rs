//! Provider interfaces and implementations

use crate::core::{error::Result, types::Embedding};
use async_trait::async_trait;

/// Embedding provider trait
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;

    /// Health check for the provider (default implementation provided)
    async fn health_check(&self) -> Result<()> {
        // Default implementation - try a simple embed operation
        self.embed("health check").await?;
        Ok(())
    }
}

/// Vector store provider trait
#[async_trait]
pub trait VectorStoreProvider: Send + Sync {
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()>;
    async fn delete_collection(&self, name: &str) -> Result<()>;
    async fn collection_exists(&self, name: &str) -> Result<bool>;
    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>>;
    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<crate::core::types::SearchResult>>;
    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()>;
    async fn get_stats(
        &self,
        collection: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>>;
    async fn flush(&self, collection: &str) -> Result<()>;
    fn provider_name(&self) -> &str;

    /// Health check for the provider (default implementation)
    async fn health_check(&self) -> Result<()> {
        // Default implementation - try a simple operation
        self.collection_exists("__health_check__").await?;
        Ok(())
    }
}

// Submodules
pub mod embedding;
pub mod routing;
pub mod vector_store;

// Re-export implementations
pub use embedding::NullEmbeddingProvider as MockEmbeddingProvider; // Backward compatibility
pub use vector_store::InMemoryVectorStoreProvider;
