//! Null Hybrid Search Provider
//!
//! A no-op implementation of HybridSearchProvider for testing and development.
//! This provider passes through semantic results without any BM25 enhancement.

use async_trait::async_trait;
use mcb_application::ports::providers::HybridSearchProvider;
use mcb_domain::{entities::CodeChunk, error::Result, value_objects::SearchResult};
use serde_json::Value;
use std::collections::HashMap;

/// Null implementation of HybridSearchProvider
///
/// This provider is useful for:
/// - Testing without BM25 indexing overhead
/// - Development environments
/// - Fallback when hybrid search is not needed
///
/// It passes through semantic results unchanged, effectively disabling
/// the BM25 component of hybrid search.
#[derive(Debug, Default, Clone)]
pub struct NullHybridSearchProvider;

impl NullHybridSearchProvider {
    /// Create a new null hybrid search provider
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HybridSearchProvider for NullHybridSearchProvider {
    /// Index chunks (no-op)
    ///
    /// This implementation does nothing, as no BM25 index is maintained.
    async fn index_chunks(&self, _collection: &str, _chunks: &[CodeChunk]) -> Result<()> {
        Ok(())
    }

    /// Search (pass-through)
    ///
    /// Returns semantic results unchanged, limited to the requested count.
    async fn search(
        &self,
        _collection: &str,
        _query: &str,
        semantic_results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        Ok(semantic_results.into_iter().take(limit).collect())
    }

    /// Clear collection (no-op)
    async fn clear_collection(&self, _collection: &str) -> Result<()> {
        Ok(())
    }

    /// Get statistics
    ///
    /// Returns minimal stats indicating this is a null provider.
    async fn get_stats(&self) -> HashMap<String, Value> {
        let mut stats = HashMap::new();
        stats.insert("provider".to_string(), serde_json::json!("null"));
        stats.insert("bm25_enabled".to_string(), serde_json::json!(false));
        stats.insert("collection_count".to_string(), serde_json::json!(0));
        stats
    }
}
