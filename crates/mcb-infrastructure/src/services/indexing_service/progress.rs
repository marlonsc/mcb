//! Indexing progress tracking and result accumulation.
//!
//! This module defines the `IndexingProgress` struct which accumulates metrics
//! during an indexing operation and converts them to final results.

use std::path::Path;

use mcb_domain::ports::IndexingResult;
use mcb_domain::value_objects::OperationId;

/// Accumulator for indexing progress and operational metrics.
///
/// Tracks the state of an active indexing operation, including success counts,
/// skipped files, and encountered errors for final reporting.
pub struct IndexingProgress {
    /// Number of files successfully processed.
    pub files_processed: usize,
    /// Total number of chunks created across all processed files.
    pub chunks_created: usize,
    /// Number of files skipped (unchanged since last indexing).
    pub files_skipped: usize,
    /// Errors encountered during indexing.
    pub errors: Vec<String>,
}

impl IndexingProgress {
    /// Create a new empty progress tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            files_processed: 0,
            chunks_created: 0,
            files_skipped: 0,
            errors: Vec::new(),
        }
    }

    /// Create a progress tracker with pre-set counts.
    #[must_use]
    pub fn with_counts(
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

    /// Record an error encountered during file processing.
    pub fn record_error(&mut self, context: &str, path: &Path, error: impl std::fmt::Display) {
        self.errors
            .push(format!("{} {}: {}", context, path.display(), error));
    }

    /// Build final `IndexingResult` (used by sync path and tests).
    #[must_use]
    pub fn into_result(self, operation_id: Option<OperationId>, status: &str) -> IndexingResult {
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

impl Default for IndexingProgress {
    fn default() -> Self {
        Self::new()
    }
}
