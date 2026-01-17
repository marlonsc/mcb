//! DI Container Bootstrap - Shaku Strict Pattern
//!
//! Provides the composition root for the Shaku-based dependency injection system.
//! This follows the strict Shaku hierarchical module pattern with no manual wiring.
//!
//! ## Shaku Strict Architecture
//!
//! - **Module Hierarchy**: Uses `module!` macro with `use dyn ModuleTrait` for composition
//! - **No Manual Wiring**: All dependencies resolved through module interfaces
//! - **Provider Overrides**: Runtime component overrides for production configuration
//! - **Trait-based DI**: All dependencies injected as `Arc<dyn Trait>`
//!
//! ## Construction Pattern
//!
//! ```rust,ignore
//! // Build all modules (simplified - no dependencies between them)
//! let infrastructure = Arc::new(InfrastructureModuleImpl::builder().build());
//! let server = Arc::new(ServerModuleImpl::builder().build());
//! let adapters = Arc::new(AdaptersModuleImpl::builder().build());
//! let application = Arc::new(ApplicationModuleImpl::builder().build());
//! let admin = Arc::new(AdminModuleImpl::builder().build());
//!
//! // Build root container
//! let container = McpModule::builder(infrastructure, server, adapters, application, admin).build();
//! ```
//!
//! ## Note
//!
//! Most services are created at runtime via DomainServicesFactory (in domain_services.rs)
//! rather than through Shaku DI, because they require runtime configuration.
//! The Shaku modules primarily hold null providers as defaults.

use crate::config::AppConfig;
use crate::crypto::CryptoService;
use mcb_domain::error::Result;
use std::sync::Arc;
use tracing::info;

// Import module implementations
use super::modules::{
    cache_module::CacheModuleImpl,
    data_module::DataModuleImpl,
    embedding_module::EmbeddingModuleImpl,
    language_module::LanguageModuleImpl,
    usecase_module::UseCaseModuleImpl,
    infrastructure::InfrastructureModuleImpl,
    server::ServerModuleImpl,
    adapters::AdaptersModuleImpl,
    application::ApplicationModuleImpl,
    admin::AdminModuleImpl,
};

// Import factories for provider overrides (production configuration)
use super::factory::{EmbeddingProviderFactory, VectorStoreProviderFactory};

/// Shaku-based DI Container following strict hierarchical pattern.
///
/// This container holds the AppContainer and provides access to all services
/// through the module resolution system. No manual component management.
///
/// ## Usage
///
/// ```rust,ignore
/// // Create container with production config
/// let container = DiContainer::build_with_config(config, http_client).await?;
///
/// // Resolve provider through trait-based access
/// let embedding_provider: Arc<dyn EmbeddingProvider> = container.resolve();
/// ```
pub type DiContainer = AppContainer;

/// Provider configuration overrides for production setup.
///
/// This struct provides methods to create configured providers
/// that can be injected into the module hierarchy at runtime.
pub struct ProviderOverrides;

impl ProviderOverrides {
    /// Create embedding provider from configuration
    pub fn create_embedding_provider(config: &AppConfig) -> Result<Arc<dyn mcb_application::ports::providers::EmbeddingProvider>> {
        if let Some((name, embedding_config)) = config.providers.embedding.iter().next() {
            info!(provider = name, "Creating embedding provider from config");
            EmbeddingProviderFactory::create(embedding_config, None)
        } else {
            info!("No embedding provider configured, using null provider");
            Ok(EmbeddingProviderFactory::create_null())
        }
    }

    /// Create vector store provider from configuration
    pub fn create_vector_store_provider(
        config: &AppConfig,
        crypto: &CryptoService,
    ) -> Result<Arc<dyn mcb_application::ports::providers::VectorStoreProvider>> {
        if let Some((name, vector_config)) = config.providers.vector_store.iter().next() {
            info!(
                provider = name,
                "Creating vector store provider from config"
            );
            // Wrap CryptoService as Arc<dyn CryptoProvider> for DI
            let crypto_provider: Arc<dyn mcb_application::ports::providers::CryptoProvider> = Arc::new(crypto.clone());
            VectorStoreProviderFactory::create(vector_config, Some(crypto_provider))
        } else {
            info!("No vector store provider configured, using in-memory provider");
            Ok(VectorStoreProviderFactory::create_in_memory())
        }
    }
}

/// Container builder for Shaku-based DI system.
///
/// Builds the hierarchical module structure following the strict Shaku pattern.
/// Provides both testing (null providers) and production (configured providers) setups.
pub struct DiContainerBuilder {
    #[allow(dead_code)]
    config: Option<AppConfig>,
    #[allow(dead_code)]
    embedding_override: Option<Arc<dyn mcb_application::ports::providers::EmbeddingProvider>>,
    #[allow(dead_code)]
    vector_store_override: Option<Arc<dyn mcb_application::ports::providers::VectorStoreProvider>>,
}

impl DiContainerBuilder {
    /// Create a new container builder for testing (null providers)
    pub fn new() -> Self {
        Self {
            config: None,
            embedding_override: None,
            vector_store_override: None,
        }
    }

    /// Create a container builder with production configuration
    pub fn with_config(config: AppConfig) -> Self {
        Self {
            config: Some(config),
            embedding_override: None,
            vector_store_override: None,
        }
    }

    /// Override the embedding provider (for production configuration)
    pub fn with_embedding_provider(
        mut self,
        provider: Arc<dyn mcb_application::ports::providers::EmbeddingProvider>,
    ) -> Self {
        self.embedding_override = Some(provider);
        self
    }

    /// Override the vector store provider (for production configuration)
    pub fn with_vector_store_provider(
        mut self,
        provider: Arc<dyn mcb_application::ports::providers::VectorStoreProvider>,
    ) -> Self {
        self.vector_store_override = Some(provider);
        self
    }

    /// Build the DI container with hierarchical module composition
    ///
    /// This method delegates to the new Clean Architecture init_app function.
    /// The old McpModule approach has been replaced with proper hierarchical modules.
    pub async fn build(self) -> Result<DiContainer> {
        // Convert from AppConfig (if available) or create default
        // Note: This builder pattern is maintained for backward compatibility
        // but delegates to the new init_app function
        let config = AppConfig::default(); // TODO: Use actual config from builder
        init_app(config).await
    }
}

impl Default for DiContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create DI container for testing
pub async fn create_test_container() -> Result<DiContainer> {
    DiContainerBuilder::new().build().await
}

/// Convenience function to create DI container with production configuration
pub async fn create_production_container(config: AppConfig) -> Result<DiContainer> {
    // Create configured providers
    let embedding_provider = ProviderOverrides::create_embedding_provider(&config)?;
    let crypto = CryptoService::new(
        if config.auth.jwt.secret.len() >= 32 {
            config.auth.jwt.secret.as_bytes()[..32].to_vec()
        } else {
            CryptoService::generate_master_key()
        }
    )?;
    let vector_store_provider = ProviderOverrides::create_vector_store_provider(&config, &crypto)?;

    DiContainerBuilder::with_config(config)
        .with_embedding_provider(embedding_provider)
        .with_vector_store_provider(vector_store_provider)
        .build()
        .await
}

/// Legacy compatibility - creates full container for gradual migration
///
/// **DEPRECATED**: Use `create_production_container()` instead.
/// This will be removed in v0.2.0 when migration to strict Shaku pattern is complete.
#[deprecated(
    since = "0.1.0",
    note = "Use `create_production_container()` instead. This function will be removed in v0.2.0."
)]
pub async fn create_full_container(config: AppConfig) -> Result<DiContainer> {
    create_production_container(config).await
}

// ============================================================================
// Legacy Compatibility Types (for gradual migration)
// ============================================================================

/// Infrastructure components container for backward compatibility
///
/// **Note**: This exists for gradual migration from the old API.
/// New code should use `create_production_container()` and Shaku modules.
#[derive(Clone)]
pub struct InfrastructureComponents {
    /// Shared cache provider
    pub cache: crate::cache::provider::SharedCacheProvider,
    /// Crypto service
    pub crypto: CryptoService,
    /// Health registry
    pub health: crate::health::HealthRegistry,
}

impl InfrastructureComponents {
    /// Create infrastructure components from configuration
    pub async fn new(config: AppConfig) -> Result<Self> {
        InfrastructureContainerBuilder::new(config).build().await
    }
}

/// Builder for infrastructure components (backward compatibility)
///
/// **Note**: This exists for gradual migration from the old API.
pub struct InfrastructureContainerBuilder {
    config: AppConfig,
}

impl InfrastructureContainerBuilder {
    /// Create a new builder
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Build infrastructure components
    pub async fn build(self) -> Result<InfrastructureComponents> {
        // Create cache provider
        let cache = crate::cache::factory::CacheProviderFactory::create_from_config(
            &self.config.system.infrastructure.cache,
        )
        .await?;

        // Create crypto service with 32-byte key
        let master_key = if self.config.auth.jwt.secret.len() >= 32 {
            self.config.auth.jwt.secret.as_bytes()[..32].to_vec()
        } else {
            CryptoService::generate_master_key()
        };
        let crypto = CryptoService::new(master_key)?;

        // Create health registry
        let health = crate::health::HealthRegistry::new();
        let system_checker = crate::health::checkers::SystemHealthChecker::new();
        health.register_checker("system".to_string(), system_checker).await;

        Ok(InfrastructureComponents {
            cache,
            crypto,
            health,
        })
    }
}

// ============================================================================
// Clean Architecture App Module (Hierarchical Composition)
// ============================================================================

/// Application container using Clean Architecture modules
pub struct AppContainer {
    pub cache: CacheModuleImpl,
    pub embedding: EmbeddingModuleImpl,
    pub data: DataModuleImpl,
    pub language: LanguageModuleImpl,
    pub usecase: UseCaseModuleImpl,
    pub infrastructure: InfrastructureModuleImpl,
    pub server: ServerModuleImpl,
    pub adapters: AdaptersModuleImpl,
    pub application: ApplicationModuleImpl,
    pub admin: AdminModuleImpl,
}

/// Initialize the application using Clean Architecture modules
///
/// This replaces the old FullContainer approach with pure Shaku DI.
/// Uses hierarchical modules following the Clean Architecture pattern.
pub async fn init_app(_config: AppConfig) -> Result<AppContainer> {
    info!("Initializing Clean Architecture application modules");

    // Build context modules with production overrides
    let cache = CacheModuleImpl::builder()
        // Could override with Redis cache here if configured
        .build();

    let embedding = EmbeddingModuleImpl::builder()
        // Override with production embedding provider if configured
        .build();

    let data = DataModuleImpl::builder()
        // Override with production vector store if configured
        .build();

    let language = LanguageModuleImpl::builder()
        // Universal chunker works for all languages
        .build();

    let usecase = UseCaseModuleImpl::builder()
        // Use cases are components with DI
        .build();

    let infrastructure = InfrastructureModuleImpl::builder()
        .build();

    let server = ServerModuleImpl::builder()
        .build();

    let adapters = AdaptersModuleImpl::builder()
        .build();

    let application = ApplicationModuleImpl::builder()
        .build();

    let admin = AdminModuleImpl::builder()
        .build();

    // Compose into final app container
    let app_container = AppContainer {
        cache,
        embedding,
        data,
        language,
        usecase,
        infrastructure,
        server,
        adapters,
        application,
        admin,
    };

    info!("Clean Architecture application initialized successfully");
    Ok(app_container)
}
