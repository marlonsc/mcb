//! Admin service unit tests
//!
//! These tests verify the AdminService trait implementation contract

use mcp_context_browser::admin::service::AdminService;

// Import dependencies for real service creation
use arc_swap::ArcSwap;
use mcp_context_browser::admin::service::AdminServiceImpl;
use mcp_context_browser::application::search::SearchService;
use mcp_context_browser::infrastructure::di::factory::{ServiceProvider, ServiceProviderInterface};
use mcp_context_browser::infrastructure::config::Config;
use mcp_context_browser::infrastructure::events::{EventBus, SharedEventBus};
use mcp_context_browser::infrastructure::logging::SharedLogBuffer;
use mcp_context_browser::infrastructure::metrics::system::{SystemMetricsCollector, SystemMetricsCollectorInterface};
use mcp_context_browser::server::metrics::{McpPerformanceMetrics, PerformanceMetricsInterface};
use mcp_context_browser::server::operations::{McpIndexingOperations, IndexingOperationsInterface};
use mcp_context_browser::adapters::http_client::{NullHttpClientPool, HttpClientProvider};
use std::sync::Arc;

/// Test infrastructure for setting up real services
#[allow(dead_code)]
pub struct TestInfrastructure {
    pub admin_service: Arc<dyn AdminService>,
    pub config: Arc<ArcSwap<Config>>,
    pub event_bus: SharedEventBus,
    pub log_buffer: SharedLogBuffer,
    pub performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    pub indexing_operations: Arc<dyn IndexingOperationsInterface>,
    pub service_provider: Arc<dyn ServiceProviderInterface>,
    pub system_collector: Arc<dyn SystemMetricsCollectorInterface>,
    pub http_client: Arc<dyn HttpClientProvider>,
    pub search_service: Option<Arc<SearchService>>,
}

impl TestInfrastructure {
    /// Create a new test infrastructure with real services
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create test configuration
        let config = Self::create_test_config();
        let config_arc = Arc::new(ArcSwap::from_pointee(config));

        // Create shared components
        let event_bus: SharedEventBus = Arc::new(EventBus::with_default_capacity());
        // Keep a receiver alive to prevent the channel from being considered closed
        let _receiver = event_bus.subscribe();
        let log_buffer = mcp_context_browser::infrastructure::logging::create_shared_log_buffer(1000);

        // Create service components
        let performance_metrics: Arc<dyn PerformanceMetricsInterface> = Arc::new(McpPerformanceMetrics::default());
        let indexing_operations: Arc<dyn IndexingOperationsInterface> = Arc::new(McpIndexingOperations::default());
        let service_provider: Arc<dyn ServiceProviderInterface> = Arc::new(ServiceProvider::new());
        let system_collector: Arc<dyn SystemMetricsCollectorInterface> = Arc::new(SystemMetricsCollector::new());

        // Create HTTP client
        let http_client: Arc<dyn HttpClientProvider> = Arc::new(NullHttpClientPool::new());

        // Create admin service with all dependencies
        let admin_service = Arc::new(AdminServiceImpl::new(
            Arc::clone(&performance_metrics),
            Arc::clone(&indexing_operations),
            Arc::clone(&service_provider),
            Arc::clone(&system_collector),
            Arc::clone(&http_client),
            event_bus.clone(),
            log_buffer.clone(),
            Arc::clone(&config_arc),
        )) as Arc<dyn AdminService>;

        Ok(Self {
            admin_service,
            config: config_arc,
            event_bus,
            log_buffer,
            performance_metrics,
            indexing_operations,
            service_provider,
            system_collector,
            http_client,
            search_service: None,
        })
    }

    fn create_test_config() -> Config {
        Config::default()
    }
}

// Test service creation function
async fn create_test_admin_service() -> Arc<dyn AdminService> {
    let test_infra = TestInfrastructure::new().await
        .expect("Failed to create test infrastructure");
    test_infra.admin_service
}

// ============================================================================
// System Information Tests
// ============================================================================

#[tokio::test]
async fn test_service_get_system_info() {
    let service = create_test_admin_service().await;
    let result = service.get_system_info().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_get_providers() {
    let service = create_test_admin_service().await;
    let result = service.get_providers().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_get_indexing_status() {
    let service = create_test_admin_service().await;
    let result = service.get_indexing_status().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_get_performance_metrics() {
    let service = create_test_admin_service().await;
    let result = service.get_performance_metrics().await;
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.cache_hit_rate >= 0.0 && metrics.cache_hit_rate <= 1.0);
}
