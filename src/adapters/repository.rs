//! Repository pattern implementation for data access abstraction
//!
//! This module provides repository implementations following
//! the Repository pattern to separate data access logic from business logic.

pub mod chunk_repository;
pub mod search_repository;

// Re-export implementations
pub use chunk_repository::VectorStoreChunkRepository;
pub use search_repository::VectorStoreSearchRepository;

// Re-export domain types for backward compatibility and convenience
pub use crate::domain::types::{RepositoryStats, SearchStats};
pub use crate::domain::ports::repository::{ChunkRepository, SearchRepository};
