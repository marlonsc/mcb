//! Unit tests for InstrumentedEmbeddingProvider decorator

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use mcb_application::decorators::InstrumentedEmbeddingProvider;
use mcb_domain::ports::admin::{PerformanceMetricsData, PerformanceMetricsInterface};
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;

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

async fn create_fastembed_provider() -> (Arc<dyn EmbeddingProvider>, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(temp_dir.path().join("test.db"));
    let ctx = init_app(config).await.expect("init app context");
    (ctx.embedding_handle().get(), temp_dir)
}

#[tokio::test]
async fn test_instrumented_records_metrics() {
    let (inner, _temp_dir) = create_fastembed_provider().await;
    let metrics = Arc::new(InMemoryMetrics::new());

    let instrumented = InstrumentedEmbeddingProvider::new(
        inner,
        Arc::clone(&metrics) as Arc<dyn PerformanceMetricsInterface>,
    );

    // Initial state
    assert_eq!(metrics.get_query_count(), 0);

    // Call embed
    let result = instrumented.embed("test").await;
    assert!(result.is_ok());
    assert_eq!(metrics.get_query_count(), 1);

    // Call embed_batch
    let result = instrumented
        .embed_batch(&["a".to_string(), "b".to_string()])
        .await;
    assert!(result.is_ok());
    assert_eq!(metrics.get_query_count(), 2);
}

#[tokio::test]
async fn test_instrumented_delegates_to_inner() {
    let (inner, _temp_dir) = create_fastembed_provider().await;
    let metrics = Arc::new(InMemoryMetrics::new());

    let instrumented = InstrumentedEmbeddingProvider::new(
        inner,
        Arc::clone(&metrics) as Arc<dyn PerformanceMetricsInterface>,
    );

    // Check delegation
    assert_eq!(instrumented.dimensions(), 384);
    assert_eq!(instrumented.provider_name(), "fastembed");
    assert_eq!(instrumented.inner_provider_name(), "fastembed");
}
