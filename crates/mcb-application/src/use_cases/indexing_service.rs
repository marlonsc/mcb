//! Indexing Service Use Case
//!
//! Application service for code indexing and ingestion operations.
//! Orchestrates file discovery, chunking, and storage of code embeddings.
//! Supports async background indexing with event publishing.

use crate::domain_services::search::{
    ContextServiceInterface, IndexingResult, IndexingServiceInterface,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::admin::IndexingOperationsInterface;
use mcb_domain::ports::infrastructure::EventBusProvider;
use mcb_domain::ports::providers::LanguageChunkingProvider;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Directories to skip during indexing
const SKIP_DIRS: &[&str] = &[".git", "node_modules", "target", "__pycache__"];

/// Supported file extensions for indexing
const SUPPORTED_EXTENSIONS: &[&str] = &["rs", "py", "js", "ts", "java", "cpp", "c", "go"];

/// Accumulator for indexing progress and errors
///
/// Note: Fields are used via `into_result()` method. The struct is WIP
/// for async background indexing support.
// Fields used during file discovery error recording, not dead code
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

    fn record_error(&mut self, context: &str, path: &Path, error: impl std::fmt::Display) {
        self.errors
            .push(format!("{} {}: {}", context, path.display(), error));
    }

    // Reserved for sync indexing path - async path builds IndexingResult directly
    #[allow(dead_code)]
    fn into_result(self, operation_id: Option<String>, status: &str) -> IndexingResult {
        IndexingResult {
            files_processed: self.files_processed,
            chunks_created: self.chunks_created,
            files_skipped: self.files_skipped,
            errors: self.errors,
            operation_id,
            status: status.to_string(),
        }
    }
}

/// Indexing service implementation - orchestrates file discovery and chunking
///
/// Supports async background indexing with progress tracking and event publishing.
pub struct IndexingServiceImpl {
    context_service: Arc<dyn ContextServiceInterface>,
    language_chunker: Arc<dyn LanguageChunkingProvider>,
    indexing_ops: Arc<dyn IndexingOperationsInterface>,
    event_bus: Arc<dyn EventBusProvider>,
}

impl IndexingServiceImpl {
    /// Create new indexing service with injected dependencies
    pub fn new(
        context_service: Arc<dyn ContextServiceInterface>,
        language_chunker: Arc<dyn LanguageChunkingProvider>,
        indexing_ops: Arc<dyn IndexingOperationsInterface>,
        event_bus: Arc<dyn EventBusProvider>,
    ) -> Self {
        Self {
            context_service,
            language_chunker,
            indexing_ops,
            event_bus,
        }
    }

    /// Discover files recursively from a path
    async fn discover_files(
        &self,
        path: &Path,
        progress: &mut IndexingProgress,
    ) -> Vec<std::path::PathBuf> {
        use tokio::fs;

        let mut files = Vec::new();
        let mut dirs_to_visit = vec![path.to_path_buf()];

        while let Some(dir_path) = dirs_to_visit.pop() {
            let mut entries = match fs::read_dir(&dir_path).await {
                Ok(entries) => entries,
                Err(e) => {
                    progress.record_error("Failed to read directory", &dir_path, e);
                    continue;
                }
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if Self::should_visit_dir(&entry_path) {
                        dirs_to_visit.push(entry_path);
                    }
                } else if Self::is_supported_file(&entry_path) {
                    files.push(entry_path);
                }
            }
        }
        files
    }

    /// Check if directory should be visited during indexing
    fn should_visit_dir(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| !SKIP_DIRS.contains(&name))
            .unwrap_or(true)
    }

    /// Check if file has a supported extension
    fn is_supported_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Chunk file content using intelligent AST-based chunking
    ///
    /// Reserved for future background task integration with `IndexingProgress`.
    /// Currently unused but retained for planned incremental indexing feature.
    #[allow(
        dead_code,
        reason = "Reserved for IndexingProgress integration in background tasks"
    )]
    fn chunk_file_content(&self, content: &str, path: &Path) -> Vec<CodeChunk> {
        self.language_chunker
            .chunk(content, &path.to_string_lossy())
    }
}

/// Progress update interval (publish event every N files)
const PROGRESS_UPDATE_INTERVAL: usize = 10;

#[async_trait::async_trait]
impl IndexingServiceInterface for IndexingServiceImpl {
    async fn index_codebase(&self, path: &Path, collection: &str) -> Result<IndexingResult> {
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

        // Clone dependencies for the background task
        let context_service = self.context_service.clone();
        let language_chunker = self.language_chunker.clone();
        let indexing_ops = self.indexing_ops.clone();
        let event_bus = self.event_bus.clone();
        let collection_owned = collection.to_string();
        let op_id = operation_id.clone();

        // Spawn background task - explicitly drop handle since we don't await it
        // (fire-and-forget pattern for async indexing)
        let _handle = tokio::spawn(async move {
            Self::run_indexing_task(
                context_service,
                language_chunker,
                indexing_ops,
                event_bus,
                files,
                &collection_owned,
                &op_id,
            )
            .await;
        });

        // Return immediately with operation_id
        Ok(IndexingResult {
            files_processed: 0,
            chunks_created: 0,
            files_skipped: 0,
            errors: vec![],
            operation_id: Some(operation_id),
            status: "started".to_string(),
        })
    }

    fn get_status(&self) -> crate::domain_services::search::IndexingStatus {
        use crate::domain_services::search::IndexingStatus;

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

    async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.context_service.clear_collection(collection).await
    }
}

impl IndexingServiceImpl {
    /// Background task that performs the actual indexing work
    async fn run_indexing_task(
        context_service: Arc<dyn ContextServiceInterface>,
        language_chunker: Arc<dyn LanguageChunkingProvider>,
        indexing_ops: Arc<dyn IndexingOperationsInterface>,
        event_bus: Arc<dyn EventBusProvider>,
        files: Vec<PathBuf>,
        collection: &str,
        operation_id: &str,
    ) {
        let start = Instant::now();
        let total = files.len();
        let mut chunks_created = 0usize;
        let mut files_processed = 0usize;

        for (i, file_path) in files.iter().enumerate() {
            // Update progress tracker
            indexing_ops.update_progress(operation_id, Some(file_path.display().to_string()), i);

            // Publish progress event periodically
            if i % PROGRESS_UPDATE_INTERVAL == 0
                && let Err(e) = event_bus
                    .publish_event(DomainEvent::IndexingProgress {
                        collection: collection.to_string(),
                        processed: i,
                        total,
                        current_file: Some(file_path.display().to_string()),
                    })
                    .await
            {
                debug!("Failed to publish progress event: {}", e);
            }

            // Read and process file
            let content = match tokio::fs::read_to_string(&file_path).await {
                Ok(c) => c,
                Err(e) => {
                    debug!("Failed to read file {}: {}", file_path.display(), e);
                    continue;
                }
            };

            let chunks = language_chunker.chunk(&content, &file_path.to_string_lossy());
            if let Err(e) = context_service.store_chunks(collection, &chunks).await {
                debug!("Failed to store chunks for {}: {}", file_path.display(), e);
                continue;
            }

            files_processed += 1;
            chunks_created += chunks.len();
        }

        // Update final progress
        indexing_ops.update_progress(operation_id, None, total);

        // Complete and remove operation
        indexing_ops.complete_operation(operation_id);

        let duration_ms = start.elapsed().as_millis() as u64;

        // Publish completion event
        if let Err(e) = event_bus
            .publish_event(DomainEvent::IndexingCompleted {
                collection: collection.to_string(),
                chunks: chunks_created,
                duration_ms,
            })
            .await
        {
            warn!("Failed to publish IndexingCompleted event: {}", e);
        }

        info!(
            "Indexing completed: {} files, {} chunks in {}ms",
            files_processed, chunks_created, duration_ms
        );
    }
}
