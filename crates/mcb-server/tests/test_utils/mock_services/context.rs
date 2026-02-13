//! Mock Context Service implementation

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::ports::services::ContextServiceInterface;
use mcb_domain::value_objects::{CollectionId, Embedding, SearchResult};

use crate::test_utils::helpers::{arc_mutex, arc_mutex_vec};

/// Mock implementation of ContextServiceInterface for testing
pub struct TestContextService {
    /// Pre-configured search results
    pub search_results: Arc<Mutex<Vec<SearchResult>>>,
    /// Embedding dimensions
    pub dimensions: usize,
    /// Whether the next call should fail
    pub should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    pub error_message: Arc<Mutex<String>>,
}

impl TestContextService {
    /// Create a new mock context service
    pub fn new() -> Self {
        Self {
            search_results: arc_mutex_vec(),
            dimensions: 384,
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: arc_mutex("Simulated context failure".to_string()),
        }
    }

    /// Configure the mock to return specific search results
    pub fn with_search_results(self, results: Vec<SearchResult>) -> Self {
        *self.search_results.lock().expect("Lock poisoned") = results;
        self
    }

    /// Configure the mock to use specific dimensions
    pub fn with_dimensions(mut self, dims: usize) -> Self {
        self.dimensions = dims;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }
}

impl Default for TestContextService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextServiceInterface for TestContextService {
    async fn initialize(&self, _collection: &CollectionId) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }

    async fn store_chunks(&self, _collection: &CollectionId, _chunks: &[CodeChunk]) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }

    async fn search_similar(
        &self,
        _collection: &CollectionId,
        _query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let results = self.search_results.lock().expect("Lock poisoned");
        Ok(results.iter().take(limit).cloned().collect())
    }

    async fn embed_text(&self, _text: &str) -> Result<Embedding> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        Ok(Embedding {
            vector: vec![0.1; self.dimensions],
            model: "mock".to_string(),
            dimensions: self.dimensions,
        })
    }

    async fn clear_collection(&self, _collection: &CollectionId) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }

    async fn get_stats(&self) -> Result<(i64, i64)> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        Ok((100, 10))
    }

    fn embedding_dimensions(&self) -> usize {
        self.dimensions
    }
}
