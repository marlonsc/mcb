//! Mock Search Service implementation

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::services::SearchServiceInterface;
use mcb_domain::value_objects::{CollectionId, SearchResult};

use crate::test_utils::helpers::{arc_mutex, arc_mutex_vec};

/// Mock implementation of SearchServiceInterface for testing
pub struct MockSearchService {
    /// Pre-configured results to return
    pub results: Arc<Mutex<Vec<SearchResult>>>,
    /// Whether the next call should fail
    pub should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    pub error_message: Arc<Mutex<String>>,
}

impl MockSearchService {
    /// Create a new mock search service
    pub fn new() -> Self {
        Self {
            results: arc_mutex_vec(),
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: arc_mutex("Simulated search failure".to_string()),
        }
    }

    /// Configure the mock to return specific results
    pub fn with_results(self, results: Vec<SearchResult>) -> Self {
        *self.results.lock().expect("Lock poisoned") = results;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }
}

impl Default for MockSearchService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchServiceInterface for MockSearchService {
    async fn search(
        &self,
        _collection: &CollectionId,
        _query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let results = self.results.lock().expect("Lock poisoned");
        Ok(results.iter().take(limit).cloned().collect())
    }

    async fn search_with_filters(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
        _filters: Option<&mcb_domain::ports::services::SearchFilters>,
    ) -> Result<Vec<SearchResult>> {
        // Mock ignores filters and delegates to search
        self.search(collection, query, limit).await
    }
}
