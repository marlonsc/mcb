//!
//! **Documentation**: [docs/modules/application.md](../../../../docs/modules/application.md#use-cases)
//!
//! Memory Service Use Case
//!
//! # Overview
//! The `MemoryService` implements a comprehensive system for storing, retrieving, and analyzing
//! observations and long-term memory. It acts as the "brain" of the system, allowing agents
//! to recall past context, decisions, and error patterns.
//!
//! # Responsibilities
//! - **Hybrid Storage**: Persisting observations in both a relational DB (`SQLite`) for metadata/FTS
//!   and a Vector Store for semantic similarity.
//! - **Hybrid Search**: Combining keyword-based (FTS) and semantic (Vector) search results using
//!   Reciprocal Rank Fusion (RRF) for high-quality recall.
//! - **Timeline Management**: Retrieving observations in chronological order to reconstruct context.
//! - **Pattern Recognition**: Storing and retrieving error patterns to avoid repeating mistakes.
//! - **Session Summarization**: Compiling and storing high-level summaries of agent sessions.
//!
//! # Architecture
//! Implements `MemoryServiceInterface` and coordinates:
//! - `MemoryRepository`: For precise storage and FTS.
//! - `VectorStoreProvider`: For fuzzy semantic search.
//! - `EmbeddingProvider`: For generating vector representations of memory content.

use std::sync::Arc;

use mcb_domain::ports::{EmbeddingProvider, MemoryRepository, VectorStoreProvider};

mod helpers;
mod interface;
mod observation;
mod registry;
mod search;
mod session;

/// Hybrid memory service combining relational metadata with semantic vector search.
///
/// Implements a sophisticated RAG (Retrieval-Augmented Generation) pipeline using
/// Reciprocal Rank Fusion (RRF) to merge lexically precise matches (`SQLite` FTS)
/// with semantically relevant results (Vector Store).
pub struct MemoryServiceImpl {
    project_id: String,
    repository: Arc<dyn MemoryRepository>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store: Arc<dyn VectorStoreProvider>,
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
