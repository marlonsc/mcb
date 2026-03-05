//! Memory service types and implementation.

use std::sync::Arc;

use mcb_domain::ports::{EmbeddingProvider, MemoryRepository, VectorStoreProvider};

/// Hybrid memory service combining relational metadata with semantic vector search.
///
/// Implements a sophisticated RAG (Retrieval-Augmented Generation) pipeline using
/// Reciprocal Rank Fusion (RRF) to merge lexically precise matches (`SQLite` FTS)
/// with semantically relevant results (Vector Store).
pub struct MemoryServiceImpl {
    pub(super) project_id: String,
    pub(super) repository: Arc<dyn MemoryRepository>,
    pub(super) embedding_provider: Arc<dyn EmbeddingProvider>,
    pub(super) vector_store: Arc<dyn VectorStoreProvider>,
}

impl MemoryServiceImpl {
    /// Initializes the hybrid memory service with repository, embedding, and vector store providers.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project identifier for scoping observations and memories.
    /// * `repository` - SQLite-backed repository for metadata storage and full-text search.
    /// * `embedding_provider` - Provider for generating vector embeddings from content.
    /// * `vector_store` - Vector store for semantic similarity search and RAG operations.
    ///
    /// The service implements a hybrid search strategy combining full-text search (FTS)
    /// with vector similarity using reciprocal rank fusion (RRF) for balanced relevance.
    pub fn new(
        project_id: String,
        repository: Arc<dyn MemoryRepository>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            project_id,
            repository,
            embedding_provider,
            vector_store,
        }
    }
}
