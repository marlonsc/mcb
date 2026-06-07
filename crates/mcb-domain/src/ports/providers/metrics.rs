//! Metrics provider ports.

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;

/// Key-value pairs for metric categorization.
pub type MetricLabels = HashMap<String, String>;
/// Specialized result for metrics operations.
pub type MetricsResult<T> = crate::Result<T>;

/// Errors occurring during metrics collection or submission.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MetricsError {
    /// The specified metric name does not exist.
    #[error("Metric not found: {name}")]
    NotFound {
        /// Name of the missing metric.
        name: String,
    },
    /// The metric configuration or value is invalid.
    #[error("Invalid metric: {message}")]
    Invalid {
        /// Human-readable error message.
        message: String,
    },
    /// The underlying metrics collection system failed.
    #[error("Metrics backend error: {message}")]
    Backend {
        /// Human-readable error message.
        message: String,
    },
}

pub(crate) fn labels_from<const N: usize>(pairs: [(&str, &str); N]) -> MetricLabels {
    pairs
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
}

/// Common interface for recording system metrics.
#[async_trait]
pub trait MetricsProvider: Send + Sync {
    /// Get the name of this metrics provider implementation.
    fn name(&self) -> &str;
    /// Increment a counter metric by 1.
    async fn increment(&self, name: &str, labels: &MetricLabels) -> MetricsResult<()>;
    /// Increment a counter metric by a specific amount.
    async fn increment_by(
        &self,
        name: &str,
        value: f64,
        labels: &MetricLabels,
    ) -> MetricsResult<()>;
    /// Set the current value of a gauge metric.
    async fn gauge(&self, name: &str, value: f64, labels: &MetricLabels) -> MetricsResult<()>;
    /// Record a value in a histogram distribution.
    async fn histogram(&self, name: &str, value: f64, labels: &MetricLabels) -> MetricsResult<()>;
}

/// Extension trait providing common metrics operations.
#[async_trait]
pub trait MetricsProviderExt: MetricsProvider + Send + Sync {
    /// Record the duration of an indexing operation.
    async fn record_index_time(&self, duration: Duration, collection: &str) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.histogram(
            "mcb_index_duration_seconds",
            duration.as_secs_f64(),
            &labels,
        )
        .await
    }

    /// Record the latency of a search operation.
    async fn record_search_latency(
        &self,
        duration: Duration,
        collection: &str,
    ) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.histogram(
            "mcb_search_duration_seconds",
            duration.as_secs_f64(),
            &labels,
        )
        .await
    }

    /// Record the latency of an embedding operation.
    async fn record_embedding_latency(
        &self,
        duration: Duration,
        provider: &str,
    ) -> MetricsResult<()> {
        let labels = labels_from([("provider", provider)]);
        self.histogram(
            "mcb_embedding_duration_seconds",
            duration.as_secs_f64(),
            &labels,
        )
        .await
    }

    /// Increment the count of indexed files in a collection.
    async fn increment_indexed_files(&self, collection: &str, count: u64) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.increment_by("mcb_indexed_files_total", count as f64, &labels)
            .await
    }

    /// Increment the search request counter for a collection.
    async fn increment_search_requests(&self, collection: &str) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.increment("mcb_search_requests_total", &labels).await
    }

    /// Set the number of concurrent active indexing jobs.
    async fn set_active_indexing_jobs(&self, count: u64) -> MetricsResult<()> {
        self.gauge(
            "mcb_active_indexing_jobs",
            count as f64,
            &std::collections::HashMap::new(),
        )
        .await
    }

    /// Set the current size (vector count) of a collection.
    async fn set_vector_store_size(&self, collection: &str, vectors: u64) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.gauge("mcb_vector_store_size", vectors as f64, &labels)
            .await
    }

    /// Record a cache hit or miss for a specific cache type.
    async fn record_cache_access(&self, hit: bool, cache_type: &str) -> MetricsResult<()> {
        let labels = labels_from([
            ("cache_type", cache_type),
            ("result", if hit { "hit" } else { "miss" }),
        ]);
        self.increment("mcb_cache_accesses_total", &labels).await
    }
}

// Implement extension trait for any type that implements MetricsProvider
impl<T: ?Sized + MetricsProvider> MetricsProviderExt for T {}
