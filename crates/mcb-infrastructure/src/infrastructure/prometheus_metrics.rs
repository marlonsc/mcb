//! Prometheus Performance Metrics Implementation
//!
//! Implements the PerformanceMetricsCollector port using Prometheus metrics.
//! Provides histograms for latency and counters for cache operations.
//!
//! ## Metrics Exported
//!
//! | Metric | Type | Labels | Description |
//! |--------|------|--------|-------------|
//! | `mcb_embedding_latency_seconds` | Histogram | `provider`, `success` | Embedding operation duration |
//! | `mcb_vectorstore_query_latency_seconds` | Histogram | `provider`, `success` | Vector store query duration |
//! | `mcb_cache_hits_total` | Counter | `provider` | Cache hit count |
//! | `mcb_cache_misses_total` | Counter | `provider` | Cache miss count |
//! | `mcb_indexing_throughput_chunks_per_second` | Gauge | - | Current indexing throughput |
//! | `mcb_batch_embedding_duration_seconds` | Histogram | `provider`, `success` | Batch embedding duration |
//! | `mcb_batch_embedding_size` | Histogram | `provider` | Batch size distribution |

use std::sync::OnceLock;
use std::time::Duration;

use mcb_domain::ports::admin::{PerformanceMetricsData, PerformanceMetricsInterface};
use mcb_domain::ports::infrastructure::PerformanceMetricsCollector;
use prometheus::{
    CounterVec, Gauge, HistogramVec, register_counter_vec, register_gauge, register_histogram_vec,
};

use crate::constants::{METRICS_BATCH_SIZE_BUCKETS, METRICS_LATENCY_BUCKETS};

/// Global metrics registry holder
static METRICS: OnceLock<Result<PrometheusMetricsInner, String>> = OnceLock::new();

/// Inner metrics structure holding all Prometheus metrics
struct PrometheusMetricsInner {
    embedding_latency: HistogramVec,
    vectorstore_latency: HistogramVec,
    cache_hits: CounterVec,
    cache_misses: CounterVec,
    indexing_throughput: Gauge,
    batch_embedding_duration: HistogramVec,
    batch_embedding_size: HistogramVec,
}

impl PrometheusMetricsInner {
    /// Create and register all metrics
    fn try_new() -> Result<Self, String> {
        Ok(Self {
            embedding_latency: Self::register_latency_histogram(
                "mcb_embedding_latency_seconds",
                "Embedding operation latency in seconds",
            )?,
            vectorstore_latency: Self::register_latency_histogram(
                "mcb_vectorstore_query_latency_seconds",
                "Vector store query latency in seconds",
            )?,
            cache_hits: Self::register_counter(
                "mcb_cache_hits_total",
                "Total number of cache hits",
            )?,
            cache_misses: Self::register_counter(
                "mcb_cache_misses_total",
                "Total number of cache misses",
            )?,
            indexing_throughput: Self::register_throughput_gauge()?,
            batch_embedding_duration: Self::register_latency_histogram(
                "mcb_batch_embedding_duration_seconds",
                "Batch embedding operation duration in seconds",
            )?,
            batch_embedding_size: Self::register_batch_size_histogram()?,
        })
    }

    fn register_latency_histogram(name: &str, help: &str) -> Result<HistogramVec, String> {
        register_histogram_vec!(
            name,
            help,
            &["provider", "success"],
            METRICS_LATENCY_BUCKETS.to_vec()
        )
        .map_err(|e| format!("Failed to register {name}: {e}"))
    }

    fn register_counter(name: &str, help: &str) -> Result<CounterVec, String> {
        register_counter_vec!(name, help, &["provider"])
            .map_err(|e| format!("Failed to register {name}: {e}"))
    }

    fn register_throughput_gauge() -> Result<Gauge, String> {
        register_gauge!(
            "mcb_indexing_throughput_chunks_per_second",
            "Current indexing throughput in chunks per second"
        )
        .map_err(|e| format!("Failed to register throughput gauge: {e}"))
    }

    fn register_batch_size_histogram() -> Result<HistogramVec, String> {
        register_histogram_vec!(
            "mcb_batch_embedding_size",
            "Batch embedding size distribution",
            &["provider"],
            METRICS_BATCH_SIZE_BUCKETS.to_vec()
        )
        .map_err(|e| format!("Failed to register batch size histogram: {e}"))
    }
}

/// Prometheus-based performance metrics collector
///
/// Thread-safe implementation using global static metrics.
/// Metrics are registered once and reused across all instances.
pub struct PrometheusPerformanceMetrics;

impl PrometheusPerformanceMetrics {
    /// Create a new Prometheus metrics collector
    ///
    /// Initializes global metrics if not already registered.
    /// Returns None if metric registration fails.
    pub fn try_new() -> Result<Self, String> {
        let result = METRICS.get_or_init(PrometheusMetricsInner::try_new);
        match result {
            Ok(_) => Ok(Self),
            Err(e) => Err(e.clone()),
        }
    }

    /// Create a new Prometheus metrics collector
    ///
    /// Initializes global metrics if not already registered.
    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|e| {
            tracing::error!("Failed to initialize Prometheus metrics: {}", e);
            Self
        })
    }

    /// Get the global metrics instance, if initialized successfully
    fn metrics(&self) -> Option<&PrometheusMetricsInner> {
        METRICS.get().and_then(|r| r.as_ref().ok())
    }

    fn success_label(success: bool) -> &'static str {
        if success { "true" } else { "false" }
    }
}

impl Default for PrometheusPerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetricsCollector for PrometheusPerformanceMetrics {
    fn record_embedding_latency(&self, provider: &str, duration: Duration, success: bool) {
        if let Some(metrics) = self.metrics() {
            metrics
                .embedding_latency
                .with_label_values(&[provider, Self::success_label(success)])
                .observe(duration.as_secs_f64());
        }
    }

    fn record_vectorstore_latency(&self, provider: &str, duration: Duration, success: bool) {
        if let Some(metrics) = self.metrics() {
            metrics
                .vectorstore_latency
                .with_label_values(&[provider, Self::success_label(success)])
                .observe(duration.as_secs_f64());
        }
    }

    fn record_cache_hit(&self, provider: &str) {
        if let Some(metrics) = self.metrics() {
            metrics.cache_hits.with_label_values(&[provider]).inc();
        }
    }

    fn record_cache_miss(&self, provider: &str) {
        if let Some(metrics) = self.metrics() {
            metrics.cache_misses.with_label_values(&[provider]).inc();
        }
    }

    fn record_indexing_throughput(&self, chunks_per_second: f64) {
        if let Some(metrics) = self.metrics() {
            metrics.indexing_throughput.set(chunks_per_second);
        }
    }

    fn record_batch_embedding(
        &self,
        provider: &str,
        batch_size: usize,
        duration: Duration,
        success: bool,
    ) {
        if let Some(metrics) = self.metrics() {
            metrics
                .batch_embedding_duration
                .with_label_values(&[provider, Self::success_label(success)])
                .observe(duration.as_secs_f64());

            metrics
                .batch_embedding_size
                .with_label_values(&[provider])
                .observe(batch_size as f64);
        }
    }
}

impl PerformanceMetricsInterface for PrometheusPerformanceMetrics {
    fn uptime_secs(&self) -> u64 {
        0
    }

    fn record_query(&self, _response_time_ms: u64, _success: bool, _cache_hit: bool) {}

    fn update_active_connections(&self, _delta: i64) {}

    fn get_performance_metrics(&self) -> PerformanceMetricsData {
        PerformanceMetricsData {
            total_queries: 0,
            successful_queries: 0,
            failed_queries: 0,
            average_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            active_connections: 0,
            uptime_seconds: 0,
        }
    }
}

/// Export all Prometheus metrics as text
///
/// Returns metrics in Prometheus text format for the /metrics endpoint.
pub fn export_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).ok();
    String::from_utf8(buffer).unwrap_or_default()
}
