//! Domain Services
//!
//! Interfaces for domain services that encapsulate complex business logic.
//! These services define the core business operations that can be performed
//! within the domain.
//!
//! ## Domain Services
//!
//! | Service | Description |
//! |---------|-------------|
//! | [`CodeChunker`] | Service for parsing code into semantic chunks |
//! | [`ContextServiceInterface`] | High-level context and search operations |
//! | [`SearchServiceInterface`] | Semantic search operations |
//! | [`IndexingServiceInterface`] | Code indexing and ingestion operations |

/// Code chunking domain service interface
pub mod chunking;
/// Indexing domain service interface
pub mod indexing;
/// Memory domain service interface
pub mod memory;
// Re-export domain service interfaces from ports
pub use crate::ports::services::{
    ChunkingOrchestratorInterface, ContextServiceInterface, IndexingResult,
    IndexingServiceInterface, IndexingStatus, SearchServiceInterface,
};
pub use chunking::{ChunkingOptions, ChunkingResult, CodeChunker};
pub use memory::MemoryServiceInterface;
