//! Indexing service types and implementation.

use std::path::Path;
use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::{
    ContextServiceInterface, EventBusProvider, FileHashRepository, IndexingOperationsInterface,
    LanguageChunkingProvider,
};

/// Constructor dependency bundle for `IndexingServiceImpl`.
pub struct IndexingServiceDeps {
    /// Embedding pipeline and chunk persistence.
    pub context_service: Arc<dyn ContextServiceInterface>,
    /// AST-based language chunking.
    pub language_chunker: Arc<dyn LanguageChunkingProvider>,
    /// Async operation tracking.
    pub indexing_ops: Arc<dyn IndexingOperationsInterface>,
    /// Domain event publishing.
    pub event_bus: Arc<dyn EventBusProvider>,
    /// File extensions to index.
    pub supported_extensions: Vec<String>,
}

/// Extended dependency bundle that enables incremental indexing via file hashes.
pub struct IndexingServiceWithHashDeps {
    /// Core service dependencies.
    pub service: IndexingServiceDeps,
    /// Hash repository for change detection.
    pub file_hash_repository: Arc<dyn FileHashRepository>,
}

/// Indexing service implementation - orchestrates file discovery and chunking.
///
/// Supports async background indexing with granular progress tracking, event publishing,
/// and incremental updates via file hash verification.
#[derive(Clone)]
pub struct IndexingServiceImpl {
    pub(super) context_service: Arc<dyn ContextServiceInterface>,
    pub(super) language_chunker: Arc<dyn LanguageChunkingProvider>,
    pub(super) indexing_ops: Arc<dyn IndexingOperationsInterface>,
    pub(super) event_bus: Arc<dyn EventBusProvider>,
    pub(super) file_hash_repository: Option<Arc<dyn FileHashRepository>>,
    pub(super) supported_extensions: Vec<String>,
}

impl IndexingServiceImpl {
    /// Compute the relative path of a file within the workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if the file is not under the workspace root.
    pub fn workspace_relative_path(file_path: &Path, workspace_root: &Path) -> Result<String> {
        Ok(mcb_utils::utils::path::workspace_relative_path(
            file_path,
            workspace_root,
        )?)
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
