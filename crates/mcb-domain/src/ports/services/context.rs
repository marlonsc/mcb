use crate::entities::CodeChunk;
use crate::error::Result;
use crate::value_objects::{CollectionId, Embedding, SearchResult};
use async_trait::async_trait;

/// Code Intelligence Service Interface
///
/// Defines the contract for semantic code understanding operations.
#[async_trait]
pub trait ContextServiceInterface: Send + Sync {
    /// Initialize the service for a collection
    async fn initialize(&self, collection: &CollectionId) -> Result<()>;

    /// Store code chunks in the repository
    async fn store_chunks(&self, collection: &CollectionId, chunks: &[CodeChunk]) -> Result<()>;

    /// Search for code similar to the query
    async fn search_similar(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Get embedding for text
    async fn embed_text(&self, text: &str) -> Result<Embedding>;

    /// Clear/delete a collection
    async fn clear_collection(&self, collection: &CollectionId) -> Result<()>;

    /// Get combined stats for the service
    async fn get_stats(&self) -> Result<(i64, i64)>;

    /// Get embedding dimensions
    fn embedding_dimensions(&self) -> usize;
}
