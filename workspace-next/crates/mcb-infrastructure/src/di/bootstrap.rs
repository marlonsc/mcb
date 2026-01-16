//! DI Container Bootstrap
//!
//! Provides the main dependency injection container that wires together
//! all infrastructure components following Clean Architecture principles.

use crate::adapters::admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use crate::adapters::providers::{EmbeddingProviderFactory, VectorStoreProviderFactory};
use crate::cache::provider::SharedCacheProvider;
use crate::cache::CacheProviderFactory;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::health::{checkers, HealthRegistry};
use mcb_domain::error::Result;
use mcb_domain::ports::admin::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};
use std::sync::Arc;
use tracing::info;

/// Storage and caching components
#[derive(Clone)]
pub struct StorageComponents {
    pub cache: SharedCacheProvider,
    pub crypto: CryptoService,
}

/// Provider components for AI services
#[derive(Clone)]
pub struct ProviderComponents {
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    pub vector_store_provider: Arc<dyn VectorStoreProvider>,
}

/// Administrative service components
#[derive(Clone)]
pub struct AdminComponents {
    pub performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    pub indexing_operations: Arc<dyn IndexingOperationsInterface>,
}

/// Core infrastructure components container
#[derive(Clone)]
pub struct InfrastructureComponents {
    pub storage: StorageComponents,
    pub providers: ProviderComponents,
    pub admin: AdminComponents,
    pub health: HealthRegistry,
    pub config: AppConfig,
}

impl InfrastructureComponents {
    /// Create a new infrastructure components container
    pub async fn new(config: AppConfig) -> Result<Self> {
        let cache = Self::create_cache_provider(&config).await?;
        let crypto = Self::create_crypto_service(&config)?;
        let health = Self::create_health_registry().await?;
        let embedding_provider = Self::create_embedding_provider(&config)?;
        let vector_store_provider = Self::create_vector_store_provider(&config, &crypto)?;
        let (performance_metrics, indexing_operations) = Self::create_admin_services();

        Ok(Self {
            storage: StorageComponents { cache, crypto },
            providers: ProviderComponents {
                embedding_provider,
                vector_store_provider,
            },
            admin: AdminComponents {
                performance_metrics,
                indexing_operations,
            },
            health,
            config,
        })
    }

    /// Create cache provider from configuration
    async fn create_cache_provider(config: &AppConfig) -> Result<SharedCacheProvider> {
        if config.cache.enabled {
            CacheProviderFactory::create_from_config(&config.cache).await
        } else {
            Ok(CacheProviderFactory::create_null())
        }
    }

    /// Create crypto service from configuration
    fn create_crypto_service(config: &AppConfig) -> Result<CryptoService> {
        // AES-GCM requires exactly 32 bytes for the key
        let master_key = if config.auth.jwt_secret.len() >= 32 {
            // Use first 32 bytes of the JWT secret as the master key
            config.auth.jwt_secret.as_bytes()[..32].to_vec()
        } else {
            CryptoService::generate_master_key()
        };

        CryptoService::new(master_key)
    }

    /// Create and configure health registry
    async fn create_health_registry() -> Result<HealthRegistry> {
        let health = HealthRegistry::new();

        // Register system health checker
        let system_checker = checkers::SystemHealthChecker::new();
        health
            .register_checker("system".to_string(), system_checker)
            .await;

        // Register database health checker if configured
        // (This would be expanded based on actual database configuration)

        Ok(health)
    }

    /// Create embedding provider from configuration
    fn create_embedding_provider(config: &AppConfig) -> Result<Arc<dyn EmbeddingProvider>> {
        if let Some((name, embedding_config)) = config.embedding.iter().next() {
            info!(provider = name, "Creating embedding provider from config");
            EmbeddingProviderFactory::create(embedding_config, None)
        } else {
            info!("No embedding provider configured, using null provider");
            Ok(EmbeddingProviderFactory::create_null())
        }
    }

    /// Create vector store provider from configuration
    fn create_vector_store_provider(
        config: &AppConfig,
        crypto: &CryptoService,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        if let Some((name, vector_config)) = config.vector_store.iter().next() {
            info!(
                provider = name,
                "Creating vector store provider from config"
            );
            VectorStoreProviderFactory::create(vector_config, Some(crypto))
        } else {
            info!("No vector store provider configured, using in-memory provider");
            Ok(VectorStoreProviderFactory::create_in_memory())
        }
    }

    /// Create admin service implementations
    fn create_admin_services() -> (Arc<dyn PerformanceMetricsInterface>, Arc<dyn IndexingOperationsInterface>) {
        let performance_metrics: Arc<dyn PerformanceMetricsInterface> =
            Arc::new(AtomicPerformanceMetrics::new());
        let indexing_operations: Arc<dyn IndexingOperationsInterface> =
            Arc::new(DefaultIndexingOperations::new());

        (performance_metrics, indexing_operations)
    }

    /// Get the cache provider
    pub fn cache(&self) -> &SharedCacheProvider {
        &self.cache
    }

    /// Get the crypto service
    pub fn crypto(&self) -> &CryptoService {
        &self.crypto
    }

    /// Get the health registry
    pub fn health(&self) -> &HealthRegistry {
        &self.health
    }

    /// Get the embedding provider
    pub fn embedding_provider(&self) -> &Arc<dyn EmbeddingProvider> {
        &self.providers.embedding_provider
    }

    /// Get the vector store provider
    pub fn vector_store_provider(&self) -> &Arc<dyn VectorStoreProvider> {
        &self.providers.vector_store_provider
    }

    /// Get the performance metrics service
    pub fn performance_metrics(&self) -> &Arc<dyn PerformanceMetricsInterface> {
        &self.admin.performance_metrics
    }

    /// Get the indexing operations service
    pub fn indexing_operations(&self) -> &Arc<dyn IndexingOperationsInterface> {
        &self.admin.indexing_operations
    }

    /// Get the configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get the embedding provider
    pub fn embedding_provider(&self) -> &Arc<dyn EmbeddingProvider> {
        &self.embedding_provider
    }

    /// Get the vector store provider
    pub fn vector_store_provider(&self) -> &Arc<dyn VectorStoreProvider> {
        &self.vector_store_provider
    }

    /// Get the performance metrics tracker
    pub fn performance_metrics(&self) -> &Arc<dyn PerformanceMetricsInterface> {
        &self.performance_metrics
    }

    /// Get the indexing operations tracker
    pub fn indexing_operations(&self) -> &Arc<dyn IndexingOperationsInterface> {
        &self.indexing_operations
    }
}

/// Container builder for infrastructure components
pub struct InfrastructureContainerBuilder {
    config: AppConfig,
}

impl InfrastructureContainerBuilder {
    /// Create a new container builder
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Build the infrastructure container
    pub async fn build(self) -> Result<InfrastructureComponents> {
        InfrastructureComponents::new(self.config).await
    }
}

/// Helper function to create infrastructure components
pub async fn create_infrastructure_components(
    config: AppConfig,
) -> Result<InfrastructureComponents> {
    InfrastructureContainerBuilder::new(config).build().await
}

/// Combined container with both infrastructure and domain services
#[derive(Clone)]
pub struct FullContainer {
    pub infrastructure: InfrastructureComponents,
    pub domain_services: super::modules::DomainServicesContainer,
}

impl FullContainer {
    /// Create a full container with both infrastructure and domain services
    pub async fn new(config: AppConfig) -> Result<Self> {
        let infrastructure = InfrastructureContainerBuilder::new(config.clone())
            .build()
            .await?;
        let domain_services = super::modules::DomainServicesFactory::create_services(
            infrastructure.cache.clone(),
            infrastructure.crypto.clone(),
            infrastructure.health.clone(),
            config,
            infrastructure.embedding_provider.clone(),
            infrastructure.vector_store_provider.clone(),
        )
        .await?;

        Ok(Self {
            infrastructure,
            domain_services,
        })
    }

    /// Get indexing service
    pub fn indexing_service(
        &self,
    ) -> Arc<dyn mcb_domain::domain_services::search::IndexingServiceInterface> {
        self.domain_services.indexing_service.clone()
    }

    /// Get context service
    pub fn context_service(
        &self,
    ) -> Arc<dyn mcb_domain::domain_services::search::ContextServiceInterface> {
        self.domain_services.context_service.clone()
    }

    /// Get search service
    pub fn search_service(
        &self,
    ) -> Arc<dyn mcb_domain::domain_services::search::SearchServiceInterface> {
        self.domain_services.search_service.clone()
    }
}
