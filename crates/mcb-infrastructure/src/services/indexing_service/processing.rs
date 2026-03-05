//! File processing and background indexing task execution.
//!
//! This module handles the core indexing work: processing individual files,
//! computing hashes, chunking content, and publishing completion events.

use std::path::{Path, PathBuf};
use std::time::Instant;

use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::value_objects::{CollectionId, OperationId};
use mcb_utils::constants::INDEXING_STATUS_COMPLETED;

use super::{IndexingProgress, IndexingServiceImpl, ProcessResult};

async fn publish_indexing_completed_event(
    service: &IndexingServiceImpl,
    collection: &CollectionId,
    chunks_created: usize,
    duration_ms: u64,
) {
    if let Err(e) = service
        .event_bus
        .publish_event(DomainEvent::IndexingCompleted {
            collection: collection.to_string(),
            chunks: chunks_created,
            duration_ms,
        })
        .await
    {
        mcb_domain::warn!("indexing", "Failed to publish IndexingCompleted event", &e);
    }
}

fn log_indexing_completion(
    error_count: usize,
    files_processed: usize,
    chunks_created: usize,
    duration_ms: u64,
) {
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

#[allow(clippy::too_many_arguments)]
async fn finish_indexing_task(
    service: &IndexingServiceImpl,
    operation_id: &OperationId,
    collection: &CollectionId,
    total: usize,
    files_processed: usize,
    chunks_created: usize,
    failed_files: Vec<String>,
    start: std::time::Instant,
) {
    service
        .indexing_ops
        .update_progress(operation_id, None, total);

    service.indexing_ops.complete_operation(operation_id);

    let duration_ms = start.elapsed().as_millis() as u64;
    let files_skipped = total.saturating_sub(files_processed);
    let error_count = failed_files.len();

    let result =
        IndexingProgress::with_counts(files_processed, chunks_created, files_skipped, failed_files)
            .into_result(Some(*operation_id), INDEXING_STATUS_COMPLETED);

    publish_indexing_completed_event(service, collection, result.chunks_created, duration_ms).await;
    log_indexing_completion(error_count, files_processed, chunks_created, duration_ms);
}

/// Background task that performs the actual indexing work.
pub async fn run_indexing_task(
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

    finish_indexing_task(
        &service,
        &operation_id,
        &collection,
        total,
        files_processed,
        chunks_created,
        failed_files,
        start,
    )
    .await;
}

impl IndexingServiceImpl {
    /// Process a single file: check for changes, chunk it, and store results.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, hashed, chunked, or stored.
    async fn check_incremental(
        &self,
        collection: &CollectionId,
        relative_path: &str,
        content: &str,
    ) -> Result<Option<String>> {
        let current_hash = mcb_utils::utils::id::compute_content_hash(content);
        match &self.file_hash_repository {
            Some(repo)
                if !repo
                    .has_changed(&collection.to_string(), relative_path, &current_hash)
                    .await? =>
            {
                Ok(None)
            }
            _ => {
                mcb_domain::trace!(
                    "indexing",
                    "File changed or no hash check available",
                    &relative_path
                );
                Ok(Some(current_hash))
            }
        }
    }

    async fn create_and_store_chunks(
        &self,
        content: &str,
        relative_path: &str,
        collection: &CollectionId,
    ) -> Result<usize> {
        let chunks = self.language_chunker.chunk(content, relative_path);
        let chunk_count = chunks.len();

        if !chunks.is_empty() {
            self.context_service
                .store_chunks(collection, &chunks)
                .await?;
        }
        Ok(chunk_count)
    }

    /// Process a single file: check for changes, chunk it, and store results.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, hashed, chunked, or stored.
    pub async fn process_file(
        &self,
        file_path: &Path,
        workspace_root: &Path,
        collection: &CollectionId,
        operation_id: &OperationId,
        index: usize,
    ) -> Result<ProcessResult> {
        let relative_path = Self::workspace_relative_path(file_path, workspace_root)?;

        self.indexing_ops
            .update_progress(operation_id, Some(relative_path.clone()), index);

        let content = std::fs::read_to_string(file_path)
            .map_err(|e| mcb_domain::error::Error::internal(format!("Failed to read file: {e}")))?;

        let current_hash = match self
            .check_incremental(collection, &relative_path, &content)
            .await?
        {
            Some(hash) => hash,
            None => return Ok(ProcessResult::Skipped),
        };

        let chunk_count = self
            .create_and_store_chunks(&content, &relative_path, collection)
            .await?;

        if let Some(repo) = &self.file_hash_repository {
            repo.upsert_hash(&collection.to_string(), &relative_path, &current_hash)
                .await?;
        }

        Ok(ProcessResult::Processed {
            chunks: chunk_count,
        })
    }
}
