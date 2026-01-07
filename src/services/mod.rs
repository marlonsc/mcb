//! Business logic services

pub mod context;
pub mod indexing;
pub mod search;

// Re-export services from their respective modules
pub use context::ContextService;
pub use indexing::IndexingService;
pub use search::SearchService;

// Re-export for backward compatibility
pub use crate::core::types::{CodeChunk, SearchResult};
pub use crate::di::factory::ServiceProvider;
pub use crate::providers::{EmbeddingProvider, VectorStoreProvider};
