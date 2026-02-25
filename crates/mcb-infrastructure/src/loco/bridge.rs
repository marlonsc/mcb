//! Loco bridge: composes Loco framework resources into MCB's DI and domain services.

use std::sync::Arc;

use loco_rs::app::AppContext as LocoAppContext;
use mcb_domain::ports::{
    CacheProvider, EventBusProvider, IndexingOperationsInterface, ProjectDetectorService,
};
use mcb_domain::registry::database::resolve_database_repositories;
use sea_orm::DatabaseConnection;

use crate::cache::CacheAdapter;
use crate::cache::provider::SharedCacheProvider;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::di::modules::{DomainServicesFactory, ServiceDependencies};
use crate::di::provider_resolvers::{
    EmbeddingProviderResolver, LanguageProviderResolver, VectorStoreProviderResolver,
};
use crate::di::vcs::default_vcs_provider;
use crate::events::BroadcastEventBus;
use crate::infrastructure::admin::DefaultIndexingOperations;
use crate::project::ProjectService;

/// Composition root for Loco framework integration.
///
/// Wraps `DomainServicesFactory` and extracts Loco resources.
/// Bridges the Loco framework's `AppContext` to MCB's dependency injection system.
#[allow(dead_code)]
pub struct LocoBridge {
    db: DatabaseConnection,
    cache: Arc<dyn CacheProvider>,
    config: Arc<AppConfig>,
}

impl LocoBridge {
    /// Create new `LocoBridge` from `LocoAppContext`
    ///
    /// # Arguments
    ///
    /// * `ctx` - The Loco application context containing framework resources
    ///
    /// # Returns
    ///
    /// A new `LocoBridge` instance or an error if resource extraction fails
    ///
    /// # Errors
    ///
    /// Returns an error if required resources cannot be extracted from the context
    pub fn new(ctx: &LocoAppContext) -> Result<Self, Box<dyn std::error::Error>> {
        let settings_value = ctx.config.settings.clone().ok_or(
            "No 'settings' in Loco config. Ensure config/{env}.yaml has a 'settings:' key.",
        )?;
        let config: AppConfig = serde_json::from_value(settings_value)
            .map_err(|e| format!("Failed to deserialize MCB settings from Loco config: {e}"))?;

        let cache_provider: Arc<dyn CacheProvider> =
            Arc::new(CacheAdapter::new(Arc::clone(&ctx.cache)));

        Ok(Self {
            db: ctx.db.clone(),
            cache: cache_provider,
            config: Arc::new(config),
        })
    }

    /// Build `ServiceDependencies` for `DomainServicesFactory`
    ///
    /// Extracts all required dependencies from the Loco context and assembles
    /// them into a `ServiceDependencies` struct for factory consumption.
    ///
    /// # Returns
    ///
    /// A `ServiceDependencies` struct ready for `DomainServicesFactory::create_services()`
    #[must_use]
    pub fn build_service_dependencies(
        &self,
    ) -> Result<ServiceDependencies, Box<dyn std::error::Error>> {
        let embedding_provider = EmbeddingProviderResolver::new(Arc::clone(&self.config))
            .resolve_from_config()
            .map_err(|e| format!("Embedding provider: {e}"))?;
        let vector_store_provider = VectorStoreProviderResolver::new(Arc::clone(&self.config))
            .resolve_from_config()
            .map_err(|e| format!("Vector store provider: {e}"))?;
        let language_chunker = LanguageProviderResolver::new()
            .resolve_from_config()
            .map_err(|e| format!("Language provider: {e}"))?;

        let event_bus: Arc<dyn EventBusProvider> = Arc::new(BroadcastEventBus::new());
        let indexing_ops: Arc<dyn IndexingOperationsInterface> =
            Arc::new(DefaultIndexingOperations::new());

        let project_id = current_project_id()?;
        let db_arc = Arc::new(self.db.clone());
        let repos = resolve_database_repositories(
            "seaorm",
            Box::new((*db_arc).clone()),
            project_id.clone(),
        )
        .map_err(|e| format!("Database repositories: {e}"))?;

        let vcs_provider = default_vcs_provider();
        let detect_fn: crate::project::DetectAllFn = Arc::new(|path: &std::path::Path| {
            let path = path.to_path_buf();
            Box::pin(
                async move { mcb_providers::project_detection::detect_all_projects(&path).await },
            )
        });
        let project_service: Arc<dyn ProjectDetectorService> =
            Arc::new(ProjectService::new(detect_fn));
        let crypto_service = create_crypto_service(&self.config)?;
        let shared_cache = SharedCacheProvider::from_arc(Arc::clone(&self.cache));

        Ok(ServiceDependencies {
            project_id,
            cache: shared_cache,
            crypto: Arc::new(crypto_service),
            config: (*self.config).clone(),
            embedding_provider,
            vector_store_provider,
            language_chunker,
            indexing_ops,
            event_bus,
            memory_repository: repos.memory,
            agent_repository: repos.agent,
            file_hash_repository: repos.file_hash,
            vcs_provider,
            project_service,
            project_repository: repos.project,
            vcs_entity_repository: repos.vcs_entity,
            plan_entity_repository: repos.plan_entity,
            issue_entity_repository: repos.issue_entity,
            org_entity_repository: repos.org_entity,
        })
    }

    /// Build MCP server via `LocoBridge`
    ///
    /// Orchestrates the full composition: extracts Loco resources → builds `ServiceDependencies`
    /// → creates domain services → initializes MCP server.
    ///
    /// # Arguments
    ///
    /// * `_flow` - The execution flow configuration for the MCP server (deferred to Task 13)
    ///
    /// # Returns
    ///
    /// An initialized MCP server instance or an error if composition fails (deferred to Task 13)
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the composition pipeline fails
    pub async fn build_mcp_server(&self, _flow: ()) -> Result<(), Box<dyn std::error::Error>> {
        let deps = self.build_service_dependencies()?;
        DomainServicesFactory::create_services(deps)
            .await
            .map_err(|e| format!("Domain services: {e}"))?;
        Ok(())
    }
}

fn current_project_id() -> Result<String, Box<dyn std::error::Error>> {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
        .ok_or_else(|| "Cannot determine project ID from current directory".into())
}

fn create_crypto_service(config: &AppConfig) -> Result<CryptoService, Box<dyn std::error::Error>> {
    let secret_bytes = config.auth.jwt.secret.as_bytes();
    let master_key = if secret_bytes.is_empty() {
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
        return Err("JWT secret must be exactly 32 bytes long".into());
    } else {
        secret_bytes.to_vec()
    };

    CryptoService::new(master_key).map_err(|e| e.into())
}
