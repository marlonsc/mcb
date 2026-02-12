//! DI Container Bootstrap - Provider Handles + Infrastructure Services
//!
//! Provides the composition root using runtime-swappable provider handles
//! and direct infrastructure service storage.

use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::admin::{
    IndexingOperationsInterface, LifecycleManaged, PerformanceMetricsInterface, ShutdownCoordinator,
};
use mcb_domain::ports::browse::HighlightServiceInterface;
use mcb_domain::ports::infrastructure::EventBusProvider;
use mcb_domain::ports::providers::{CryptoProvider, VcsProvider};
use mcb_domain::ports::repositories::{
    AgentRepository, FileHashRepository, IssueEntityRepository, MemoryRepository,
    OrgEntityRepository, PlanEntityRepository, ProjectRepository, VcsEntityRepository,
};
use mcb_domain::ports::services::ProjectDetectorService;

use mcb_providers::database::{
    SqliteFileHashConfig, SqliteFileHashRepository, SqliteMemoryRepository,
    create_agent_repository_from_executor, create_project_repository_from_executor,
};
use tracing::info;

use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::di::admin::{
    CacheAdminInterface, CacheAdminService, EmbeddingAdminInterface, EmbeddingAdminService,
    LanguageAdminInterface, LanguageAdminService, VectorStoreAdminInterface,
    VectorStoreAdminService,
};
use crate::di::database_resolver::DatabaseProviderResolver;
use crate::di::handles::{
    CacheProviderHandle, EmbeddingProviderHandle, LanguageProviderHandle, VectorStoreProviderHandle,
};
use crate::di::provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};
use crate::infrastructure::admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use crate::infrastructure::lifecycle::DefaultShutdownCoordinator;
use crate::project::ProjectService;
use crate::services::HighlightServiceImpl;
use mcb_providers::events::TokioEventBusProvider;

/// Application context with provider handles and infrastructure services
pub struct AppContext {
    /// Application configuration
    pub config: Arc<AppConfig>,

    // ========================================================================
    // Provider Handles (runtime-swappable)
    // ========================================================================
    embedding_handle: Arc<EmbeddingProviderHandle>,
    vector_store_handle: Arc<VectorStoreProviderHandle>,
    cache_handle: Arc<CacheProviderHandle>,
    language_handle: Arc<LanguageProviderHandle>,

    // ========================================================================
    // Provider Resolvers (linkme registry access)
    // ========================================================================
    embedding_resolver: Arc<EmbeddingProviderResolver>,
    vector_store_resolver: Arc<VectorStoreProviderResolver>,
    cache_resolver: Arc<CacheProviderResolver>,
    language_resolver: Arc<LanguageProviderResolver>,

    // ========================================================================
    // Admin Services (switch providers via API)
    // ========================================================================
    embedding_admin: Arc<dyn EmbeddingAdminInterface>,
    vector_store_admin: Arc<dyn VectorStoreAdminInterface>,
    cache_admin: Arc<dyn CacheAdminInterface>,
    language_admin: Arc<dyn LanguageAdminInterface>,

    // ========================================================================
    // Infrastructure Services (direct storage)
    // ========================================================================
    event_bus: Arc<dyn EventBusProvider>,
    shutdown_coordinator: Arc<dyn ShutdownCoordinator>,
    performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    indexing_operations: Arc<dyn IndexingOperationsInterface>,
    /// Services eligible for lifecycle management
    pub lifecycle_services: Vec<Arc<dyn LifecycleManaged>>,

    // ========================================================================
    // Domain Services & Repositories (auto-registered)
    // ========================================================================
    memory_repository: Arc<dyn MemoryRepository>,
    agent_repository: Arc<dyn AgentRepository>,
    project_repository: Arc<dyn ProjectRepository>,
    vcs_provider: Arc<dyn VcsProvider>,
    project_service: Arc<dyn ProjectDetectorService>,
    vcs_entity_repository: Arc<dyn VcsEntityRepository>,
    plan_entity_repository: Arc<dyn PlanEntityRepository>,
    issue_entity_repository: Arc<dyn IssueEntityRepository>,
    org_entity_repository: Arc<dyn OrgEntityRepository>,
    file_hash_repository: Arc<dyn FileHashRepository>,

    // ========================================================================
    // Infrastructure Services
    // ========================================================================
    highlight_service: Arc<dyn HighlightServiceInterface>,
    crypto_service: Arc<dyn CryptoProvider>,
}

impl AppContext {
    /// Get embedding provider handle
    pub fn embedding_handle(&self) -> Arc<EmbeddingProviderHandle> {
        self.embedding_handle.clone()
    }

    /// Get vector store provider handle
    pub fn vector_store_handle(&self) -> Arc<VectorStoreProviderHandle> {
        self.vector_store_handle.clone()
    }

    /// Get cache provider handle
    pub fn cache_handle(&self) -> Arc<CacheProviderHandle> {
        self.cache_handle.clone()
    }

    /// Get language provider handle
    pub fn language_handle(&self) -> Arc<LanguageProviderHandle> {
        self.language_handle.clone()
    }

    /// Get embedding provider resolver
    pub fn embedding_resolver(&self) -> Arc<EmbeddingProviderResolver> {
        self.embedding_resolver.clone()
    }

    /// Get vector store provider resolver
    pub fn vector_store_resolver(&self) -> Arc<VectorStoreProviderResolver> {
        self.vector_store_resolver.clone()
    }

    /// Get cache provider resolver
    pub fn cache_resolver(&self) -> Arc<CacheProviderResolver> {
        self.cache_resolver.clone()
    }

    /// Get language provider resolver
    pub fn language_resolver(&self) -> Arc<LanguageProviderResolver> {
        self.language_resolver.clone()
    }

    /// Get embedding admin service
    pub fn embedding_admin(&self) -> Arc<dyn EmbeddingAdminInterface> {
        self.embedding_admin.clone()
    }

    /// Get vector store admin service
    pub fn vector_store_admin(&self) -> Arc<dyn VectorStoreAdminInterface> {
        self.vector_store_admin.clone()
    }

    /// Get cache admin service
    pub fn cache_admin(&self) -> Arc<dyn CacheAdminInterface> {
        self.cache_admin.clone()
    }

    /// Get language admin service
    pub fn language_admin(&self) -> Arc<dyn LanguageAdminInterface> {
        self.language_admin.clone()
    }

    /// Get event bus
    pub fn event_bus(&self) -> Arc<dyn EventBusProvider> {
        self.event_bus.clone()
    }

    /// Get shutdown coordinator
    pub fn shutdown(&self) -> Arc<dyn ShutdownCoordinator> {
        self.shutdown_coordinator.clone()
    }

    /// Get performance metrics
    pub fn performance(&self) -> Arc<dyn PerformanceMetricsInterface> {
        self.performance_metrics.clone()
    }

    /// Get indexing operations
    pub fn indexing(&self) -> Arc<dyn IndexingOperationsInterface> {
        self.indexing_operations.clone()
    }

    /// Get memory repository
    pub fn memory_repository(&self) -> Arc<dyn MemoryRepository> {
        self.memory_repository.clone()
    }

    /// Get agent repository
    pub fn agent_repository(&self) -> Arc<dyn AgentRepository> {
        self.agent_repository.clone()
    }

    /// Get project repository
    pub fn project_repository(&self) -> Arc<dyn ProjectRepository> {
        self.project_repository.clone()
    }

    /// Get VCS provider
    pub fn vcs_provider(&self) -> Arc<dyn VcsProvider> {
        self.vcs_provider.clone()
    }

    /// Get project service
    pub fn project_service(&self) -> Arc<dyn ProjectDetectorService> {
        self.project_service.clone()
    }

    /// Get VCS entity repository
    pub fn vcs_entity_repository(&self) -> Arc<dyn VcsEntityRepository> {
        self.vcs_entity_repository.clone()
    }

    /// Get plan entity repository
    pub fn plan_entity_repository(&self) -> Arc<dyn PlanEntityRepository> {
        self.plan_entity_repository.clone()
    }

    /// Get issue entity repository
    pub fn issue_entity_repository(&self) -> Arc<dyn IssueEntityRepository> {
        self.issue_entity_repository.clone()
    }

    /// Get org entity repository
    pub fn org_entity_repository(&self) -> Arc<dyn OrgEntityRepository> {
        self.org_entity_repository.clone()
    }

    /// Get file hash repository
    pub fn file_hash_repository(&self) -> Arc<dyn FileHashRepository> {
        self.file_hash_repository.clone()
    }

    /// Get highlight service
    pub fn highlight_service(&self) -> Arc<dyn HighlightServiceInterface> {
        self.highlight_service.clone()
    }

    /// Get crypto service
    pub fn crypto_service(&self) -> Arc<dyn CryptoProvider> {
        self.crypto_service.clone()
    }

    /// Build domain services for the server layer
    /// This method creates all domain services needed by McpServer
    pub async fn build_domain_services(
        &self,
    ) -> Result<crate::di::modules::domain_services::DomainServicesContainer> {
        let embedding_provider = self.embedding_handle().get();
        let vector_store_provider = self.vector_store_handle().get();
        let cache_provider = self.cache_handle().get();
        let language_chunker = self.language_handle().get();

        let shared_cache = crate::cache::provider::SharedCacheProvider::from_arc(cache_provider);
        let crypto = self.crypto_service();

        let indexing_ops = self.indexing();
        let event_bus = self.event_bus();

        let project_id = std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "default".to_string());

        let memory_repository = self.memory_repository();
        let agent_repository = self.agent_repository();
        let vcs_provider = self.vcs_provider();
        let project_service = self.project_service();

        let deps = crate::di::modules::domain_services::ServiceDependencies {
            project_id,
            cache: shared_cache,
            crypto,
            config: (*self.config).clone(),
            embedding_provider,
            vector_store_provider,
            language_chunker,
            indexing_ops,
            event_bus,
            memory_repository,
            agent_repository,
            vcs_provider,
            project_service,
            project_repository: self.project_repository(),
            vcs_entity_repository: self.vcs_entity_repository(),
            plan_entity_repository: self.plan_entity_repository(),
            issue_entity_repository: self.issue_entity_repository(),
            org_entity_repository: self.org_entity_repository(),
        };

        crate::di::modules::domain_services::DomainServicesFactory::create_services(deps).await
    }
}

impl std::fmt::Debug for AppContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppContext")
            .field("embedding", &self.embedding_handle)
            .field("vector_store", &self.vector_store_handle)
            .field("cache", &self.cache_handle)
            .field("language", &self.language_handle)
            .finish_non_exhaustive()
    }
}

/// Initialize application context with provider handles and infrastructure services
pub async fn init_app(config: AppConfig) -> Result<AppContext> {
    info!("Initializing application context with provider handles");

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

    let embedding_provider = embedding_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("Embedding: {e}")))?;

    let vector_store_provider = vector_store_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("VectorStore: {e}")))?;

    let cache_provider = cache_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("Cache: {e}")))?;

    let language_provider = language_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("Language: {e}")))?;

    // ========================================================================
    // Create Handles
    // ========================================================================

    let embedding_handle = Arc::new(EmbeddingProviderHandle::new(embedding_provider));
    let vector_store_handle = Arc::new(VectorStoreProviderHandle::new(vector_store_provider));
    let cache_handle = Arc::new(CacheProviderHandle::new(cache_provider));
    let language_handle = Arc::new(LanguageProviderHandle::new(language_provider));

    // ========================================================================
    // Create Admin Services
    // ========================================================================

    let embedding_admin_svc = Arc::new(EmbeddingAdminService::new(
        "Embedding Service",
        embedding_resolver.clone(),
        embedding_handle.clone(),
    ));
    let embedding_admin: Arc<dyn EmbeddingAdminInterface> = embedding_admin_svc.clone();

    let vector_store_admin_svc = Arc::new(VectorStoreAdminService::new(
        "Vector Store Service",
        vector_store_resolver.clone(),
        vector_store_handle.clone(),
    ));
    let vector_store_admin: Arc<dyn VectorStoreAdminInterface> = vector_store_admin_svc.clone();

    let cache_admin_svc = Arc::new(CacheAdminService::new(
        "Cache Service",
        cache_resolver.clone(),
        cache_handle.clone(),
    ));
    let cache_admin: Arc<dyn CacheAdminInterface> = cache_admin_svc.clone();

    let language_admin_svc = Arc::new(LanguageAdminService::new(
        "Language Service",
        language_resolver.clone(),
        language_handle.clone(),
    ));
    let language_admin: Arc<dyn LanguageAdminInterface> = language_admin_svc.clone();

    // Collect lifecycle managed services
    let lifecycle_services: Vec<Arc<dyn LifecycleManaged>> = vec![
        embedding_admin_svc,
        vector_store_admin_svc,
        cache_admin_svc,
        language_admin_svc,
    ];

    // ========================================================================
    // Create Infrastructure Services
    // ========================================================================

    let event_bus: Arc<dyn EventBusProvider> = Arc::new(TokioEventBusProvider::new());
    let shutdown_coordinator: Arc<dyn ShutdownCoordinator> =
        Arc::new(DefaultShutdownCoordinator::new());
    let performance_metrics: Arc<dyn PerformanceMetricsInterface> =
        Arc::new(AtomicPerformanceMetrics::new());
    let indexing_operations: Arc<dyn IndexingOperationsInterface> =
        Arc::new(DefaultIndexingOperations::new());

    info!("Created infrastructure services");

    // ========================================================================
    // Create Domain Services & Repositories
    // ========================================================================

    // Use configured path or fallback to default
    let memory_db_path = config.auth.user_db_path.clone().unwrap_or_else(|| {
        dirs::data_local_dir()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".mcb")
            .join("memory.db")
    });

    let db_resolver = DatabaseProviderResolver::new(config.clone());
    let db_executor = db_resolver
        .resolve_and_connect(memory_db_path.as_path())
        .await
        .map_err(|e| {
            mcb_domain::error::Error::internal(format!("Failed to create database executor: {e}"))
        })?;

    let memory_repository: Arc<dyn MemoryRepository> =
        Arc::new(SqliteMemoryRepository::new(Arc::clone(&db_executor)));
    let agent_repository = create_agent_repository_from_executor(Arc::clone(&db_executor));
    let project_repository = create_project_repository_from_executor(Arc::clone(&db_executor));
    let file_hash_repository: Arc<dyn FileHashRepository> = Arc::new(
        SqliteFileHashRepository::new(Arc::clone(&db_executor), SqliteFileHashConfig::default()),
    );

    let vcs_provider = crate::di::vcs::default_vcs_provider();
    let project_service: Arc<dyn ProjectDetectorService> = Arc::new(ProjectService::new());

    let vcs_entity_repository: Arc<dyn VcsEntityRepository> = Arc::new(
        mcb_providers::database::SqliteVcsEntityRepository::new(Arc::clone(&db_executor)),
    );
    let plan_entity_repository: Arc<dyn PlanEntityRepository> = Arc::new(
        mcb_providers::database::SqlitePlanEntityRepository::new(Arc::clone(&db_executor)),
    );
    let issue_entity_repository: Arc<dyn IssueEntityRepository> = Arc::new(
        mcb_providers::database::SqliteIssueEntityRepository::new(Arc::clone(&db_executor)),
    );
    let org_entity_repository: Arc<dyn OrgEntityRepository> = Arc::new(
        mcb_providers::database::SqliteOrgEntityRepository::new(Arc::clone(&db_executor)),
    );

    let highlight_service: Arc<dyn HighlightServiceInterface> =
        Arc::new(HighlightServiceImpl::new());

    // ========================================================================
    // Create Crypto Service
    // ========================================================================

    let crypto_service = Arc::new(create_crypto_service(&config)?);

    info!("Created domain services and repositories");

    Ok(AppContext {
        config,
        embedding_handle,
        vector_store_handle,
        cache_handle,
        language_handle,
        embedding_resolver,
        vector_store_resolver,
        cache_resolver,
        language_resolver,
        embedding_admin,
        vector_store_admin,
        cache_admin,
        language_admin,
        event_bus,
        shutdown_coordinator,
        performance_metrics,
        indexing_operations,
        memory_repository,
        agent_repository,
        project_repository,
        vcs_provider,
        project_service,
        vcs_entity_repository,
        plan_entity_repository,
        issue_entity_repository,
        org_entity_repository,
        file_hash_repository,
        highlight_service,
        crypto_service,
        lifecycle_services,
    })
}

/// Initialize application for testing
pub async fn init_test_app() -> Result<AppContext> {
    let config = AppConfig::default();
    init_app(config).await
}

pub type DiContainer = AppContext;

/// Create a test DI container with default configuration
pub async fn create_test_container() -> Result<AppContext> {
    init_test_app().await
}

/// Create crypto service from configuration
fn create_crypto_service(config: &AppConfig) -> Result<CryptoService> {
    let master_key = if config.auth.jwt.secret.len() >= 32 {
        config.auth.jwt.secret.as_bytes()[..32].to_vec()
    } else {
        CryptoService::generate_master_key()
    };

    CryptoService::new(master_key)
}
