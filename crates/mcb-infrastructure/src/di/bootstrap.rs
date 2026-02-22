//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#dependency-injection)
//!
//! DI Container Bootstrap - Provider Handles + Infrastructure Services
//!
//! Provides the composition root using runtime-swappable provider handles
//! and direct infrastructure service storage.

use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::{
    AgentRepository, CacheAdminInterface, CryptoProvider, EmbeddingAdminInterface,
    EventBusProvider, FileHashRepository, FileSystemProvider, HighlightServiceInterface,
    IndexingOperationsInterface, IssueEntityRepository, LanguageAdminInterface, LifecycleManaged,
    MemoryRepository, OrgEntityRepository, PerformanceMetricsInterface, PlanEntityRepository,
    ProjectDetectorService, ProjectRepository, ShutdownCoordinator, TaskRunnerProvider,
    VcsEntityRepository, VcsProvider, VectorStoreAdminInterface,
};

use crate::config::{AppConfig, ConfigLoader};
use crate::constants::providers::DEFAULT_DB_CONFIG_NAME;
use crate::constants::services::{
    CACHE_SERVICE_NAME, EMBEDDING_SERVICE_NAME, LANGUAGE_SERVICE_NAME, VECTOR_STORE_SERVICE_NAME,
};
use crate::crypto::CryptoService;
use crate::di::admin::{
    CacheAdminService, EmbeddingAdminService, LanguageAdminService, VectorStoreAdminService,
};
use crate::di::database_resolver::DatabaseProviderResolver;
use crate::di::handles::{
    CacheProviderHandle, EmbeddingProviderHandle, LanguageProviderHandle, VectorStoreProviderHandle,
};
use crate::di::provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, EventBusProviderResolver,
    FileSystemProviderResolver, LanguageProviderResolver, TaskRunnerProviderResolver,
    VcsProviderResolver, VectorStoreProviderResolver,
};
use crate::infrastructure::admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use crate::infrastructure::lifecycle::DefaultShutdownCoordinator;
use crate::project::ProjectService;
use crate::services::HighlightServiceImpl;

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
    file_system_provider: Arc<dyn FileSystemProvider>,
    task_runner_provider: Arc<dyn TaskRunnerProvider>,
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
    mcb_domain::arc_getters! {
        embedding_handle: EmbeddingProviderHandle,
        vector_store_handle: VectorStoreProviderHandle,
        cache_handle: CacheProviderHandle,
        language_handle: LanguageProviderHandle,
        embedding_resolver: EmbeddingProviderResolver,
        vector_store_resolver: VectorStoreProviderResolver,
        cache_resolver: CacheProviderResolver,
        language_resolver: LanguageProviderResolver,
        embedding_admin: dyn EmbeddingAdminInterface,
        vector_store_admin: dyn VectorStoreAdminInterface,
        cache_admin: dyn CacheAdminInterface,
        language_admin: dyn LanguageAdminInterface,
        event_bus: dyn EventBusProvider,
        shutdown: dyn ShutdownCoordinator => shutdown_coordinator,
        performance: dyn PerformanceMetricsInterface => performance_metrics,
        indexing: dyn IndexingOperationsInterface => indexing_operations,
        file_system_provider: dyn FileSystemProvider,
        task_runner_provider: dyn TaskRunnerProvider,
        memory_repository: dyn MemoryRepository,
        agent_repository: dyn AgentRepository,
        project_repository: dyn ProjectRepository,
        vcs_provider: dyn VcsProvider,
        project_service: dyn ProjectDetectorService,
        vcs_entity_repository: dyn VcsEntityRepository,
        plan_entity_repository: dyn PlanEntityRepository,
        issue_entity_repository: dyn IssueEntityRepository,
        org_entity_repository: dyn OrgEntityRepository,
        file_hash_repository: dyn FileHashRepository,
        highlight_service: dyn HighlightServiceInterface,
        crypto_service: dyn CryptoProvider,
    }

    /// Build domain services for the server layer
    /// This method creates all domain services needed by `McpServer`
    ///
    /// # Errors
    ///
    /// Returns an error if any domain service fails to initialize.
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
        let file_system_provider = self.file_system_provider();
        let task_runner_provider = self.task_runner_provider();

        let project_id = current_project_id()?;

        let memory_repository = self.memory_repository();
        let agent_repository = self.agent_repository();
        let file_hash_repository = self.file_hash_repository();
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
            file_system_provider,
            task_runner_provider,
            memory_repository,
            agent_repository,
            file_hash_repository,
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
///
/// # Errors
///
/// Returns an error if provider resolution, database connection, or service initialization fails.
pub async fn init_app(config: AppConfig) -> Result<AppContext> {
    mcb_domain::info!(
        "bootstrap",
        "Initializing application context with provider handles"
    );

    let config = Arc::new(config);

    // ========================================================================
    // Create Resolvers
    // ========================================================================

    let embedding_resolver = Arc::new(EmbeddingProviderResolver::new(Arc::clone(&config)));
    let vector_store_resolver = Arc::new(VectorStoreProviderResolver::new(Arc::clone(&config)));
    let cache_resolver = Arc::new(CacheProviderResolver::new(Arc::clone(&config)));
    let language_resolver = Arc::new(LanguageProviderResolver::new(Arc::clone(&config)));

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
        EMBEDDING_SERVICE_NAME,
        Arc::clone(&embedding_resolver),
        Arc::clone(&embedding_handle),
    ));
    let embedding_admin = Arc::clone(&embedding_admin_svc) as Arc<dyn EmbeddingAdminInterface>;

    let vector_store_admin_svc = Arc::new(VectorStoreAdminService::new(
        VECTOR_STORE_SERVICE_NAME,
        Arc::clone(&vector_store_resolver),
        Arc::clone(&vector_store_handle),
    ));
    let vector_store_admin =
        Arc::clone(&vector_store_admin_svc) as Arc<dyn VectorStoreAdminInterface>;

    let cache_admin_svc = Arc::new(CacheAdminService::new(
        CACHE_SERVICE_NAME,
        Arc::clone(&cache_resolver),
        Arc::clone(&cache_handle),
    ));
    let cache_admin = Arc::clone(&cache_admin_svc) as Arc<dyn CacheAdminInterface>;

    let language_admin_svc = Arc::new(LanguageAdminService::new(
        LANGUAGE_SERVICE_NAME,
        Arc::clone(&language_resolver),
        Arc::clone(&language_handle),
    ));
    let language_admin = Arc::clone(&language_admin_svc) as Arc<dyn LanguageAdminInterface>;

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

    let event_bus_resolver = EventBusProviderResolver::new(Arc::clone(&config));
    let event_bus: Arc<dyn EventBusProvider> = event_bus_resolver.resolve_from_config()?;
    let shutdown_coordinator: Arc<dyn ShutdownCoordinator> =
        Arc::new(DefaultShutdownCoordinator::new());
    let performance_metrics: Arc<dyn PerformanceMetricsInterface> =
        Arc::new(AtomicPerformanceMetrics::new());
    let indexing_operations: Arc<dyn IndexingOperationsInterface> =
        Arc::new(DefaultIndexingOperations::new());
    let fs_resolver = FileSystemProviderResolver::new(Arc::clone(&config));
    let file_system_provider: Arc<dyn FileSystemProvider> = fs_resolver.resolve_from_config()?;
    let task_runner_resolver = TaskRunnerProviderResolver::new(Arc::clone(&config));
    let task_runner_provider: Arc<dyn TaskRunnerProvider> =
        task_runner_resolver.resolve_from_config()?;

    mcb_domain::info!("bootstrap", "Created infrastructure services");

    // ========================================================================
    // Create Domain Services & Repositories
    // ========================================================================

    let db_config = config.providers.database.configs.get(DEFAULT_DB_CONFIG_NAME).ok_or_else(|| {
        mcb_domain::error::Error::config(
            "providers.database.configs.default is required; set path in config/default.toml under [providers.database.configs.default]",
        )
    })?;
    let memory_db_path = db_config.path.clone().ok_or_else(|| {
        mcb_domain::error::Error::config(
            "providers.database.configs.default.path is required; set the database file path in config/default.toml",
        )
    })?;

    let db_resolver = DatabaseProviderResolver::new(Arc::clone(&config));
    let db_provider = db_resolver.resolve_provider().map_err(|e| {
        mcb_domain::error::Error::internal(format!("Failed to resolve database provider: {e}"))
    })?;
    let db_executor = db_resolver
        .resolve_and_connect(memory_db_path.as_path())
        .await
        .map_err(|e| {
            mcb_domain::error::Error::internal(format!("Failed to create database executor: {e}"))
        })?;

    let memory_repository: Arc<dyn MemoryRepository> =
        db_provider.create_memory_repository(Arc::clone(&db_executor));
    let agent_repository = db_provider.create_agent_repository(Arc::clone(&db_executor));
    let project_repository = db_provider.create_project_repository(Arc::clone(&db_executor));
    let project_id = current_project_id()?;
    let file_hash_repository: Arc<dyn FileHashRepository> =
        db_provider.create_file_hash_repository(Arc::clone(&db_executor), project_id);

    let vcs_resolver = VcsProviderResolver::new(Arc::clone(&config));
    let vcs_provider = vcs_resolver.resolve_from_config()?;
    let project_service: Arc<dyn ProjectDetectorService> = Arc::new(ProjectService::new());

    let vcs_entity_repository: Arc<dyn VcsEntityRepository> =
        db_provider.create_vcs_entity_repository(Arc::clone(&db_executor));
    let plan_entity_repository: Arc<dyn PlanEntityRepository> =
        db_provider.create_plan_entity_repository(Arc::clone(&db_executor));
    let issue_entity_repository: Arc<dyn IssueEntityRepository> =
        db_provider.create_issue_entity_repository(Arc::clone(&db_executor));
    let org_entity_repository: Arc<dyn OrgEntityRepository> =
        db_provider.create_org_entity_repository(Arc::clone(&db_executor));

    let highlight_service: Arc<dyn HighlightServiceInterface> =
        Arc::new(HighlightServiceImpl::new());

    // ========================================================================
    // Create Crypto Service
    // ========================================================================

    let crypto_service = Arc::new(create_crypto_service(&config)?);

    mcb_domain::info!("bootstrap", "Created domain services and repositories");

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
        file_system_provider,
        task_runner_provider,
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
///
/// # Errors
///
/// Returns an error if application initialization fails.
pub async fn init_test_app() -> Result<AppContext> {
    let config = ConfigLoader::new().load()?;
    init_app(config).await
}

/// Create a test DI container with default configuration
///
/// # Errors
///
/// Returns an error if test application initialization fails.
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

fn current_project_id() -> Result<String> {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
        .ok_or_else(|| {
            mcb_domain::error::Error::config(
                "cannot determine project ID from current directory; ensure MCB is launched from a named directory",
            )
        })
}
