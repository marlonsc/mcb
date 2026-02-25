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
    PlanEntityRepository, ProjectDetectorService, ProjectRepository, ValidationOperationsInterface,
    VcsEntityRepository, VcsProvider, VectorStoreProvider,
};
use mcb_domain::registry::database::resolve_database_repositories;

use crate::config::{AppConfig, ConfigLoader};
use crate::constants::providers::DEFAULT_DB_CONFIG_NAME;
use crate::crypto::CryptoService;
use crate::di::provider_resolvers::{
    EmbeddingProviderResolver, LanguageProviderResolver, VectorStoreProviderResolver,
};
use crate::events::BroadcastEventBus;
use crate::infrastructure::admin::{DefaultIndexingOperations, DefaultValidationOperations};
use crate::project::ProjectService;
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
    validation_ops_arc: Arc<dyn ValidationOperationsInterface>,

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

    /// Get validation operations
    #[must_use]
    pub fn validation_ops(&self) -> Arc<dyn ValidationOperationsInterface> {
        Arc::clone(&self.validation_ops_arc)
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
            validation_ops: Arc::new(DefaultValidationOperations::new()),
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

    let language_chunker = LanguageProviderResolver::new()
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("Language: {e}")))?;

    // ── Infrastructure services ─────────────────────────────────────────

    let event_bus: Arc<dyn EventBusProvider> = Arc::new(BroadcastEventBus::new());
    let indexing_ops: Arc<dyn IndexingOperationsInterface> =
        Arc::new(DefaultIndexingOperations::new());
    let validation_ops: Arc<dyn ValidationOperationsInterface> =
        Arc::new(DefaultValidationOperations::new());

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

    let db_url = if memory_db_path.to_string_lossy().contains("://") {
        memory_db_path.to_string_lossy().into_owned()
    } else {
        format!("sqlite://{}?mode=rwc", memory_db_path.display())
    };
    let mut connect_opts = ConnectOptions::new(db_url);
    connect_opts
        .max_connections(5)
        .min_connections(1)
        .sqlx_logging(false);

    let db: DatabaseConnection = Database::connect(connect_opts).await.map_err(|e| {
        mcb_domain::error::Error::internal(format!("Failed to connect to database: {e}"))
    })?;

    use sea_orm_migration::MigratorTrait;
    mcb_providers::migration::Migrator::up(&db, None) // CA-EXCEPTION: SeaORM migration requirement
        .await
        .map_err(|e| {
            mcb_domain::error::Error::internal(format!("Failed to run migrations: {e}"))
        })?;

    let db = Arc::new(db);
    let project_id = current_project_id()?;

    // ── Repositories (via linkme registry) ─────────────────────────
    let repos = resolve_database_repositories("seaorm", Box::new((*db).clone()), project_id)
        .map_err(mcb_domain::error::Error::configuration)?;
    let memory_repository = repos.memory;
    let agent_repository = repos.agent;
    let project_repository = repos.project;
    let vcs_entity_repository = repos.vcs_entity;
    let plan_entity_repository = repos.plan_entity;
    let issue_entity_repository = repos.issue_entity;
    let org_entity_repository = repos.org_entity;
    let file_hash_repository = repos.file_hash;

    let vcs_provider = crate::di::vcs::default_vcs_provider();
    let detect_fn: crate::project::DetectAllFn = std::sync::Arc::new(|path: &std::path::Path| {
        let path = path.to_path_buf();
        Box::pin(async move {
            mcb_providers::project_detection::detect_all_projects(&path).await // CA-EXCEPTION: DI composition root wires concrete provider
        })
    });
    let project_service: Arc<dyn ProjectDetectorService> = Arc::new(ProjectService::new(detect_fn));
    let crypto_service = Arc::new(create_crypto_service(&config)?);

    Ok(AppContext {
        config,
        embedding_provider,
        vector_store_provider,
        cache_provider,
        language_chunker,
        event_bus,
        indexing_ops,
        validation_ops_arc: validation_ops,
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
    let secret_bytes = config.auth.jwt.secret.as_bytes();
    let master_key = if secret_bytes.is_empty() {
        // ADR-025: default config has empty secret. When auth is disabled,
        // generate a random 32-byte key for internal crypto operations.
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .hash(&mut hasher);
        let h1 = hasher.finish().to_le_bytes();
        hasher.write_u8(0xff);
        let h2 = hasher.finish().to_le_bytes();
        let h3 = hasher.finish().to_le_bytes();
        let h4 = hasher.finish().to_le_bytes();
        let mut key = Vec::with_capacity(32);
        key.extend_from_slice(&h1);
        key.extend_from_slice(&h2);
        key.extend_from_slice(&h3);
        key.extend_from_slice(&h4);
        key
    } else if secret_bytes.len() != 32 {
        return Err(mcb_domain::error::Error::configuration(
            "JWT secret must be exactly 32 bytes long".to_owned(),
        ));
    } else {
        secret_bytes.to_vec()
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
