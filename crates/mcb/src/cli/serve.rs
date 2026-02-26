use std::sync::Arc;

use clap::Args;
use loco_rs::app::Hooks;
use loco_rs::boot::{self, ServeParams, StartMode};
use loco_rs::environment::Environment;

use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::resolution_context::ServiceResolutionContext;
use mcb_server::build_mcp_server_bootstrap;
use mcb_server::tools::ExecutionFlow;
use mcb_server::transport::stdio::StdioServerExt;

use crate::loco_app::McbApp;

/// Arguments for the `serve` subcommand.
#[derive(Args, Debug, Clone)]
pub struct ServeArgs {
    /// Run as server daemon (HTTP only, no stdio).
    #[arg(long, help = "Run as server daemon (HTTP only, no stdio)")]
    pub server: bool,
    /// Run in stdio-only mode (MCP over stdin/stdout, no HTTP server).
    #[arg(long, help = "Stdio-only mode for MCP clients (no HTTP server)")]
    pub stdio: bool,
}

impl ServeArgs {
    /// # Errors
    /// Returns an error if Loco boot or MCP server initialization fails.
    pub async fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server {
            // SAFETY: called once at startup before any other threads are spawned.
            #[allow(unsafe_code)]
            unsafe {
                std::env::set_var("MCB_NO_STDIO", "1");
            }
        }
        let environment = Environment::from(loco_rs::environment::resolve_from_env());
        let loco_config = McbApp::load_config(&environment).await?;
        let boot_result =
            McbApp::boot(StartMode::server_only(), &environment, loco_config.clone()).await?;
        if self.stdio {
            // Stdio-only mode: build MCP server from Loco context, serve stdio directly.
            let app_config: AppConfig = serde_json::from_value(serde_json::to_value(
                boot_result.app_context.config.clone(),
            )?)?;
            let event_bus = mcb_domain::registry::events::resolve_event_bus_provider(
                &mcb_domain::registry::events::EventBusProviderConfig::new(
                    app_config
                        .system
                        .infrastructure
                        .event_bus
                        .provider
                        .provider_name(),
                ),
            )
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            let resolution_ctx = ServiceResolutionContext {
                db: boot_result.app_context.db.clone(),
                config: Arc::new(app_config),
                event_bus,
            };
            let bootstrap = build_mcp_server_bootstrap(&resolution_ctx, ExecutionFlow::StdioOnly)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            let server = (*bootstrap.mcp_server).clone();
            server.serve_stdio().await?;
        } else {
            // Default: HTTP server + background stdio (unless --server).
            let serve = ServeParams {
                port: loco_config.server.port,
                binding: loco_config.server.binding.clone(),
            };
            boot::start::<McbApp>(boot_result, serve, false).await?;
        }
        Ok(())
    }
}
