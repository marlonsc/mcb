//! Dependency injection structures for `IndexingService`.
//!
//! This module defines the dependency containers used to construct
//! the `IndexingService` with all required ports and providers.

use std::sync::Arc;

use mcb_domain::ports::{
    ContextServiceInterface, EventBusProvider, FileHashRepository, IndexingOperationsInterface,
    LanguageChunkingProvider,
};

/// `IndexingServiceDeps` struct.
pub struct IndexingServiceDeps {
    /// Service for Context operations
    pub context_service: Arc<dyn ContextServiceInterface>,
    /// Chunker
    pub language_chunker: Arc<dyn LanguageChunkingProvider>,
    /// Indexing operations
    pub indexing_ops: Arc<dyn IndexingOperationsInterface>,
    /// Event bus
    pub event_bus: Arc<dyn EventBusProvider>,
    /// Supported file extensions
    pub supported_extensions: Vec<String>,
}

/// `IndexingServiceWithHashDeps` struct.
pub struct IndexingServiceWithHashDeps {
    /// The nested dependencies
    pub service: IndexingServiceDeps,
    /// The file hash repository
    pub file_hash_repository: Arc<dyn FileHashRepository>,
}
