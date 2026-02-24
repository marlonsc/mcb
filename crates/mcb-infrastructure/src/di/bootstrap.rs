//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#dependency-injection)
//!
//! DI Container Bootstrap — Direct Providers + Infrastructure Services
//!
//! Provides the composition root using directly-resolved providers.
//! No runtime switching (handles/admin services removed — Loco migration Wave 3).
//!
//! Production code uses `loco_app.rs::create_mcp_server()` directly.
//! This module exists for test infrastructure compatibility.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mcb_domain::error::Result;
use mcb_domain::ports::{
    AgentRepository, CacheEntryConfig, CacheProvider, CacheStats, CryptoProvider,
    EmbeddingProvider, EventBusProvider, FileHashRepository, IndexingOperationsInterface,
    IssueEntityRepository, LanguageChunkingProvider, MemoryRepository, OrgEntityRepository,
    PlanEntityRepository, ProjectDetectorService, ProjectRepository, VcsEntityRepository,
    VcsProvider, VectorStoreProvider,
};

use crate::config::{AppConfig, ConfigLoader};
use crate::constants::providers::DEFAULT_DB_CONFIG_NAME;
use crate::crypto::CryptoService;
use crate::di::provider_resolvers::{
    EmbeddingProviderResolver, LanguageProviderResolver, VectorStoreProviderResolver,
};
use crate::infrastructure::admin::DefaultIndexingOperations;
use crate::project::ProjectService;
use mcb_providers::database::seaorm::repos::{
    SeaOrmAgentRepository, SeaOrmEntityRepository, SeaOrmIndexRepository,
    SeaOrmObservationRepository, SeaOrmProjectRepository,
};
use mcb_providers::events::TokioEventBusProvider;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

/// Application context with resolved providers and infrastructure services.
///
/// Used by test infrastructure. Production code uses
/// `loco_app.rs::create_mcp_server()` which wires services directly.
pub struct AppContext {
    /// Application configuration
    pub config: Arc<AppConfig>,

    // Providers (resolved once, immutable)
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
    cache_provider: Arc<dyn CacheProvider>,
    language_chunker: Arc<dyn LanguageChunkingProvider>,

    // Infrastructure services
    event_bus: Arc<dyn EventBusProvider>,
    indexing_ops: Arc<dyn IndexingOperationsInterface>,

    // Repositories
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

    // Services
    crypto_service: Arc<dyn CryptoProvider>,
}

impl AppContext {
    // ── Provider accessors ──────────────────────────────────────────────

    /// Get embedding provider
    #[must_use]
    pub fn embedding_provider(&self) -> Arc<dyn EmbeddingProvider> {
        Arc::clone(&self.embedding_provider)
    }

    /// Get vector store provider
    #[must_use]
    pub fn vector_store_provider(&self) -> Arc<dyn VectorStoreProvider> {
        Arc::clone(&self.vector_store_provider)
    }

    /// Get cache provider
    #[must_use]
    pub fn cache_provider(&self) -> Arc<dyn CacheProvider> {
        Arc::clone(&self.cache_provider)
    }

    /// Get language chunking provider
    #[must_use]
    pub fn language_chunker(&self) -> Arc<dyn LanguageChunkingProvider> {
        Arc::clone(&self.language_chunker)
    }

    // ── Infrastructure accessors ────────────────────────────────────────

    /// Get event bus
    #[must_use]
    pub fn event_bus(&self) -> Arc<dyn EventBusProvider> {
        Arc::clone(&self.event_bus)
    }

    /// Get indexing operations
    #[must_use]
    pub fn indexing(&self) -> Arc<dyn IndexingOperationsInterface> {
        Arc::clone(&self.indexing_ops)
    }

    // ── Repository accessors ────────────────────────────────────────────

    /// Get memory repository
    #[must_use]
    pub fn memory_repository(&self) -> Arc<dyn MemoryRepository> {
        Arc::clone(&self.memory_repository)
    }

    /// Get agent repository
    #[must_use]
    pub fn agent_repository(&self) -> Arc<dyn AgentRepository> {
        Arc::clone(&self.agent_repository)
    }

    /// Get project repository
    #[must_use]
    pub fn project_repository(&self) -> Arc<dyn ProjectRepository> {
        Arc::clone(&self.project_repository)
    }

    /// Get VCS provider
    #[must_use]
    pub fn vcs_provider(&self) -> Arc<dyn VcsProvider> {
        Arc::clone(&self.vcs_provider)
    }

    /// Get project service
    #[must_use]
    pub fn project_service(&self) -> Arc<dyn ProjectDetectorService> {
        Arc::clone(&self.project_service)
    }

    /// Get VCS entity repository
    #[must_use]
    pub fn vcs_entity_repository(&self) -> Arc<dyn VcsEntityRepository> {
        Arc::clone(&self.vcs_entity_repository)
    }

    /// Get plan entity repository
    #[must_use]
    pub fn plan_entity_repository(&self) -> Arc<dyn PlanEntityRepository> {
        Arc::clone(&self.plan_entity_repository)
    }

    /// Get issue entity repository
    #[must_use]
    pub fn issue_entity_repository(&self) -> Arc<dyn IssueEntityRepository> {
        Arc::clone(&self.issue_entity_repository)
    }

    /// Get org entity repository
    #[must_use]
    pub fn org_entity_repository(&self) -> Arc<dyn OrgEntityRepository> {
        Arc::clone(&self.org_entity_repository)
    }

    /// Get file hash repository
    #[must_use]
    pub fn file_hash_repository(&self) -> Arc<dyn FileHashRepository> {
        Arc::clone(&self.file_hash_repository)
    }

    // ── Service accessors ───────────────────────────────────────────────

    /// Get crypto service
    #[must_use]
    pub fn crypto_service(&self) -> Arc<dyn CryptoProvider> {
        Arc::clone(&self.crypto_service)
    }

    /// Build domain services for the server layer.
    ///
    /// # Errors
    ///
    /// Returns an error if any domain service fails to initialize.
    pub async fn build_domain_services(
        &self,
    ) -> Result<crate::di::modules::domain_services::DomainServicesContainer> {
        let shared_cache =
            crate::cache::provider::SharedCacheProvider::from_arc(Arc::clone(&self.cache_provider));

        let project_id = current_project_id()?;

        let deps = crate::di::modules::domain_services::ServiceDependencies {
            project_id,
            cache: shared_cache,
            crypto: self.crypto_service(),
            config: (*self.config).clone(),
            embedding_provider: self.embedding_provider(),
            vector_store_provider: self.vector_store_provider(),
            language_chunker: self.language_chunker(),
            indexing_ops: self.indexing(),
            event_bus: self.event_bus(),
            memory_repository: self.memory_repository(),
            agent_repository: self.agent_repository(),
            file_hash_repository: self.file_hash_repository(),
            vcs_provider: self.vcs_provider(),
            project_service: self.project_service(),
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
            .field("embedding", &self.embedding_provider.provider_name())
            .field("vector_store", &self.vector_store_provider.provider_name())
            .field("cache", &self.cache_provider.provider_name())
            .field("language", &self.language_chunker.provider_name())
            .finish_non_exhaustive()
    }
}

// =========================================================================
// Initialization
// =========================================================================

/// Initialize application context with directly resolved providers.
///
/// For production, use `loco_app.rs::create_mcp_server()` instead.
///
/// # Errors
///
/// Returns an error if provider resolution, database connection, or service initialization fails.
pub async fn init_app(config: AppConfig) -> Result<AppContext> {
    init_app_with_overrides(config, None).await
}

/// Initialize with an optional embedding provider override.
///
/// Pass `embedding_override` to substitute the embedding provider
/// (used for ONNX fallback in tests when `FastEmbed` is unavailable).
///
/// # Errors
///
/// Returns an error if provider resolution or initialization fails.
pub async fn init_app_with_overrides(
    config: AppConfig,
    embedding_override: Option<Arc<dyn EmbeddingProvider>>,
) -> Result<AppContext> {
    let config = Arc::new(config);

    // ── Resolve providers ───────────────────────────────────────────────

    let embedding_provider = if let Some(provider) = embedding_override {
        provider
    } else {
        EmbeddingProviderResolver::new(Arc::clone(&config))
            .resolve_from_config()
            .map_err(|e| mcb_domain::error::Error::configuration(format!("Embedding: {e}")))?
    };

    let vector_store_provider = VectorStoreProviderResolver::new(Arc::clone(&config))
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("VectorStore: {e}")))?;

    let cache_provider: Arc<dyn CacheProvider> = Arc::new(TestCache::new());

    let language_chunker = LanguageProviderResolver::new(Arc::clone(&config))
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("Language: {e}")))?;

    // ── Infrastructure services ─────────────────────────────────────────

    let event_bus: Arc<dyn EventBusProvider> = Arc::new(TokioEventBusProvider::new());
    let indexing_ops: Arc<dyn IndexingOperationsInterface> =
        Arc::new(DefaultIndexingOperations::new());

    // ── Database ────────────────────────────────────────────────────────

    let db_config = config
        .providers
        .database
        .configs
        .get(DEFAULT_DB_CONFIG_NAME)
        .ok_or_else(|| {
            mcb_domain::error::Error::config("providers.database.configs.default is required")
        })?;
    let memory_db_path = db_config.path.clone().ok_or_else(|| {
        mcb_domain::error::Error::config("providers.database.configs.default.path is required")
    })?;

    if let Some(parent) = memory_db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            mcb_domain::error::Error::internal(format!(
                "Failed to create database directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    let db_url = format!("sqlite://{}?mode=rwc", memory_db_path.display());
    let mut connect_opts = ConnectOptions::new(db_url);
    connect_opts
        .max_connections(5)
        .min_connections(1)
        .sqlx_logging(false);

    let db: DatabaseConnection = Database::connect(connect_opts).await.map_err(|e| {
        mcb_domain::error::Error::internal(format!("Failed to connect to database: {e}"))
    })?;

    use sea_orm_migration::MigratorTrait;
    mcb_providers::database::seaorm::migration::Migrator::up(&db, None)
        .await
        .map_err(|e| {
            mcb_domain::error::Error::internal(format!("Failed to run migrations: {e}"))
        })?;

    let db = Arc::new(db);
    let project_id = current_project_id()?;

    // ── Repositories ────────────────────────────────────────────────────

    let memory_repository: Arc<dyn MemoryRepository> =
        Arc::new(SeaOrmObservationRepository::new((*db).clone()));
    let agent_repository: Arc<dyn AgentRepository> =
        Arc::new(SeaOrmAgentRepository::new(Arc::clone(&db)));
    let project_repository: Arc<dyn ProjectRepository> =
        Arc::new(SeaOrmProjectRepository::new((*db).clone()));
    let entity_repo = Arc::new(SeaOrmEntityRepository::new(Arc::clone(&db)));
    let vcs_entity_repository: Arc<dyn VcsEntityRepository> = Arc::clone(&entity_repo) as _;
    let plan_entity_repository: Arc<dyn PlanEntityRepository> = Arc::clone(&entity_repo) as _;
    let issue_entity_repository: Arc<dyn IssueEntityRepository> = Arc::clone(&entity_repo) as _;
    let org_entity_repository: Arc<dyn OrgEntityRepository> = Arc::clone(&entity_repo) as _;
    let file_hash_repository: Arc<dyn FileHashRepository> =
        Arc::new(SeaOrmIndexRepository::new(Arc::clone(&db), project_id));

    let vcs_provider = crate::di::vcs::default_vcs_provider();
    let project_service: Arc<dyn ProjectDetectorService> = Arc::new(ProjectService::new());
    let crypto_service = Arc::new(create_crypto_service(&config)?);

    Ok(AppContext {
        config,
        embedding_provider,
        vector_store_provider,
        cache_provider,
        language_chunker,
        event_bus,
        indexing_ops,
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
        crypto_service,
    })
}

/// Initialize application for testing.
///
/// # Errors
///
/// Returns an error if application initialization fails.
pub async fn init_test_app() -> Result<AppContext> {
    let config = ConfigLoader::new().load()?;
    init_app(config).await
}

// =========================================================================
// Helpers
// =========================================================================

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
            mcb_domain::error::Error::config("cannot determine project ID from current directory")
        })
}

// ============================================================================
// Test Cache Implementation
// ============================================================================

/// Simple in-memory cache for test infrastructure.
/// Replaces the linkme-registered cache providers for test-only bootstrap.
#[derive(Debug)]
struct TestCache {
    data: Mutex<HashMap<String, String>>,
}

impl TestCache {
    fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl CacheProvider for TestCache {
    async fn get_json(&self, key: &str) -> Result<Option<String>> {
        let data = self
            .data
            .lock()
            .map_err(|_| mcb_domain::error::Error::Infrastructure {
                message: "Cache lock poisoned".to_owned(),
                source: None,
            })?;
        Ok(data.get(key).cloned())
    }

    async fn set_json(&self, key: &str, value: &str, _config: CacheEntryConfig) -> Result<()> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| mcb_domain::error::Error::Infrastructure {
                message: "Cache lock poisoned".to_owned(),
                source: None,
            })?;
        data.insert(key.to_owned(), value.to_owned());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| mcb_domain::error::Error::Infrastructure {
                message: "Cache lock poisoned".to_owned(),
                source: None,
            })?;
        Ok(data.remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let data = self
            .data
            .lock()
            .map_err(|_| mcb_domain::error::Error::Infrastructure {
                message: "Cache lock poisoned".to_owned(),
                source: None,
            })?;
        Ok(data.contains_key(key))
    }

    async fn clear(&self) -> Result<()> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| mcb_domain::error::Error::Infrastructure {
                message: "Cache lock poisoned".to_owned(),
                source: None,
            })?;
        data.clear();
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats> {
        let data = self
            .data
            .lock()
            .map_err(|_| mcb_domain::error::Error::Infrastructure {
                message: "Cache lock poisoned".to_owned(),
                source: None,
            })?;
        Ok(CacheStats {
            hits: 0,
            misses: 0,
            entries: data.len() as u64,
            hit_rate: 0.0,
            bytes_used: 0,
        })
    }

    async fn size(&self) -> Result<usize> {
        let data = self
            .data
            .lock()
            .map_err(|_| mcb_domain::error::Error::Infrastructure {
                message: "Cache lock poisoned".to_owned(),
                source: None,
            })?;
        Ok(data.len())
    }

    fn provider_name(&self) -> &str {
        "test"
    }
}
