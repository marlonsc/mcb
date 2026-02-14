//! Unit tests for InstrumentedEmbeddingProvider decorator

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use mcb_application::decorators::InstrumentedEmbeddingProvider;
use mcb_domain::ports::admin::{PerformanceMetricsData, PerformanceMetricsInterface};
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use rstest::*;

struct InMemoryMetrics {
    query_count: AtomicU64,
    last_response_time: AtomicU64,
}

impl InMemoryMetrics {
    fn new() -> Self {
        Self {
            query_count: AtomicU64::new(0),
            last_response_time: AtomicU64::new(0),
        }
    }

    fn get_query_count(&self) -> u64 {
        self.query_count.load(Ordering::SeqCst)
    }
}

impl PerformanceMetricsInterface for InMemoryMetrics {
    fn uptime_secs(&self) -> u64 {
        0
    }

    fn record_query(&self, response_time_ms: u64, _success: bool, _cache_hit: bool) {
        self.query_count.fetch_add(1, Ordering::SeqCst);
        self.last_response_time
            .store(response_time_ms, Ordering::SeqCst);
    }

    fn update_active_connections(&self, _delta: i64) {}

    fn get_performance_metrics(&self) -> PerformanceMetricsData {
        PerformanceMetricsData {
            total_queries: self.query_count.load(Ordering::SeqCst),
            successful_queries: 0,
            failed_queries: 0,
            average_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            active_connections: 0,
            uptime_seconds: 0,
        }
    }
}

#[fixture]
async fn provider_context() -> (Arc<dyn EmbeddingProvider>, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(temp_dir.path().join("test.db"));
    let ctx = init_app(config).await.expect("init app context");
    (ctx.embedding_handle().get(), temp_dir)
}

#[fixture]
fn metrics() -> Arc<InMemoryMetrics> {
    Arc::new(InMemoryMetrics::new())
}

#[fixture]
async fn instrumented(
    #[future] provider_context: (Arc<dyn EmbeddingProvider>, tempfile::TempDir),
    metrics: Arc<InMemoryMetrics>,
) -> (
    InstrumentedEmbeddingProvider,
    Arc<InMemoryMetrics>,
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
        Arc<InMemoryMetrics>,
        tempfile::TempDir,
    ),
) {
    let (provider, metrics, _temp_dir) = instrumented.await;

    // Initial state
    assert_eq!(metrics.get_query_count(), 0);

    // Call embed
    let result = provider.embed("test").await;
    assert!(result.is_ok());
    assert_eq!(metrics.get_query_count(), 1);

    // Call embed_batch
    let result = provider
        .embed_batch(&["a".to_string(), "b".to_string()])
        .await;
    assert!(result.is_ok());
    assert_eq!(metrics.get_query_count(), 2);
}

#[rstest]
#[tokio::test]
async fn test_instrumented_delegates_to_inner(
    #[future] instrumented: (
        InstrumentedEmbeddingProvider,
        Arc<InMemoryMetrics>,
        tempfile::TempDir,
    ),
) {
    let (provider, _metrics, _temp_dir) = instrumented.await;

    // Check delegation
    assert_eq!(provider.dimensions(), 384); // Assuming fastembed default
    assert_eq!(provider.provider_name(), "fastembed");
    assert_eq!(provider.inner_provider_name(), "fastembed");
}
