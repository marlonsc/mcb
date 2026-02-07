//! Dill Catalog Configuration
//!
//! Configures the dill IoC container with all infrastructure services.

use crate::config::AppConfig;
use crate::di::admin::{
    CacheAdminInterface, CacheAdminService, EmbeddingAdminInterface, EmbeddingAdminService,
    LanguageAdminInterface, LanguageAdminService, VectorStoreAdminInterface,
    VectorStoreAdminService,
};
use crate::di::handles::{
    CacheProviderHandle, EmbeddingProviderHandle, LanguageProviderHandle, VectorStoreProviderHandle,
};
use crate::di::provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};
use crate::infrastructure::{
    admin::{AtomicPerformanceMetrics, DefaultIndexingOperations},
    events::TokioBroadcastEventBus,
    lifecycle::DefaultShutdownCoordinator,
};
use dill::{Catalog, CatalogBuilder};
use mcb_domain::error::Result;
use mcb_domain::ports::admin::{
    IndexingOperationsInterface, PerformanceMetricsInterface, ShutdownCoordinator,
};
use mcb_domain::ports::infrastructure::EventBusProvider;
use mcb_domain::ports::providers::{
    CacheProvider, EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};
use std::sync::Arc;
use tracing::info;

/// Build the dill Catalog with all application services
pub async fn build_catalog(config: AppConfig) -> Result<Catalog> {
    info!("Building dill Catalog");

    let config = Arc::new(config);

    // ========================================================================
    // Create Resolvers
    // ========================================================================

    let embedding_resolver = Arc::new(EmbeddingProviderResolver::new(config.clone()));
    let vector_store_resolver = Arc::new(VectorStoreProviderResolver::new(config.clone()));
    let cache_resolver = Arc::new(CacheProviderResolver::new(config.clone()));
    let language_resolver = Arc::new(LanguageProviderResolver::new(config.clone()));

    // ========================================================================
    // Resolve initial providers from config
    // ========================================================================

    let embedding_provider: Arc<dyn EmbeddingProvider> =
        embedding_resolver
            .resolve_from_config()
            .map_err(|e| mcb_domain::error::Error::configuration(format!("Embedding: {e}")))?;

    let vector_store_provider: Arc<dyn VectorStoreProvider> = vector_store_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("VectorStore: {e}")))?;

    let cache_provider: Arc<dyn CacheProvider> = cache_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("Cache: {e}")))?;

    let language_provider: Arc<dyn LanguageChunkingProvider> = language_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("Language: {e}")))?;

    // ========================================================================
    // Create Handles
    // ========================================================================

    let embedding_handle = Arc::new(EmbeddingProviderHandle::new(embedding_provider.clone()));
    let vector_store_handle = Arc::new(VectorStoreProviderHandle::new(
        vector_store_provider.clone(),
    ));
    let cache_handle = Arc::new(CacheProviderHandle::new(cache_provider.clone()));
    let language_handle = Arc::new(LanguageProviderHandle::new(language_provider.clone()));

    // ========================================================================
    // Create Admin Services
    // ========================================================================

    let embedding_admin: Arc<dyn EmbeddingAdminInterface> = Arc::new(EmbeddingAdminService::new(
        embedding_resolver.clone(),
        embedding_handle.clone(),
    ));
    let vector_store_admin: Arc<dyn VectorStoreAdminInterface> = Arc::new(
        VectorStoreAdminService::new(vector_store_resolver.clone(), vector_store_handle.clone()),
    );
    let cache_admin: Arc<dyn CacheAdminInterface> = Arc::new(CacheAdminService::new(
        cache_resolver.clone(),
        cache_handle.clone(),
    ));
    let language_admin: Arc<dyn LanguageAdminInterface> = Arc::new(LanguageAdminService::new(
        language_resolver.clone(),
        language_handle.clone(),
    ));

    // ========================================================================
    // Create Infrastructure Services
    // ========================================================================

    let event_bus: Arc<dyn EventBusProvider> = Arc::new(TokioBroadcastEventBus::new());
    let shutdown_coordinator: Arc<dyn ShutdownCoordinator> =
        Arc::new(DefaultShutdownCoordinator::new());
    let performance_metrics: Arc<dyn PerformanceMetricsInterface> =
        Arc::new(AtomicPerformanceMetrics::new());
    let indexing_operations: Arc<dyn IndexingOperationsInterface> =
        Arc::new(DefaultIndexingOperations::new());

    info!("Created infrastructure services");

    // ========================================================================
    // Build the Catalog
    // ========================================================================

    let catalog = CatalogBuilder::new()
        .add_value(config)
        .add_value(embedding_provider)
        .add_value(vector_store_provider)
        .add_value(cache_provider)
        .add_value(language_provider)
        .add_value(embedding_handle)
        .add_value(vector_store_handle)
        .add_value(cache_handle)
        .add_value(language_handle)
        .add_value(embedding_resolver)
        .add_value(vector_store_resolver)
        .add_value(cache_resolver)
        .add_value(language_resolver)
        .add_value(embedding_admin)
        .add_value(vector_store_admin)
        .add_value(cache_admin)
        .add_value(language_admin)
        .add_value(event_bus)
        .add_value(shutdown_coordinator)
        .add_value(performance_metrics)
        .add_value(indexing_operations)
        .build();

    Ok(catalog)
}
