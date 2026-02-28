//! `IndexingServiceInterface` implementation.
//!
//! This module implements the core indexing service interface, handling
//! the async indexing workflow, status tracking, and collection management.

use std::path::Path;

use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::{IndexingResult, IndexingServiceInterface, IndexingStatus};
use mcb_domain::value_objects::CollectionId;

use super::{IndexingProgress, IndexingServiceImpl};

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
            super::processing::run_indexing_task(
                service,
                files,
                workspace_root,
                collection_id,
                op_id,
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
            status: mcb_domain::constants::INDEXING_STATUS_STARTED.to_owned(),
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
    /// Returns an error if the context service or hash repository fails to clear
    /// the collection.
    async fn clear_collection(&self, collection: &CollectionId) -> Result<()> {
        self.context_service.clear_collection(collection).await?;
        // Also clear stale hashes so next indexing re-processes all files
        if let Some(repo) = &self.file_hash_repository {
            repo.clear_collection(&collection.to_string()).await?;
        }
        Ok(())
    }
}
