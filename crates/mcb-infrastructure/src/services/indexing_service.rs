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

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use ignore::WalkBuilder;
use mcb_domain::constants::{INDEXING_STATUS_COMPLETED, INDEXING_STATUS_STARTED};
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::{
    ContextServiceInterface, EventBusProvider, FileHashRepository, IndexingOperationsInterface,
    IndexingResult, IndexingServiceInterface, IndexingStatus, LanguageChunkingProvider,
};
use mcb_domain::registry::database::resolve_database_repositories;
use mcb_domain::registry::language::{LanguageProviderConfig, resolve_language_provider};
use mcb_domain::registry::services::{
    INDEXING_SERVICE_NAME, ServiceBuilder, ServiceRegistryEntry, resolve_context_service,
};
use mcb_domain::value_objects::{CollectionId, OperationId};

use crate::constants::use_cases::SKIP_DIRS;
use crate::infrastructure::DefaultIndexingOperations;

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

/// Result of processing a single file during indexing.
#[derive(Debug, Clone)]
enum ProcessResult {
    /// File was successfully indexed with the given number of chunks.
    Processed { chunks: usize },
    /// File was skipped because it hasn't changed.
    Skipped,
}

#[async_trait::async_trait]
impl IndexingServiceInterface for IndexingServiceImpl {
    /// # Errors
    ///
    /// Returns an error if collection initialization fails.
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

        mcb_domain::info!(
            "indexing",
            &format!(
                "Starting indexing: {} files in {}",
                total_files,
                path.display()
            )
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
            mcb_domain::warn!("indexing", "Failed to publish IndexingStarted event", &e);
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
            status: INDEXING_STATUS_STARTED.to_owned(),
        })
    }

    fn get_status(&self) -> IndexingStatus {
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

    /// # Errors
    ///
    /// Returns an error if the context service fails to clear the collection.
    async fn clear_collection(&self, collection: &CollectionId) -> Result<()> {
        self.context_service.clear_collection(collection).await
    }
}

impl IndexingServiceImpl {
    /// Background task that performs the actual indexing work.
    async fn run_indexing_task(
        service: IndexingServiceImpl,
        files: Vec<PathBuf>,
        workspace_root: PathBuf,
        collection: CollectionId,
        operation_id: OperationId,
    ) {
        let start = Instant::now();
        let total = files.len();
        let mut chunks_created = 0;
        let mut files_processed = 0;
        let mut failed_files: Vec<String> = Vec::new();

        for (i, file_path) in files.iter().enumerate() {
            match service
                .process_file(file_path, &workspace_root, &collection, &operation_id, i)
                .await
            {
                Ok(ProcessResult::Processed { chunks }) => {
                    files_processed += 1;
                    chunks_created += chunks;
                }
                Ok(ProcessResult::Skipped) => {
                    // File hasn't changed, increment skip count but don't record as processed
                }
                Err(e) => {
                    mcb_domain::warn!(
                        "indexing",
                        "Failed to process file during indexing",
                        &format!("file={} error={}", file_path.display(), e)
                    );
                    failed_files.push(file_path.display().to_string());
                }
            }
        }

        service
            .indexing_ops
            .update_progress(&operation_id, None, total);

        service.indexing_ops.complete_operation(&operation_id);

        let duration_ms = start.elapsed().as_millis() as u64;
        let files_skipped = total.saturating_sub(files_processed);

        let result = IndexingProgress::with_counts(
            files_processed,
            chunks_created,
            files_skipped,
            failed_files.clone(),
        )
        .into_result(Some(operation_id), INDEXING_STATUS_COMPLETED);

        if let Err(e) = service
            .event_bus
            .publish_event(DomainEvent::IndexingCompleted {
                collection: collection.to_string(),
                chunks: result.chunks_created,
                duration_ms,
            })
            .await
        {
            mcb_domain::warn!("indexing", "Failed to publish IndexingCompleted event", &e);
        }

        let error_count = failed_files.len();
        if error_count > 0 {
            mcb_domain::error!(
                "indexing",
                "Indexing completed with errors",
                &format!(
                    "files_processed={files_processed} chunks_created={chunks_created} errors={error_count} duration_ms={duration_ms}"
                )
            );
        } else {
            mcb_domain::info!(
                "indexing",
                "Indexing completed successfully",
                &format!(
                    "files_processed={files_processed} chunks_created={chunks_created} duration_ms={duration_ms}"
                )
            );
        }
    }

    /// Process a single file: check for changes, chunk it, and store results.
    async fn process_file(
        &self,
        file_path: &Path,
        workspace_root: &Path,
        collection: &CollectionId,
        operation_id: &OperationId,
        index: usize,
    ) -> Result<ProcessResult> {
        let relative_path = Self::workspace_relative_path(file_path, workspace_root)?;

        // Update progress tracking
        self.indexing_ops
            .update_progress(operation_id, Some(relative_path.clone()), index);

        // Read file content
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| mcb_domain::error::Error::internal(format!("Failed to read file: {e}")))?;

        // Incremental check using file hashes
        let current_hash = mcb_domain::utils::compute_content_hash(&content);
        // Cannot collapse: outer `if let` unwraps Option, inner `if` checks a bool condition.
        #[allow(clippy::collapsible_if)]
        if let Some(repo) = &self.file_hash_repository {
            if !repo
                .has_changed(&collection.to_string(), &relative_path, &current_hash)
                .await?
            {
                return Ok(ProcessResult::Skipped);
            }
        }

        // Generate semantic chunks
        let chunks = self.language_chunker.chunk(&content, &relative_path);
        let chunk_count = chunks.len();

        // Store chunks in context storage
        if !chunks.is_empty() {
            self.context_service
                .store_chunks(collection, &chunks)
                .await?;
        }

        // Update hash repository to reflect success
        if let Some(repo) = &self.file_hash_repository {
            repo.upsert_hash(&collection.to_string(), &relative_path, &current_hash)
                .await?;
        }

        Ok(ProcessResult::Processed {
            chunks: chunk_count,
        })
    }
}

fn build_indexing_service_from_registry(
    context: &dyn std::any::Any,
) -> Result<Arc<dyn IndexingServiceInterface>> {
    let ctx = context
        .downcast_ref::<crate::resolution_context::ServiceResolutionContext>()
        .ok_or_else(|| {
            mcb_domain::error::Error::internal(
                "Indexing registry builder requires ServiceResolutionContext",
            )
        })?;

    let app_config = &ctx.config;
    let db = ctx.db.clone();

    let context_service = resolve_context_service(context)?;
    let language_chunker = resolve_language_provider(&LanguageProviderConfig::new("universal"))?;

    let database_provider = app_config.providers.database.provider.clone();
    let repositories =
        resolve_database_repositories(&database_provider, Box::new(db), "default".to_owned())
            .map_err(mcb_domain::error::Error::internal)?;

    let indexing_ops: Arc<dyn IndexingOperationsInterface> =
        Arc::new(DefaultIndexingOperations::new());
    let event_bus = ctx.event_bus.clone();

    Ok(Arc::new(
        IndexingServiceImpl::new_with_file_hash_repository(IndexingServiceWithHashDeps {
            service: IndexingServiceDeps {
                context_service,
                language_chunker,
                indexing_ops,
                event_bus,
                supported_extensions: app_config.mcp.indexing.supported_extensions.clone(),
            },
            file_hash_repository: repositories.file_hash,
        }),
    ))
}

#[linkme::distributed_slice(mcb_domain::registry::services::SERVICES_REGISTRY)]
static INDEXING_SERVICE_REGISTRY_ENTRY: ServiceRegistryEntry = ServiceRegistryEntry {
    name: INDEXING_SERVICE_NAME,
    build: ServiceBuilder::Indexing(build_indexing_service_from_registry),
};

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
