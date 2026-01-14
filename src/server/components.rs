//! Server Components Module
//!
//! Defines the components required to initialize McpServer.
//! This provides a clean configuration structure for dependency injection.

use arc_swap::ArcSwap;
use std::sync::Arc;

use crate::adapters::http_client::HttpClientProvider;
use crate::application::admin::traits::AdminService;
use crate::domain::ports::{IndexingOperationsInterface, PerformanceMetricsInterface};
use crate::domain::ports::{IndexingServiceInterface, SearchServiceInterface};
use crate::infrastructure::cache::SharedCacheProvider;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::events::SharedEventBusProvider;
use crate::infrastructure::limits::ResourceLimits;
use crate::infrastructure::logging::SharedLogBuffer;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;

/// Components required to initialize McpServer
///
/// This struct aggregates all dependencies needed to construct an McpServer instance.
/// It supports both DI-resolved services and manual construction fallbacks.
pub struct ServerComponents {
    /// Application configuration with hot-reload support
    pub config: Arc<ArcSwap<crate::infrastructure::config::Config>>,
    /// Optional cache provider for query caching
    pub cache_provider: Option<SharedCacheProvider>,
    /// Performance metrics tracking
    pub performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    /// Active indexing operations tracker
    pub indexing_operations: Arc<dyn IndexingOperationsInterface>,
    /// Admin service for system management
    pub admin_service: Arc<dyn AdminService>,
    /// Service provider for runtime provider creation
    pub service_provider: Arc<dyn ServiceProviderInterface>,
    /// Resource limits configuration
    pub resource_limits: Arc<ResourceLimits>,
    /// HTTP client for provider communication
    pub http_client: Arc<dyn HttpClientProvider>,
    /// Event bus for decoupled communication
    pub event_bus: SharedEventBusProvider,
    /// Log buffer for real-time monitoring
    pub log_buffer: SharedLogBuffer,
    /// System metrics collector
    pub system_collector: Arc<dyn SystemMetricsCollectorInterface>,
    /// Optional DI-resolved indexing service (skips manual construction if provided)
    pub indexing_service: Option<Arc<dyn IndexingServiceInterface>>,
    /// Optional DI-resolved search service (skips manual construction if provided)
    pub search_service: Option<Arc<dyn SearchServiceInterface>>,
}

impl ServerComponents {
    /// Create a builder for constructing ServerComponents
    pub fn builder() -> ServerComponentsBuilder {
        ServerComponentsBuilder::default()
    }
}

/// Builder for ServerComponents
///
/// Provides a fluent API for constructing ServerComponents with validation.
#[derive(Default)]
pub struct ServerComponentsBuilder {
    config: Option<Arc<ArcSwap<crate::infrastructure::config::Config>>>,
    cache_provider: Option<SharedCacheProvider>,
    performance_metrics: Option<Arc<dyn PerformanceMetricsInterface>>,
    indexing_operations: Option<Arc<dyn IndexingOperationsInterface>>,
    admin_service: Option<Arc<dyn AdminService>>,
    service_provider: Option<Arc<dyn ServiceProviderInterface>>,
    resource_limits: Option<Arc<ResourceLimits>>,
    http_client: Option<Arc<dyn HttpClientProvider>>,
    event_bus: Option<SharedEventBusProvider>,
    log_buffer: Option<SharedLogBuffer>,
    system_collector: Option<Arc<dyn SystemMetricsCollectorInterface>>,
    indexing_service: Option<Arc<dyn IndexingServiceInterface>>,
    search_service: Option<Arc<dyn SearchServiceInterface>>,
}

impl ServerComponentsBuilder {
    /// Set the configuration
    pub fn config(mut self, config: Arc<ArcSwap<crate::infrastructure::config::Config>>) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the cache provider
    pub fn cache_provider(mut self, cache_provider: SharedCacheProvider) -> Self {
        self.cache_provider = Some(cache_provider);
        self
    }

    /// Set the performance metrics
    pub fn performance_metrics(
        mut self,
        performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    ) -> Self {
        self.performance_metrics = Some(performance_metrics);
        self
    }

    /// Set the indexing operations tracker
    pub fn indexing_operations(
        mut self,
        indexing_operations: Arc<dyn IndexingOperationsInterface>,
    ) -> Self {
        self.indexing_operations = Some(indexing_operations);
        self
    }

    /// Set the admin service
    pub fn admin_service(mut self, admin_service: Arc<dyn AdminService>) -> Self {
        self.admin_service = Some(admin_service);
        self
    }

    /// Set the service provider
    pub fn service_provider(mut self, service_provider: Arc<dyn ServiceProviderInterface>) -> Self {
        self.service_provider = Some(service_provider);
        self
    }

    /// Set the resource limits
    pub fn resource_limits(mut self, resource_limits: Arc<ResourceLimits>) -> Self {
        self.resource_limits = Some(resource_limits);
        self
    }

    /// Set the HTTP client
    pub fn http_client(mut self, http_client: Arc<dyn HttpClientProvider>) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Set the event bus
    pub fn event_bus(mut self, event_bus: SharedEventBusProvider) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Set the log buffer
    pub fn log_buffer(mut self, log_buffer: SharedLogBuffer) -> Self {
        self.log_buffer = Some(log_buffer);
        self
    }

    /// Set the system collector
    pub fn system_collector(
        mut self,
        system_collector: Arc<dyn SystemMetricsCollectorInterface>,
    ) -> Self {
        self.system_collector = Some(system_collector);
        self
    }

    /// Set the DI-resolved indexing service
    pub fn indexing_service(mut self, indexing_service: Arc<dyn IndexingServiceInterface>) -> Self {
        self.indexing_service = Some(indexing_service);
        self
    }

    /// Set the DI-resolved search service
    pub fn search_service(mut self, search_service: Arc<dyn SearchServiceInterface>) -> Self {
        self.search_service = Some(search_service);
        self
    }

    /// Build the ServerComponents
    ///
    /// # Panics
    ///
    /// Panics if required fields are not set.
    pub fn build(self) -> ServerComponents {
        ServerComponents {
            config: self.config.expect("config is required"),
            cache_provider: self.cache_provider,
            performance_metrics: self
                .performance_metrics
                .expect("performance_metrics is required"),
            indexing_operations: self
                .indexing_operations
                .expect("indexing_operations is required"),
            admin_service: self.admin_service.expect("admin_service is required"),
            service_provider: self.service_provider.expect("service_provider is required"),
            resource_limits: self.resource_limits.expect("resource_limits is required"),
            http_client: self.http_client.expect("http_client is required"),
            event_bus: self.event_bus.expect("event_bus is required"),
            log_buffer: self.log_buffer.expect("log_buffer is required"),
            system_collector: self.system_collector.expect("system_collector is required"),
            indexing_service: self.indexing_service,
            search_service: self.search_service,
        }
    }
}
