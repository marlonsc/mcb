use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::Result;
use loco_rs::app::{AppContext as LocoAppContext, Hooks, Initializer};
use loco_rs::boot::{BootResult, StartMode, create_app};
use loco_rs::config::Config as LocoConfig;
use loco_rs::controller::AppRoutes;
use loco_rs::environment::Environment;
use mcb_providers::migration::Migrator;
use std::path::{Path, PathBuf};

/// MCB Loco application type implementing [`Hooks`].
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
            .add_route(mcb_server::controllers::admin::routes())
            .add_route(mcb_server::controllers::graphql::routes())
    }
    async fn initializers(_ctx: &LocoAppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![
            Box::new(crate::initializers::graphql::GraphQLInitializer),
            Box::new(crate::initializers::mcp_server::McpServerInitializer),
        ])
    }

    async fn after_routes(router: AxumRouter, _ctx: &LocoAppContext) -> Result<AxumRouter> {
        Ok(router)
    }
    async fn connect_workers(
        _ctx: &LocoAppContext,
        _queue: &loco_rs::bgworker::Queue,
    ) -> Result<()> {
        Ok(())
    }
    fn register_tasks(_tasks: &mut loco_rs::task::Tasks) {}
    async fn truncate(_ctx: &LocoAppContext) -> Result<()> {
        Ok(())
    }
    async fn seed(_ctx: &LocoAppContext, _path: &Path) -> Result<()> {
        Ok(())
    }
}
