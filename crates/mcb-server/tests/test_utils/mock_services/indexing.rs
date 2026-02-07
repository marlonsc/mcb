//! Mock Indexing Service implementation

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::services::{IndexingResult, IndexingServiceInterface, IndexingStatus};
use mcb_domain::value_objects::CollectionId;

/// Mock implementation of IndexingServiceInterface for testing
pub struct MockIndexingService {
    /// Pre-configured indexing result
    pub indexing_result: Arc<Mutex<Option<IndexingResult>>>,
    /// Current status to return
    pub status: Arc<Mutex<IndexingStatus>>,
    /// Whether the next indexing call should fail
    pub should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    pub error_message: Arc<Mutex<String>>,
}

impl MockIndexingService {
    /// Create a new mock indexing service
    pub fn new() -> Self {
        Self {
            indexing_result: Arc::new(Mutex::new(Some(IndexingResult {
                files_processed: 0,
                chunks_created: 0,
                files_skipped: 0,
                errors: Vec::new(),
                operation_id: None,
                status: "completed".to_string(),
            }))),
            status: Arc::new(Mutex::new(IndexingStatus::default())),
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated indexing failure".to_string())),
        }
    }

    /// Configure the mock to return specific indexing result
    pub fn with_result(self, result: IndexingResult) -> Self {
        *self.indexing_result.lock().expect("Lock poisoned") = Some(result);
        self
    }

    /// Configure the mock to return specific status
    pub fn with_status(self, status: IndexingStatus) -> Self {
        *self.status.lock().expect("Lock poisoned") = status;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }
}

impl Default for MockIndexingService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IndexingServiceInterface for MockIndexingService {
    async fn index_codebase(
        &self,
        _path: &Path,
        _collection: &CollectionId,
    ) -> Result<IndexingResult> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let result = self.indexing_result.lock().expect("Lock poisoned");
        Ok(result.clone().unwrap_or_else(|| IndexingResult {
            files_processed: 0,
            chunks_created: 0,
            files_skipped: 0,
            errors: Vec::new(),
            operation_id: None,
            status: "completed".to_string(),
        }))
    }

    fn get_status(&self) -> IndexingStatus {
        self.status.lock().expect("Lock poisoned").clone()
    }

    async fn clear_collection(&self, _collection: &CollectionId) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }
}
