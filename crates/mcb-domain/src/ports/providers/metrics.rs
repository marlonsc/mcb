//! Observability Metrics Provider Port
//!
//! Port for observability metrics collection providers. Implementations integrate
//! with monitoring systems like Prometheus, OpenTelemetry, or custom backends.
//!
//! This port is distinct from [`MetricsAnalysisProvider`](crate::ports::providers::metrics_analysis::MetricsAnalysisProvider)
//! which analyzes code complexity. This port collects runtime observability metrics.
//!
//! ## Metric Types
//!
//! | Type | Description | Example |
//! |------|-------------|---------|
//! | Counter | Monotonically increasing value | `requests_total` |
//! | Gauge | Value that can go up or down | `active_connections` |
//! | Histogram | Distribution of values | `request_duration_seconds` |
//!
//! ## Domain-Specific Metrics
//!
//! The trait includes convenience methods for common MCB operations:
//! - `record_index_time` - Time to index a codebase
//! - `record_search_latency` - Time to perform a search
//! - `record_embedding_latency` - Time to generate embeddings
//! - `increment_indexed_files` - Count of indexed files
//! - `increment_search_requests` - Count of search requests

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;

// ============================================================================
// Core Types
// ============================================================================

/// Labels/tags for a metric (key-value pairs)
pub type MetricLabels = HashMap<String, String>;

/// Result type for metrics operations
pub type MetricsResult<T> = crate::Result<T>;

/// Errors that can occur during metrics operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum MetricsError {
    /// Metric not found
    #[error("Metric not found: {name}")]
    NotFound { name: String },

    /// Invalid metric name or labels
    #[error("Invalid metric: {message}")]
    Invalid { message: String },

    /// Backend error
    #[error("Metrics backend error: {message}")]
    Backend { message: String },
}

// ============================================================================
// MetricsProvider Trait
// ============================================================================

/// Port for observability metrics collection
///
/// Implementations should be thread-safe and efficient, as metrics may be
/// recorded from multiple concurrent tasks.
///
/// ## Implementation Notes
///
/// - All methods are async to support remote metrics backends
/// - Labels should be used sparingly to avoid cardinality explosion
/// - Implementations should handle errors gracefully (logging but not failing)
///
/// ## Example
///
/// ```ignore
/// // Record a counter increment
/// metrics.increment("search_requests_total", &labels!("collection" => "my-project")).await?;
///
/// // Record a gauge value
/// metrics.gauge("active_indexing_jobs", 5.0, &labels!()).await?;
///
/// // Record a histogram observation
/// metrics.histogram("search_duration_seconds", 0.123, &labels!()).await?;
///
/// // Use domain-specific convenience methods
/// metrics.record_search_latency(Duration::from_millis(123), "my-collection").await?;
/// ```
#[async_trait]
pub trait MetricsProvider: Send + Sync {
    /// Provider name for identification
    fn name(&self) -> &str;

    // ========================================================================
    // Core Primitives (Prometheus-compatible)
    // ========================================================================

    /// Increment a counter by 1
    ///
    /// Counters are monotonically increasing values (e.g., total requests).
    async fn increment(&self, name: &str, labels: &MetricLabels) -> MetricsResult<()>;

    /// Increment a counter by a specific amount
    async fn increment_by(
        &self,
        name: &str,
        value: f64,
        labels: &MetricLabels,
    ) -> MetricsResult<()>;

    /// Set a gauge value
    ///
    /// Gauges represent a single value that can go up or down (e.g., temperature).
    async fn gauge(&self, name: &str, value: f64, labels: &MetricLabels) -> MetricsResult<()>;

    /// Record a histogram observation
    ///
    /// Histograms track the distribution of values (e.g., request latencies).
    async fn histogram(&self, name: &str, value: f64, labels: &MetricLabels) -> MetricsResult<()>;

    // ========================================================================
    // Domain-Specific Convenience Methods
    // ========================================================================

    /// Record time to index a codebase
    async fn record_index_time(&self, duration: Duration, collection: &str) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.histogram(
            "mcb_index_duration_seconds",
            duration.as_secs_f64(),
            &labels,
        )
        .await
    }

    /// Record search latency
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

    /// Record embedding generation latency
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

    /// Increment indexed files counter
    async fn increment_indexed_files(&self, collection: &str, count: u64) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.increment_by("mcb_indexed_files_total", count as f64, &labels)
            .await
    }

    /// Increment search requests counter
    async fn increment_search_requests(&self, collection: &str) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.increment("mcb_search_requests_total", &labels).await
    }

    /// Set current active indexing jobs gauge
    async fn set_active_indexing_jobs(&self, count: u64) -> MetricsResult<()> {
        self.gauge("mcb_active_indexing_jobs", count as f64, &HashMap::new())
            .await
    }

    /// Record vector store size
    async fn set_vector_store_size(&self, collection: &str, vectors: u64) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.gauge("mcb_vector_store_size", vectors as f64, &labels)
            .await
    }

    /// Record cache hit/miss
    async fn record_cache_access(&self, hit: bool, cache_type: &str) -> MetricsResult<()> {
        let labels = labels_from([
            ("cache_type", cache_type),
            ("result", if hit { "hit" } else { "miss" }),
        ]);
        self.increment("mcb_cache_accesses_total", &labels).await
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create labels from a slice of key-value pairs
fn labels_from<const N: usize>(pairs: [(&str, &str); N]) -> MetricLabels {
    pairs
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// Macro for creating labels inline
#[macro_export]
macro_rules! labels {
    () => {
        std::collections::HashMap::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert($key.to_string(), $value.to_string());)+
        map
    }};
}
