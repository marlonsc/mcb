//! File processing and background indexing task execution.
//!
//! This module handles the core indexing work: processing individual files,
//! computing hashes, chunking content, and publishing completion events.

use std::path::{Path, PathBuf};
use std::time::Instant;

use mcb_domain::constants::INDEXING_STATUS_COMPLETED;
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::value_objects::{CollectionId, OperationId};

use super::{IndexingProgress, IndexingServiceImpl, ProcessResult};

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

impl IndexingServiceImpl {
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

        // Update progress tracking
        self.indexing_ops
            .update_progress(operation_id, Some(relative_path.clone()), index);

        // Read file content
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| mcb_domain::error::Error::internal(format!("Failed to read file: {e}")))?;

        // Incremental check using file hashes
        let current_hash = mcb_domain::utils::id::compute_content_hash(&content);
        match &self.file_hash_repository {
            Some(repo)
                if !repo
                    .has_changed(&collection.to_string(), &relative_path, &current_hash)
                    .await? =>
            {
                return Ok(ProcessResult::Skipped);
            }
            _ => {}
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
