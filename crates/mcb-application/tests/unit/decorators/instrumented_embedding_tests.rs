//! Unit tests for InstrumentedEmbeddingProvider decorator

use std::sync::Arc;

use mcb_application::decorators::InstrumentedEmbeddingProvider;
use mcb_domain::ports::admin::PerformanceMetricsInterface;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use mcb_infrastructure::infrastructure::admin::AtomicPerformanceMetrics;
use rstest::*;

#[fixture]
async fn provider_context() -> (Arc<dyn EmbeddingProvider>, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(temp_dir.path().join("test.db"));
    let ctx = init_app(config).await.expect("init app context");
    (ctx.embedding_handle().get(), temp_dir)
}

#[fixture]
fn metrics() -> Arc<AtomicPerformanceMetrics> {
    AtomicPerformanceMetrics::new_shared()
}

#[fixture]
async fn instrumented(
    #[future] provider_context: (Arc<dyn EmbeddingProvider>, tempfile::TempDir),
    metrics: Arc<AtomicPerformanceMetrics>,
) -> (
    InstrumentedEmbeddingProvider,
    Arc<AtomicPerformanceMetrics>,
    tempfile::TempDir,
) {
    let (inner, temp_dir) = provider_context.await;
    let provider = InstrumentedEmbeddingProvider::new(
        inner,
        Arc::clone(&metrics) as Arc<dyn PerformanceMetricsInterface>,
    );
    (provider, metrics, temp_dir)
}

#[rstest]
#[tokio::test]
async fn test_instrumented_records_metrics(
    #[future] instrumented: (
        InstrumentedEmbeddingProvider,
        Arc<AtomicPerformanceMetrics>,
        tempfile::TempDir,
    ),
) {
    let (provider, metrics, _temp_dir) = instrumented.await;

    // Initial state
    assert_eq!(metrics.get_performance_metrics().total_queries, 0);

    // Call embed
    let result = provider.embed("test").await;
    assert!(result.is_ok());
    assert_eq!(metrics.get_performance_metrics().total_queries, 1);

    // Call embed_batch
    let result = provider
        .embed_batch(&["a".to_string(), "b".to_string()])
        .await;
    assert!(result.is_ok());
    assert_eq!(metrics.get_performance_metrics().total_queries, 2);
}

#[rstest]
#[tokio::test]
async fn test_instrumented_delegates_to_inner(
    #[future] instrumented: (
        InstrumentedEmbeddingProvider,
        Arc<AtomicPerformanceMetrics>,
        tempfile::TempDir,
    ),
) {
    let (provider, _metrics, _temp_dir) = instrumented.await;

    // Check delegation
    assert_eq!(provider.dimensions(), 384); // Assuming fastembed default
    assert_eq!(provider.provider_name(), "fastembed");
    assert_eq!(provider.inner_provider_name(), "fastembed");
}
