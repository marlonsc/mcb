//! Loco.rs application for MCB.
//!
//! Replaces the custom `init.rs` bootstrap with Loco's lifecycle.
//! All MCP services are fully wired in `after_routes()` using Loco's database
//! connection and MCB's provider resolvers.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::Result;
use loco_rs::app::{AppContext as LocoAppContext, Hooks};
use loco_rs::bgworker::Queue;
use loco_rs::boot::{BootResult, StartMode, create_app};
use loco_rs::config::Config as LocoConfig;
use loco_rs::controller::AppRoutes;
use loco_rs::environment::Environment;
use loco_rs::task::Tasks;
use sea_orm::DatabaseConnection;

use mcb_domain::ports::{
    AgentRepository, EventBusProvider, FileHashRepository, IndexingOperationsInterface,
    IssueEntityRepository, MemoryRepository, OrgEntityRepository, PlanEntityRepository,
    ProjectDetectorService, ProjectRepository, VcsEntityRepository,
};
use mcb_infrastructure::cache::provider::SharedCacheProvider;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::modules::domain_services::{
    DomainServicesFactory, ServiceDependencies,
};
use mcb_infrastructure::di::provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};
use mcb_infrastructure::infrastructure::admin::DefaultIndexingOperations;
use mcb_providers::database::seaorm::migration::Migrator;
use mcb_providers::database::seaorm::repos::{
    SeaOrmAgentRepository, SeaOrmEntityRepository, SeaOrmIndexRepository,
    SeaOrmObservationRepository, SeaOrmProjectRepository,
};
use mcb_providers::events::TokioEventBusProvider;

use crate::McpServer;
use crate::mcp_server::{McpEntityRepositories, McpServices};
use crate::transport::http::HttpTransportState;
use crate::transport::stdio::StdioServerExt;

// =========================================================================
// Loco Application
// =========================================================================

/// MCB Loco application — the single entry point for all operating modes.
#[derive(Debug)]
pub struct McbApp;

#[async_trait]
impl Hooks for McbApp {
    fn app_name() -> &'static str {
        "mcb"
    }

    fn app_version() -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: LocoConfig,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    fn routes(_ctx: &LocoAppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
    }

    async fn after_routes(router: AxumRouter, ctx: &LocoAppContext) -> Result<AxumRouter> {
        let server = create_mcp_server(ctx.db.clone())
            .await
            .map_err(|e| loco_rs::Error::string(&format!("MCP server init failed: {e}")))?;
        let server = Arc::new(server);

        // Spawn MCP stdio transport unless MCB_NO_STDIO is set
        if std::env::var("MCB_NO_STDIO").is_err() {
            let stdio_server = (*server).clone();
            tokio::spawn(async move {
                if let Err(e) = stdio_server.serve_stdio().await {
                    tracing::error!(error = %e, "MCP stdio server stopped with error");
                }
            });
        }

        // Mount MCP HTTP endpoint on Loco's router
        let mcp_state = Arc::new(HttpTransportState {
            server: Arc::clone(&server),
        });
        let mcp_routes = axum::Router::new()
            .route(
                "/mcp",
                axum::routing::post(crate::transport::http::handle_mcp_request),
            )
            .with_state(mcp_state);

        Ok(router.merge(mcp_routes))
    }

    async fn connect_workers(_ctx: &LocoAppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn truncate(_ctx: &LocoAppContext) -> Result<()> {
        Ok(())
    }

    async fn seed(_ctx: &LocoAppContext, _path: &Path) -> Result<()> {
        Ok(())
    }
}

// =========================================================================
// MCP Server Factory
// =========================================================================

/// Creates a fully-wired MCP server from a Loco database connection.
///
/// Replaces the old `init.rs` bootstrap chain:
///   `load_config → init_app → build_domain_services → McpServer::new`
///
/// Provider resolution uses MCB's config (Figment/TOML) for domain settings
/// (embedding provider, vector store, etc.). The database connection comes
/// from Loco's `AppContext.db`.
async fn create_mcp_server(
    db: DatabaseConnection,
) -> std::result::Result<McpServer, Box<dyn std::error::Error>> {
    // ── MCB domain config (provider settings, MCP config, etc.) ─────────
    let config = mcb_infrastructure::config::ConfigLoader::new().load()?;
    let config = Arc::new(config);

    // ── Resolve providers directly (no handles, no admin switching) ──────
    let embedding_provider = EmbeddingProviderResolver::new(Arc::clone(&config))
        .resolve_from_config()
        .map_err(|e| format!("Embedding provider: {e}"))?;
    let vector_store_provider = VectorStoreProviderResolver::new(Arc::clone(&config))
        .resolve_from_config()
        .map_err(|e| format!("Vector store provider: {e}"))?;
    let cache_provider = CacheProviderResolver::new(Arc::clone(&config))
        .resolve_from_config()
        .map_err(|e| format!("Cache provider: {e}"))?;
    let language_chunker = LanguageProviderResolver::new(Arc::clone(&config))
        .resolve_from_config()
        .map_err(|e| format!("Language provider: {e}"))?;

    // ── Infrastructure services ─────────────────────────────────────────
    let event_bus: Arc<dyn EventBusProvider> = Arc::new(TokioEventBusProvider::new());
    let indexing_ops: Arc<dyn IndexingOperationsInterface> =
        Arc::new(DefaultIndexingOperations::new());

    // ── SeaORM repositories (from Loco's database connection) ───────────
    let db_arc = Arc::new(db.clone());
    let project_id = current_project_id()?;

    let memory_repository: Arc<dyn MemoryRepository> =
        Arc::new(SeaOrmObservationRepository::new(db));
    let agent_repository: Arc<dyn AgentRepository> =
        Arc::new(SeaOrmAgentRepository::new(Arc::clone(&db_arc)));
    let project_repository: Arc<dyn ProjectRepository> =
        Arc::new(SeaOrmProjectRepository::new((*db_arc).clone()));
    let entity_repo = Arc::new(SeaOrmEntityRepository::new(Arc::clone(&db_arc)));
    let vcs_entity_repository: Arc<dyn VcsEntityRepository> = Arc::clone(&entity_repo) as _;
    let plan_entity_repository: Arc<dyn PlanEntityRepository> = Arc::clone(&entity_repo) as _;
    let issue_entity_repository: Arc<dyn IssueEntityRepository> = Arc::clone(&entity_repo) as _;
    let org_entity_repository: Arc<dyn OrgEntityRepository> = Arc::clone(&entity_repo) as _;
    let file_hash_repository: Arc<dyn FileHashRepository> = Arc::new(SeaOrmIndexRepository::new(
        Arc::clone(&db_arc),
        project_id.clone(),
    ));

    // ── Domain-level services ───────────────────────────────────────────
    let vcs_provider = mcb_infrastructure::di::vcs::default_vcs_provider();
    let project_service: Arc<dyn ProjectDetectorService> =
        Arc::new(mcb_infrastructure::project::ProjectService::new());
    let crypto_service = create_crypto_service(&config)?;

    // ── Build domain services via factory ────────────────────────────────
    let shared_cache = SharedCacheProvider::from_arc(cache_provider);
    let deps = ServiceDependencies {
        project_id,
        cache: shared_cache,
        crypto: Arc::new(crypto_service),
        config: (*config).clone(),
        embedding_provider,
        vector_store_provider,
        language_chunker,
        indexing_ops,
        event_bus,
        memory_repository,
        agent_repository,
        file_hash_repository,
        vcs_provider: Arc::clone(&vcs_provider),
        project_service: Arc::clone(&project_service),
        project_repository: Arc::clone(&project_repository),
        vcs_entity_repository: Arc::clone(&vcs_entity_repository),
        plan_entity_repository: Arc::clone(&plan_entity_repository),
        issue_entity_repository: Arc::clone(&issue_entity_repository),
        org_entity_repository: Arc::clone(&org_entity_repository),
    };
    let services = DomainServicesFactory::create_services(deps)
        .await
        .map_err(|e| format!("Domain services: {e}"))?;

    // ── Assemble McpServer ──────────────────────────────────────────────
    let mcp_services = McpServices {
        indexing: services.indexing_service,
        context: services.context_service,
        search: services.search_service,
        validation: services.validation_service,
        memory: services.memory_service,
        agent_session: services.agent_session_service,
        project: services.project_service,
        project_workflow: services.project_repository,
        vcs: services.vcs_provider,
        entities: McpEntityRepositories {
            vcs: services.vcs_entity_repository,
            plan: services.plan_entity_repository,
            issue: services.issue_entity_repository,
            org: services.org_entity_repository,
        },
    };

    Ok(McpServer::new(mcp_services, Some("loco".to_owned())))
}

// =========================================================================
// Helpers
// =========================================================================

fn current_project_id() -> std::result::Result<String, Box<dyn std::error::Error>> {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
        .ok_or_else(|| "Cannot determine project ID from current directory".into())
}

fn create_crypto_service(
    config: &AppConfig,
) -> std::result::Result<mcb_infrastructure::crypto::CryptoService, Box<dyn std::error::Error>> {
    let master_key = if config.auth.jwt.secret.len() >= 32 {
        config.auth.jwt.secret.as_bytes()[..32].to_vec()
    } else {
        mcb_infrastructure::crypto::CryptoService::generate_master_key()
    };
    mcb_infrastructure::crypto::CryptoService::new(master_key)
        .map_err(|e| format!("Crypto service: {e}").into())
}
