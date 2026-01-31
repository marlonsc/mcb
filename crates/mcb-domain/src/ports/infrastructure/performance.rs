//! Performance Metrics Port
//!
//! Defines the contract for collecting performance metrics from providers.
//! Implements Phase 6 of the v0.1.5 MAX SCOPE release plan.

use std::time::Duration;

/// Performance metrics collector interface
///
/// Provides methods to record timing and count metrics for:
/// - Embedding operations
/// - Vector store queries
/// - Cache hits/misses
/// - Indexing throughput
pub trait PerformanceMetricsCollector: Send + Sync {
    /// Record an embedding operation duration
    ///
    /// # Arguments
    /// * `provider` - The embedding provider name (e.g., "ollama", "openai")
    /// * `duration` - The operation duration
    /// * `success` - Whether the operation succeeded
    fn record_embedding_latency(&self, provider: &str, duration: Duration, success: bool);

    /// Record a vector store query duration
    ///
    /// # Arguments
    /// * `provider` - The vector store provider name (e.g., "milvus", "qdrant")
    /// * `duration` - The query duration
    /// * `success` - Whether the query succeeded
    fn record_vectorstore_latency(&self, provider: &str, duration: Duration, success: bool);

    /// Record a cache hit
    ///
    /// # Arguments
    /// * `provider` - The cache provider name (e.g., "moka", "redis")
    fn record_cache_hit(&self, provider: &str);

    /// Record a cache miss
    ///
    /// # Arguments
    /// * `provider` - The cache provider name (e.g., "moka", "redis")
    fn record_cache_miss(&self, provider: &str);

    /// Record indexing throughput
    ///
    /// # Arguments
    /// * `chunks_per_second` - Current indexing throughput in chunks/second
    fn record_indexing_throughput(&self, chunks_per_second: f64);

    /// Record a batch embedding operation
    ///
    /// # Arguments
    /// * `provider` - The embedding provider name
    /// * `batch_size` - Number of texts in the batch
    /// * `duration` - Total batch operation duration
    /// * `success` - Whether the batch operation succeeded
    fn record_batch_embedding(
        &self,
        provider: &str,
        batch_size: usize,
        duration: Duration,
        success: bool,
    );
}

/// Null implementation for testing
pub struct NullMetricsCollector;

impl NullMetricsCollector {
    /// Create a new null metrics collector
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetricsCollector for NullMetricsCollector {
    fn record_embedding_latency(&self, _provider: &str, _duration: Duration, _success: bool) {}
    fn record_vectorstore_latency(&self, _provider: &str, _duration: Duration, _success: bool) {}
    fn record_cache_hit(&self, _provider: &str) {}
    fn record_cache_miss(&self, _provider: &str) {}
    fn record_indexing_throughput(&self, _chunks_per_second: f64) {}
    fn record_batch_embedding(
        &self,
        _provider: &str,
        _batch_size: usize,
        _duration: Duration,
        _success: bool,
    ) {
    }
}
