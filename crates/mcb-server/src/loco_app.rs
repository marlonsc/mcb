//! Loco.rs application for MCB.
//!
//! Replaces the custom `init.rs` bootstrap with Loco's lifecycle.
//! All MCP services are fully wired in `after_routes()` using Loco's database
//! connection and MCB's provider resolvers.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::Result;
use loco_rs::app::{AppContext as LocoAppContext, Hooks, Initializer};
use loco_rs::bgworker::Queue;
use loco_rs::boot::{BootResult, StartMode, create_app};
use loco_rs::config::Config as LocoConfig;
use loco_rs::controller::AppRoutes;
use loco_rs::environment::Environment;
use loco_rs::task::Tasks;

use mcb_domain::ports::{EventBusProvider, IndexingOperationsInterface, ProjectDetectorService};
use mcb_domain::registry::database::resolve_database_repositories;
use mcb_infrastructure::cache::CacheAdapter;
use mcb_infrastructure::cache::provider::SharedCacheProvider;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::modules::domain_services::{
    DomainServicesFactory, ServiceDependencies,
};
use mcb_infrastructure::di::provider_resolvers::{
    EmbeddingProviderResolver, LanguageProviderResolver, VectorStoreProviderResolver,
};
use mcb_infrastructure::events::BroadcastEventBus;
use mcb_infrastructure::infrastructure::admin::DefaultIndexingOperations;
use mcb_providers::migration::Migrator;

use crate::McpServer;
use crate::mcp_server::{McpEntityRepositories, McpServices};
use crate::tools::ExecutionFlow;
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

    /// MCB config resolution (overrides Loco default).
    ///
    /// Resolution order:
    /// 1. `MCB_CONFIG_FOLDER` env var (explicit override)
    /// 2. `config/` in CWD (standard Loco dev layout)
    /// 3. `~/.config/mcb/config/` (installed binary)
    async fn load_config(env: &Environment) -> loco_rs::Result<LocoConfig> {
        // 1. Explicit MCB_CONFIG_FOLDER
        if let Ok(folder) = std::env::var("MCB_CONFIG_FOLDER") {
            return env.load_from_folder(Path::new(&folder));
        }

        // 2. Local config/ (standard Loco dev layout)
        let env_name = loco_rs::environment::resolve_from_env();
        let local_candidates = [
            PathBuf::from("config").join(format!("{env_name}.local.yaml")),
            PathBuf::from("config").join(format!("{env_name}.yaml")),
        ];
        if local_candidates.iter().any(|p| p.exists()) {
            return env.load_from_folder(Path::new("config"));
        }

        // 3. Installed config: ~/.config/mcb/config/
        let installed = dirs::config_dir()
            .ok_or_else(|| loco_rs::Error::string("Cannot determine config directory"))?
            .join("mcb")
            .join("config");
        env.load_from_folder(&installed)
    }

    fn routes(_ctx: &LocoAppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .prefix("/api")
            .add_route(crate::controllers::admin::routes())
            .add_route(crate::controllers::graphql::routes())
    }

    async fn initializers(_ctx: &LocoAppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(
            crate::initializers::graphql::GraphQLInitializer,
        )])
    }

    async fn after_routes(router: AxumRouter, ctx: &LocoAppContext) -> Result<AxumRouter> {
        let server = create_mcp_server(ctx, ExecutionFlow::ServerHybrid)
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

/// Creates a fully-wired MCP server from a Loco application context.
///
/// Extracts `AppConfig` from Loco's `config.settings` and uses `ctx.db`
/// for the database connection. No separate config loading step needed.
///
/// # Errors
///
/// Returns an error if settings extraction or provider resolution fails.
pub async fn create_mcp_server(
    ctx: &LocoAppContext,
    execution_flow: ExecutionFlow,
) -> std::result::Result<McpServer, Box<dyn std::error::Error>> {
    // ── Extract MCB config from Loco settings ───────────────────────────
    let settings_value =
        ctx.config.settings.clone().ok_or(
            "No 'settings' in Loco config. Ensure config/{env}.yaml has a 'settings:' key.",
        )?;
    let config: AppConfig = serde_json::from_value(settings_value)
        .map_err(|e| format!("Failed to deserialize MCB settings from Loco config: {e}"))?;
    let config = Arc::new(config);
    let db = ctx.db.clone();

    // ── Resolve providers directly (no handles, no admin switching) ──────
    let embedding_provider = EmbeddingProviderResolver::new(Arc::clone(&config))
        .resolve_from_config()
        .map_err(|e| format!("Embedding provider: {e}"))?;
    let vector_store_provider = VectorStoreProviderResolver::new(Arc::clone(&config))
        .resolve_from_config()
        .map_err(|e| format!("Vector store provider: {e}"))?;
    let cache_provider: Arc<dyn mcb_domain::ports::CacheProvider> = Arc::new(CacheAdapter::new(
        std::sync::Arc::<loco_rs::cache::Cache>::clone(&ctx.cache),
    ));
    let language_chunker = LanguageProviderResolver::new()
        .resolve_from_config()
        .map_err(|e| format!("Language provider: {e}"))?;

    // ── Infrastructure services ─────────────────────────────────────────
    let event_bus: Arc<dyn EventBusProvider> = Arc::new(BroadcastEventBus::new());
    let indexing_ops: Arc<dyn IndexingOperationsInterface> =
        Arc::new(DefaultIndexingOperations::new());

    // ── SeaORM repositories (from Loco's database connection) ───────────
    let db_arc = Arc::new(db);
    let project_id = current_project_id()?;

    let repos =
        resolve_database_repositories("seaorm", Box::new((*db_arc).clone()), project_id.clone())
            .map_err(|e| format!("Database repositories: {e}"))?;
    let memory_repository = repos.memory;
    let agent_repository = repos.agent;
    let project_repository = repos.project;
    let vcs_entity_repository = repos.vcs_entity;
    let plan_entity_repository = repos.plan_entity;
    let issue_entity_repository = repos.issue_entity;
    let org_entity_repository = repos.org_entity;
    let file_hash_repository = repos.file_hash;

    // ── Domain-level services ───────────────────────────────────────────
    let vcs_provider = mcb_infrastructure::di::vcs::default_vcs_provider();
    let detect_fn: mcb_infrastructure::project::DetectAllFn = std::sync::Arc::new(|path: &std::path::Path| {
        let path = path.to_path_buf();
        Box::pin(async move {
            mcb_providers::project_detection::detect_all_projects(&path).await
        })
    });
    let project_service: Arc<dyn ProjectDetectorService> =
        Arc::new(mcb_infrastructure::project::ProjectService::new(detect_fn));
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

    let vcs_for_defaults = Arc::clone(&mcp_services.vcs);
    Ok(McpServer::new(
        mcp_services,
        &vcs_for_defaults,
        Some(execution_flow),
    ))
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
