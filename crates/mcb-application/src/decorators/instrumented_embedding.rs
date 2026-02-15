//! Instrumented Embedding Provider Decorator
//!
//! Wraps an `EmbeddingProvider` to record timing metrics for all operations.
//! Follows SOLID Open/Closed principle - adds metrics without modifying providers.
//!
//! ## Example
//!
//! ```rust,ignore
//! use mcb_application::decorators::InstrumentedEmbeddingProvider;
//! use std::sync::Arc;
//!
//! let base_provider = Arc::new(OpenAIEmbeddingProvider::new(...));
//! let metrics = Arc::new(PerformanceMetricsService::new());
//!
//! let instrumented = InstrumentedEmbeddingProvider::new(base_provider, metrics);
//! // Now all embed calls will record timing metrics
//! ```

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use delegate::delegate;
use mcb_domain::error::Result;
use mcb_domain::ports::admin::PerformanceMetricsInterface;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;

/// Instrumented embedding provider decorator
///
/// Wraps any `EmbeddingProvider` to add timing metrics collection.
/// All operations are delegated to the inner provider after recording metrics.
///
/// ## Metrics Recorded
///
/// - Response time (ms) for each operation
/// - Success/failure status
/// - Cache hits (always false for embeddings, but follows interface)
pub struct InstrumentedEmbeddingProvider {
    /// The wrapped provider
    inner: Arc<dyn EmbeddingProvider>,
    /// Metrics collector
    metrics: Arc<dyn PerformanceMetricsInterface>,
}

impl InstrumentedEmbeddingProvider {
    /// Create a new instrumented embedding provider
    ///
    /// # Arguments
    /// * `inner` - The embedding provider to wrap
    /// * `metrics` - The metrics collector to record timing data
    pub fn new(
        inner: Arc<dyn EmbeddingProvider>,
        metrics: Arc<dyn PerformanceMetricsInterface>,
    ) -> Self {
        Self { inner, metrics }
    }

    /// Get the wrapped provider name for logging/debugging
    #[must_use]
    pub fn inner_provider_name(&self) -> &str {
        self.inner.provider_name()
    }
}

#[async_trait]
impl EmbeddingProvider for InstrumentedEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let start = Instant::now();
        let result = self.inner.embed(text).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Record metrics: response_time, success, cache_hit (always false for embeddings)
        self.metrics
            .record_query(duration_ms, result.is_ok(), false);

        result
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let start = Instant::now();
        let result = self.inner.embed_batch(texts).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Record metrics for batch operation
        self.metrics
            .record_query(duration_ms, result.is_ok(), false);

        result
    }

    delegate! {
        to self.inner {
            fn dimensions(&self) -> usize;
            fn provider_name(&self) -> &str;
        }
    }
}

// Tests are in crates/mcb-application/tests/unit/instrumented_embedding_test.rs
