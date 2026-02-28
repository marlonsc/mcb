//!
//! **Documentation**: [docs/modules/application.md](../../../../docs/modules/application.md#use-cases)
//!
//! Indexing Service Use Case
//!
//! # Overview
//! The `IndexingService` manages the ingestion and processing of code assets into the semantic
//! context system. It handles the full lifecycle from file discovery to vector storage, ensuring
//! that the system's understanding of the codebase remains up-to-date.
//!
//! # Responsibilities
//! - **File Discovery**: Recursively scanning workspace directories while respecting ignore patterns.
//! - **Language-Aware Chunking**: Splitting code files into semantic chunks using AST-based strategies.
//! - **Incremental Indexing**: Optimizing ingestion by only processing changed files (via hash tracking).
//! - **Async Processing**: Executing long-running indexing tasks in the background to maintain responsiveness.
//! - **Event Publishing**: Notifying the system of indexing progress and completion.
//!
//! # Architecture
//! Implements `IndexingServiceInterface` and acts as a coordinator between:
//! - `LanguageChunkingProvider`: For parsing and splitting code.
//! - `ContextService`: For embedding and storing chunks.
//! - `FileHashRepository`: For change detection.
//! - `EventBusProvider`: For system-wide notifications.

use std::path::Path;
use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::{
    ContextServiceInterface, EventBusProvider, FileHashRepository, IndexingOperationsInterface,
    LanguageChunkingProvider,
};

mod deps;
mod discovery;
mod interface;
mod processing;
mod progress;
mod registry;

pub use deps::{IndexingServiceDeps, IndexingServiceWithHashDeps};
pub use processing::*;
pub use progress::IndexingProgress;

/// Indexing service implementation - orchestrates file discovery and chunking.
///
/// Supports async background indexing with granular progress tracking, event publishing,
/// and incremental updates via file hash verification.
#[derive(Clone)]
pub struct IndexingServiceImpl {
    context_service: Arc<dyn ContextServiceInterface>,
    language_chunker: Arc<dyn LanguageChunkingProvider>,
    indexing_ops: Arc<dyn IndexingOperationsInterface>,
    event_bus: Arc<dyn EventBusProvider>,
    file_hash_repository: Option<Arc<dyn FileHashRepository>>,
    supported_extensions: Vec<String>,
}

impl IndexingServiceImpl {
    /// Compute the relative path of a file within the workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if the file is not under the workspace root.
    pub fn workspace_relative_path(file_path: &Path, workspace_root: &Path) -> Result<String> {
        mcb_domain::utils::path::workspace_relative_path(file_path, workspace_root)
    }

    /// Create new indexing service with injected dependencies
    pub fn new(
        context_service: Arc<dyn ContextServiceInterface>,
        language_chunker: Arc<dyn LanguageChunkingProvider>,
        indexing_ops: Arc<dyn IndexingOperationsInterface>,
        event_bus: Arc<dyn EventBusProvider>,
        supported_extensions: Vec<String>,
    ) -> Self {
        Self {
            context_service,
            language_chunker,
            indexing_ops,
            event_bus,
            file_hash_repository: None,
            supported_extensions: Self::normalize_supported_extensions(supported_extensions),
        }
    }

    /// Create a new indexing service with file hash persistence enabled.
    #[must_use]
    pub fn new_with_file_hash_repository(deps: IndexingServiceWithHashDeps) -> Self {
        let IndexingServiceWithHashDeps {
            service,
            file_hash_repository,
        } = deps;
        Self {
            context_service: service.context_service,
            language_chunker: service.language_chunker,
            indexing_ops: service.indexing_ops,
            event_bus: service.event_bus,
            file_hash_repository: Some(file_hash_repository),
            supported_extensions: Self::normalize_supported_extensions(
                service.supported_extensions,
            ),
        }
    }

    fn normalize_supported_extensions(extensions: Vec<String>) -> Vec<String> {
        extensions
            .into_iter()
            .map(|ext| ext.trim().trim_start_matches('.').to_ascii_lowercase())
            .filter(|ext| !ext.is_empty())
            .collect()
    }
}
/// Result of processing a single file during indexing.
#[derive(Debug, Clone)]
pub enum ProcessResult {
    /// File was successfully indexed.
    Processed {
        /// Number of chunks created from this file.
        chunks: usize,
    },
    /// File was skipped because it hasn't changed.
    Skipped,
}
