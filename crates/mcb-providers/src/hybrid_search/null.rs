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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_chunk(content: &str, file_path: &str, start_line: u32) -> CodeChunk {
        CodeChunk {
            id: format!("{}:{}", file_path, start_line),
            content: content.to_string(),
            file_path: file_path.to_string(),
            start_line,
            end_line: start_line + 1,
            language: "Rust".to_string(),
            metadata: serde_json::json!({}),
        }
    }

    fn create_test_search_result(file_path: &str, score: f64) -> SearchResult {
        SearchResult {
            id: format!("{}:1", file_path),
            content: "test content".to_string(),
            file_path: file_path.to_string(),
            start_line: 1,
            score,
            language: "Rust".to_string(),
        }
    }

    #[tokio::test]
    async fn test_null_provider_index() {
        let provider = NullHybridSearchProvider::new();
        let chunks = vec![create_test_chunk("fn test() {}", "test.rs", 1)];

        // Should succeed without error
        provider.index_chunks("test", &chunks).await.unwrap();
    }

    #[tokio::test]
    async fn test_null_provider_search_passthrough() {
        let provider = NullHybridSearchProvider::new();

        let semantic_results = vec![
            create_test_search_result("a.rs", 0.9),
            create_test_search_result("b.rs", 0.8),
            create_test_search_result("c.rs", 0.7),
        ];

        let results = provider
            .search("test", "query", semantic_results.clone(), 2)
            .await
            .unwrap();

        // Should return first 2 results unchanged
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].file_path, "a.rs");
        assert!((results[0].score - 0.9).abs() < f64::EPSILON);
        assert_eq!(results[1].file_path, "b.rs");
    }

    #[tokio::test]
    async fn test_null_provider_clear() {
        let provider = NullHybridSearchProvider::new();

        // Should succeed without error
        provider.clear_collection("test").await.unwrap();
    }

    #[tokio::test]
    async fn test_null_provider_stats() {
        let provider = NullHybridSearchProvider::new();

        let stats = provider.get_stats().await;

        assert_eq!(stats.get("provider"), Some(&serde_json::json!("null")));
        assert_eq!(stats.get("bm25_enabled"), Some(&serde_json::json!(false)));
    }
}
