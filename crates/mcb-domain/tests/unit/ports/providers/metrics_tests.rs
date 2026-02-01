//! Unit tests for metrics provider
//!
//! Tests for `NullMetricsObservabilityProvider` and metric helper macros.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// labels! macro is exported at crate root, labels_from is internal
use mcb_domain::labels;
use mcb_domain::ports::providers::metrics::{
    MetricLabels, MetricsProvider, NullMetricsObservabilityProvider,
};

#[tokio::test]
async fn test_null_provider() {
    let provider = NullMetricsObservabilityProvider::new();

    // All operations should succeed silently
    assert!(provider.increment("test", &HashMap::new()).await.is_ok());
    assert!(
        provider
            .increment_by("test", 5.0, &HashMap::new())
            .await
            .is_ok()
    );
    assert!(provider.gauge("test", 42.0, &HashMap::new()).await.is_ok());
    assert!(
        provider
            .histogram("test", 0.123, &HashMap::new())
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn test_domain_metrics() {
    let provider = NullMetricsObservabilityProvider::new();

    // Domain convenience methods should work
    assert!(
        provider
            .record_index_time(Duration::from_secs(5), "test-collection")
            .await
            .is_ok()
    );
    assert!(
        provider
            .record_search_latency(Duration::from_millis(100), "test-collection")
            .await
            .is_ok()
    );
    assert!(
        provider
            .record_embedding_latency(Duration::from_millis(50), "ollama")
            .await
            .is_ok()
    );
    assert!(
        provider
            .increment_indexed_files("test-collection", 100)
            .await
            .is_ok()
    );
    assert!(
        provider
            .increment_search_requests("test-collection")
            .await
            .is_ok()
    );
    assert!(provider.set_active_indexing_jobs(3).await.is_ok());
    assert!(
        provider
            .set_vector_store_size("test-collection", 10000)
            .await
            .is_ok()
    );
    assert!(
        provider
            .record_cache_access(true, "embedding")
            .await
            .is_ok()
    );
}

#[test]
fn test_labels_macro() {
    let empty: MetricLabels = labels!();
    assert!(empty.is_empty());

    let with_values = labels!("collection" => "test", "provider" => "ollama");
    assert_eq!(with_values.len(), 2);
    assert_eq!(with_values.get("collection"), Some(&"test".to_string()));
    assert_eq!(with_values.get("provider"), Some(&"ollama".to_string()));
}

// Note: test_labels_from was removed - labels_from is a private internal function

#[test]
fn test_provider_name() {
    let provider = NullMetricsObservabilityProvider::new();
    assert_eq!(provider.name(), "null");
}

#[test]
fn test_arc_construction() {
    let provider: Arc<dyn MetricsProvider> = NullMetricsObservabilityProvider::arc();
    assert_eq!(provider.name(), "null");
}
