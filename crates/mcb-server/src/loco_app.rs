use crate::McpServer;
use crate::mcp_server::{McpEntityRepositories, McpServices};
use crate::state::McbState;
use crate::tools::ExecutionFlow;
use crate::transport::http::HttpTransportState;
use crate::transport::stdio::StdioServerExt;
use async_trait::async_trait;
use axum::Extension;
use axum::Router as AxumRouter;
use loco_rs::Result;
use loco_rs::app::{AppContext as LocoAppContext, Hooks, Initializer};
use loco_rs::bgworker::Queue;
use loco_rs::boot::{BootResult, StartMode, create_app};
use loco_rs::config::Config as LocoConfig;
use loco_rs::controller::AppRoutes;
use loco_rs::environment::Environment;
use loco_rs::task::Tasks;
use mcb_infrastructure::di::modules::domain_services::DomainServicesFactory;
use mcb_infrastructure::loco::bridge::LocoBridge;
use mcb_providers::database::seaorm::{SeaOrmAuthRepositoryAdapter, SeaOrmDashboardAdapter};
use mcb_providers::migration::Migrator;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

    async fn load_config(env: &Environment) -> loco_rs::Result<LocoConfig> {
        if let Ok(folder) = std::env::var("MCB_CONFIG_FOLDER") {
            return env.load_from_folder(Path::new(&folder));
        }

        let env_name = loco_rs::environment::resolve_from_env();
        let local_candidates = [
            PathBuf::from("config").join(format!("{env_name}.local.yaml")),
            PathBuf::from("config").join(format!("{env_name}.yaml")),
        ];
        if local_candidates.iter().any(|p| p.exists()) {
            return env.load_from_folder(Path::new("config"));
        }

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
        let mcp_server = create_mcp_server(ctx, ExecutionFlow::ServerHybrid)
            .await
            .map_err(|e| loco_rs::Error::string(&format!("MCP server init failed: {e}")))?;

        if std::env::var("MCB_NO_STDIO").is_err() {
            let stdio_server = Arc::clone(&mcp_server);
            tokio::spawn(async move {
                if let Err(e) = stdio_server.serve_stdio().await {
                    tracing::error!(error = %e, "MCP stdio server stopped with error");
                }
            });
        }

        let dashboard_adapter = Arc::new(SeaOrmDashboardAdapter::new(ctx.db.clone()));
        let auth_adapter = Arc::new(SeaOrmAuthRepositoryAdapter::new(ctx.db.clone()));
        let mcb_state = McbState::new(dashboard_adapter, auth_adapter, Arc::clone(&mcp_server));
        let router = router.layer(Extension(mcb_state));

        let mcp_state = Arc::new(HttpTransportState {
            server: Arc::clone(&mcp_server),
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

pub async fn create_mcp_server(
    ctx: &LocoAppContext,
    execution_flow: ExecutionFlow,
) -> std::result::Result<Arc<McpServer>, Box<dyn std::error::Error + Send + Sync>> {
    let bridge = LocoBridge::new(ctx)?;
    build_mcp_server_from_bridge(&bridge, execution_flow).await
}

async fn build_mcp_server_from_bridge(
    bridge: &LocoBridge,
    execution_flow: ExecutionFlow,
) -> std::result::Result<Arc<McpServer>, Box<dyn std::error::Error + Send + Sync>> {
    let services = DomainServicesFactory::create_services(bridge.build_service_dependencies()?)
        .await
        .map_err(|e| format!("Domain services: {e}"))?;

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
    Ok(Arc::new(McpServer::new(
        mcp_services,
        &vcs_for_defaults,
        Some(execution_flow),
    )))
}
