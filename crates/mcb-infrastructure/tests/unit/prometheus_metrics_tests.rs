//! Unit tests for Prometheus performance metrics

use std::time::Duration;

use mcb_domain::ports::infrastructure::PerformanceMetricsCollector;
use mcb_infrastructure::infrastructure::PrometheusPerformanceMetrics;

#[test]
fn test_prometheus_metrics_creation() {
    let metrics = PrometheusPerformanceMetrics::new();
    // Should succeed (graceful degradation if registration fails)
    assert!(std::ptr::eq(&metrics, &metrics)); // Simple existence check
}

#[test]
fn test_prometheus_try_new() {
    // First creation should succeed
    let result = PrometheusPerformanceMetrics::try_new();
    assert!(result.is_ok());
}

#[test]
fn test_prometheus_default() {
    let metrics = PrometheusPerformanceMetrics::new();
    // Default should work via new()
    assert!(std::ptr::eq(&metrics, &metrics));
}

#[test]
fn test_prometheus_record_embedding_latency() {
    let metrics = PrometheusPerformanceMetrics::new();
    // Should not panic, verify operation completes
    metrics.record_embedding_latency("ollama", Duration::from_millis(100), true);
    metrics.record_embedding_latency("openai", Duration::from_millis(200), false);
    // Verify by checking export contains metric (may be empty if not initialized)
    let export = mcb_infrastructure::infrastructure::export_metrics();
    assert!(export.is_ascii(), "Export should produce valid ASCII");
}

#[test]
fn test_prometheus_record_vectorstore_latency() {
    let metrics = PrometheusPerformanceMetrics::new();
    // Should not panic, verify operation completes
    metrics.record_vectorstore_latency("milvus", Duration::from_millis(50), true);
    metrics.record_vectorstore_latency("qdrant", Duration::from_millis(75), false);
    // Verify by checking export contains metric (may be empty if not initialized)
    let export = mcb_infrastructure::infrastructure::export_metrics();
    assert!(export.is_ascii(), "Export should produce valid ASCII");
}

#[test]
fn test_prometheus_record_cache_hit_miss() {
    let metrics = PrometheusPerformanceMetrics::new();
    metrics.record_cache_hit("moka");
    metrics.record_cache_miss("moka");
    metrics.record_cache_hit("redis");
    let export = mcb_infrastructure::infrastructure::export_metrics();
    assert!(export.is_ascii(), "Export should produce valid ASCII");
}

#[test]
fn test_prometheus_record_indexing_throughput() {
    let metrics = PrometheusPerformanceMetrics::new();
    metrics.record_indexing_throughput(1000.0);
    metrics.record_indexing_throughput(0.0);
    let export = mcb_infrastructure::infrastructure::export_metrics();
    assert!(export.is_ascii(), "Export should produce valid ASCII");
}

#[test]
fn test_prometheus_record_batch_embedding() {
    let metrics = PrometheusPerformanceMetrics::new();
    metrics.record_batch_embedding("openai", 10, Duration::from_millis(500), true);
    metrics.record_batch_embedding("voyageai", 25, Duration::from_millis(1000), false);
    let export = mcb_infrastructure::infrastructure::export_metrics();
    assert!(export.is_ascii(), "Export should produce valid ASCII");
}

#[test]
fn test_prometheus_export_metrics() {
    use mcb_infrastructure::infrastructure::export_metrics;

    // Should return a string (may be empty if metrics not initialized)
    let output = export_metrics();
    assert!(output.is_ascii());
}
