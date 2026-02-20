//! Unit tests for `InstrumentedEmbeddingProvider` decorator

use rstest::*;
use std::sync::Arc;

use mcb_application::decorators::InstrumentedEmbeddingProvider;
use mcb_domain::ports::{EmbeddingProvider, PerformanceMetricsInterface};
use mcb_infrastructure::infrastructure::admin::AtomicPerformanceMetrics;

use crate::utils::shared_context::try_shared_app_context;

#[fixture]
async fn provider_context() -> Option<Arc<dyn EmbeddingProvider>> {
    try_shared_app_context().map(|ctx| ctx.embedding_handle().get())
}

#[fixture]
fn metrics() -> Arc<AtomicPerformanceMetrics> {
    AtomicPerformanceMetrics::new_shared()
}

#[fixture]
async fn instrumented(
    #[future] provider_context: Option<Arc<dyn EmbeddingProvider>>,
    metrics: Arc<AtomicPerformanceMetrics>,
) -> Option<(InstrumentedEmbeddingProvider, Arc<AtomicPerformanceMetrics>)> {
    let inner = provider_context.await?;
    let provider = InstrumentedEmbeddingProvider::new(
        inner,
        Arc::clone(&metrics) as Arc<dyn PerformanceMetricsInterface>,
    );
    Some((provider, metrics))
}

#[rstest]
#[tokio::test]
async fn test_instrumented_records_metrics(
    #[future] instrumented: Option<(InstrumentedEmbeddingProvider, Arc<AtomicPerformanceMetrics>)>,
) {
    let Some((provider, metrics)) = instrumented.await else {
        tracing::warn!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };

    assert_eq!(metrics.get_performance_metrics().total_queries, 0);

    let result = provider.embed("test").await;
    assert!(result.is_ok());
    assert_eq!(metrics.get_performance_metrics().total_queries, 1);

    let result = provider
        .embed_batch(&["a".to_owned(), "b".to_owned()])
        .await;
    assert!(result.is_ok());
    assert_eq!(metrics.get_performance_metrics().total_queries, 2);
}

#[rstest]
#[tokio::test]
async fn test_instrumented_delegates_to_inner(
    #[future] instrumented: Option<(InstrumentedEmbeddingProvider, Arc<AtomicPerformanceMetrics>)>,
) {
    let Some((provider, _metrics)) = instrumented.await else {
        tracing::warn!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };

    assert_eq!(provider.dimensions(), 384);
    assert_eq!(provider.provider_name(), "fastembed");
    assert_eq!(provider.inner_provider_name(), "fastembed");
}
