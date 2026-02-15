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

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use ignore::WalkBuilder;
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::admin::IndexingOperationsInterface;
use mcb_domain::ports::infrastructure::EventBusProvider;
use mcb_domain::ports::providers::LanguageChunkingProvider;
use mcb_domain::ports::repositories::FileHashRepository;
use mcb_domain::ports::services::{
    ContextServiceInterface, IndexingResult, IndexingServiceInterface,
};
use mcb_domain::value_objects::{CollectionId, OperationId};
use tracing::{error, info, warn};

use crate::constants::{PROGRESS_UPDATE_INTERVAL, SKIP_DIRS};

/// Accumulator for indexing progress and operational metrics.
///
/// Tracks the state of an active indexing operation, including success counts,
/// skipped files, and encountered errors for final reporting.
struct IndexingProgress {
    files_processed: usize,
    chunks_created: usize,
    files_skipped: usize,
    errors: Vec<String>,
}

impl IndexingProgress {
    fn new() -> Self {
        Self {
            files_processed: 0,
            chunks_created: 0,
            files_skipped: 0,
            errors: Vec::new(),
        }
    }

    fn with_counts(
        files_processed: usize,
        chunks_created: usize,
        files_skipped: usize,
        errors: Vec<String>,
    ) -> Self {
        Self {
            files_processed,
            chunks_created,
            files_skipped,
            errors,
        }
    }

    fn record_error(&mut self, context: &str, path: &Path, error: impl std::fmt::Display) {
        self.errors
            .push(format!("{} {}: {}", context, path.display(), error));
    }

    /// Build final `IndexingResult` (used by sync path and tests).
    fn into_result(self, operation_id: Option<OperationId>, status: &str) -> IndexingResult {
        IndexingResult {
            files_processed: self.files_processed,
            chunks_created: self.chunks_created,
            files_skipped: self.files_skipped,
            errors: self.errors,
            operation_id,
            status: status.to_owned(),
        }
    }
}

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
    fn workspace_relative_path(file_path: &Path, workspace_root: &Path) -> Result<String> {
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
    pub fn new_with_file_hash_repository(
        context_service: Arc<dyn ContextServiceInterface>,
        language_chunker: Arc<dyn LanguageChunkingProvider>,
        indexing_ops: Arc<dyn IndexingOperationsInterface>,
        event_bus: Arc<dyn EventBusProvider>,
        file_hash_repository: Arc<dyn FileHashRepository>,
        supported_extensions: Vec<String>,
    ) -> Self {
        Self {
            context_service,
            language_chunker,
            indexing_ops,
            event_bus,
            file_hash_repository: Some(file_hash_repository),
            supported_extensions: Self::normalize_supported_extensions(supported_extensions),
        }
    }

    fn normalize_supported_extensions(extensions: Vec<String>) -> Vec<String> {
        extensions
            .into_iter()
            .map(|ext| ext.trim().trim_start_matches('.').to_ascii_lowercase())
            .filter(|ext| !ext.is_empty())
            .collect()
    }

    /// Discover files recursively from a path
    async fn discover_files(
        &self,
        path: &Path,
        progress: &mut IndexingProgress,
    ) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .filter_entry(|entry| {
                if !entry.file_type().is_some_and(|ft| ft.is_dir()) {
                    return true;
                }

                entry
                    .file_name()
                    .to_str()
                    .is_none_or(|name| !SKIP_DIRS.contains(&name))
            })
            .build();

        for entry_result in walker {
            match entry_result {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|ft| ft.is_file())
                        && self.is_supported_file(entry.path())
                    {
                        files.push(entry.path().to_path_buf());
                    }
                }
                Err(e) => {
                    progress.record_error("Failed to read directory entry", path, e);
                }
            }
        }

        files
    }

    /// Check if file has a supported extension
    fn is_supported_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| {
                self.supported_extensions
                    .iter()
                    .any(|supported| supported == &ext.to_ascii_lowercase())
            })
    }
}

#[async_trait::async_trait]
impl IndexingServiceInterface for IndexingServiceImpl {
    async fn index_codebase(
        &self,
        path: &Path,
        collection: &CollectionId,
    ) -> Result<IndexingResult> {
        // Initialize collection
        self.context_service.initialize(collection).await?;

        // Discover files first (quick operation)
        let mut progress = IndexingProgress::new();
        let files = self.discover_files(path, &mut progress).await;
        let total_files = files.len();

        info!(
            "Starting indexing: {} files in {}",
            total_files,
            path.display()
        );

        // Start tracking operation
        let operation_id = self.indexing_ops.start_operation(collection, total_files);

        // Publish IndexingStarted event
        if let Err(e) = self
            .event_bus
            .publish_event(DomainEvent::IndexingStarted {
                collection: collection.to_string(),
                total_files,
            })
            .await
        {
            warn!("Failed to publish IndexingStarted event: {}", e);
        }

        // Clone service for the background task
        // IndexingServiceImpl is cheap to clone (Arc-based)
        let service = self.clone();
        let collection_id = *collection;
        let op_id = operation_id;
        let workspace_root = path.to_path_buf();

        // Fire-and-forget: caller gets operation_id immediately, polling for completion.
        // Sync execution path available via run_indexing_task() directly in tests.
        let _handle = tokio::spawn(async move {
            Self::run_indexing_task(service, files, workspace_root, collection_id, op_id).await;
        });

        // Return immediately with operation_id
        Ok(IndexingResult {
            files_processed: 0,
            chunks_created: 0,
            files_skipped: 0,
            errors: vec![],
            operation_id: Some(operation_id),
            status: "started".to_owned(),
        })
    }

    fn get_status(&self) -> mcb_domain::ports::services::IndexingStatus {
        use mcb_domain::ports::services::IndexingStatus;

        let ops = self.indexing_ops.get_operations();
        // Get first active operation if any - use if-let to avoid expect()
        if let Some((_, op)) = ops.iter().next() {
            let total = op.total_files.max(1);
            IndexingStatus {
                is_indexing: true,
                progress: op.processed_files as f64 / total as f64,
                current_file: op.current_file.clone(),
                total_files: op.total_files,
                processed_files: op.processed_files,
            }
        } else {
            IndexingStatus::default()
        }
    }

    async fn clear_collection(&self, collection: &CollectionId) -> Result<()> {
        self.context_service.clear_collection(collection).await
    }
}

impl IndexingServiceImpl {
    /// Background task that performs the actual indexing work.
    ///
    /// # Code Smells
    /// TODO(qlty): Function with high complexity (count = 27).
    async fn run_indexing_task(
        service: IndexingServiceImpl,
        files: Vec<PathBuf>,
        workspace_root: PathBuf,
        collection: CollectionId,
        operation_id: OperationId,
    ) {
        let start = Instant::now();
        let total = files.len();
        let mut chunks_created = 0usize;
        let mut files_processed = 0usize;
        let mut failed_files: Vec<String> = Vec::new();

        for (i, file_path) in files.iter().enumerate() {
            let relative_path = match Self::workspace_relative_path(file_path, &workspace_root) {
                Ok(path) => path,
                Err(e) => {
                    warn!(file = %file_path.display(), error = %e, "Skipping file outside workspace root");
                    failed_files.push(file_path.display().to_string());
                    continue;
                }
            };

            service
                .indexing_ops
                .update_progress(&operation_id, Some(relative_path.clone()), i);

            if i % PROGRESS_UPDATE_INTERVAL == 0
                && let Err(e) = service
                    .event_bus
                    .publish_event(DomainEvent::IndexingProgress {
                        collection: collection.to_string(),
                        processed: i,
                        total,
                        current_file: Some(relative_path.clone()),
                    })
                    .await
            {
                warn!("Failed to publish progress event: {}", e);
            }

            let content = match tokio::fs::read_to_string(&file_path).await {
                Ok(c) => c,
                Err(e) => {
                    warn!(file = %file_path.display(), error = %e, "Failed to read file during indexing");
                    failed_files.push(file_path.display().to_string());
                    continue;
                }
            };

            let mut computed_hash: Option<String> = None;
            if let Some(file_hash_repository) = &service.file_hash_repository {
                match file_hash_repository.compute_hash(file_path.as_path()) {
                    Ok(hash) => {
                        match file_hash_repository
                            .has_changed(&collection.to_string(), &relative_path, &hash)
                            .await
                        {
                            Ok(false) => {
                                continue;
                            }
                            Ok(true) => {
                                computed_hash = Some(hash);
                            }
                            Err(e) => {
                                warn!(
                                    file = %file_path.display(),
                                    error = %e,
                                    "Failed to check file hash delta"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            file = %file_path.display(),
                            error = %e,
                            "Failed to compute file hash"
                        );
                    }
                }
            }

            let chunks = service.language_chunker.chunk(&content, &relative_path);
            if let Err(e) = service
                .context_service
                .store_chunks(&collection, &chunks)
                .await
            {
                error!(file = %file_path.display(), error = %e, "Failed to store chunks — vector store or embedding provider may be unreachable");
                failed_files.push(file_path.display().to_string());
                continue;
            }

            if let (Some(file_hash_repository), Some(hash)) =
                (&service.file_hash_repository, computed_hash.as_deref())
                && let Err(e) = file_hash_repository
                    .upsert_hash(&collection.to_string(), &relative_path, hash)
                    .await
            {
                warn!(
                    file = %file_path.display(),
                    error = %e,
                    "Failed to persist file hash metadata"
                );
            }

            files_processed += 1;
            chunks_created += chunks.len();
        }

        service
            .indexing_ops
            .update_progress(&operation_id, None, total);

        service.indexing_ops.complete_operation(&operation_id);

        let duration_ms = start.elapsed().as_millis() as u64;
        let error_count = failed_files.len();
        let files_skipped = total.saturating_sub(files_processed);

        let result = IndexingProgress::with_counts(
            files_processed,
            chunks_created,
            files_skipped,
            failed_files.clone(),
        )
        .into_result(Some(operation_id), "completed");

        if let Err(e) = service
            .event_bus
            .publish_event(DomainEvent::IndexingCompleted {
                collection: collection.to_string(),
                chunks: result.chunks_created,
                duration_ms,
            })
            .await
        {
            warn!("Failed to publish IndexingCompleted event: {}", e);
        }

        if error_count > 0 {
            error!(
                files_processed = files_processed,
                chunks_created = chunks_created,
                errors = error_count,
                duration_ms = duration_ms,
                "Indexing completed with errors"
            );
        } else {
            info!(
                files_processed = files_processed,
                chunks_created = chunks_created,
                duration_ms = duration_ms,
                "Indexing completed successfully"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IndexingServiceImpl;
    use std::path::Path;

    #[test]
    fn workspace_relative_path_normalizes_within_workspace() {
        let workspace = Path::new("/repo");
        let file = Path::new("/repo/src/main.rs");

        let relative =
            IndexingServiceImpl::workspace_relative_path(file, workspace).expect("relative path");

        assert_eq!(relative, "src/main.rs");
    }

    #[test]
    fn workspace_relative_path_rejects_outside_workspace() {
        let workspace = Path::new("/repo");
        let file = Path::new("/other/main.rs");

        let err = IndexingServiceImpl::workspace_relative_path(file, workspace)
            .expect_err("outside path must fail");

        assert!(err.to_string().contains("is not under root"));
    }
}
