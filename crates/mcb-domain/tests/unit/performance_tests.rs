//! Unit tests for performance metrics port

use mcb_domain::ports::infrastructure::{NullMetricsCollector, PerformanceMetricsCollector};
use std::time::Duration;

#[test]
fn test_null_metrics_collector_compiles() {
    let metrics = NullMetricsCollector::new();
    metrics.record_embedding_latency("ollama", Duration::from_millis(100), true);
    metrics.record_vectorstore_latency("milvus", Duration::from_millis(50), true);
    metrics.record_cache_hit("moka");
    metrics.record_cache_miss("moka");
    metrics.record_indexing_throughput(1000.0);
    metrics.record_batch_embedding("openai", 10, Duration::from_millis(500), true);
}

#[test]
fn test_null_metrics_default() {
    let metrics = NullMetricsCollector;
    metrics.record_embedding_latency("test", Duration::from_millis(1), false);
    assert!(
        std::mem::size_of_val(&metrics) == 0,
        "NullMetricsCollector should be zero-sized"
    );
}
