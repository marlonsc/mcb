//! # Application Layer
//!
//! Business logic services that orchestrate domain operations.
//!
//! This layer contains use case implementations that coordinate between
//! domain entities and ports. Services here contain no infrastructure
//! concerns - they depend only on domain ports (traits).
//!
//! ## Services
//!
//! | Service | Description |
//! |---------|-------------|
//! | [`ContextService`] | Code intelligence coordinator - embeddings and search |
//! | [`SearchService`] | Semantic code search across indexed collections |
//! | [`IndexingService`] | Codebase indexing with file discovery and chunking |
//!
//! ## Architecture
//!
//! ```text
//! Application Layer
//! ├── ContextService      # Core intelligence: embeddings + vector storage
//! ├── SearchService       # Query interface for semantic search
//! └── Indexing/
//!     ├── IndexingService        # Main indexing orchestration
//!     ├── FileDiscoveryService   # File system traversal
//!     └── ChunkingOrchestrator   # Batch code chunking
//! ```
//!
//! ## Dependency Direction
//!
//! Application services depend **only** on domain ports (traits), never on
//! concrete implementations. This enables:
//!
//! - **Testability**: Mock providers for unit testing
//! - **Flexibility**: Swap implementations without changing business logic
//! - **Clean boundaries**: Business logic isolated from infrastructure
//!
//! ## Example
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use mcp_context_browser::application::SearchService;
//! use mcp_context_browser::domain::ports::ContextServiceInterface;
//!
//! async fn search_code(
//!     context: Arc<dyn ContextServiceInterface>,
//! ) -> anyhow::Result<()> {
//!     let search = SearchService::new(context);
//!     let results = search.search("my-collection", "authentication logic", 10).await?;
//!     for result in results {
//!         println!("{}: {:.2}", result.file_path, result.score);
//!     }
//!     Ok(())
//! }
//! ```

pub mod admin;
pub mod context;
pub mod indexing;
pub mod search;

// Re-export services for convenient access
pub use context::ContextService;
pub use indexing::{
    BatchChunkResult, ChunkingConfig, ChunkingOrchestrator, DiscoveryOptions, DiscoveryResult,
    FileChunkResult, FileDiscoveryService, IndexingService,
};
pub use search::SearchService;
