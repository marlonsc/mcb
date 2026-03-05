use clap::Args;
use loco_rs::app::Hooks;
use loco_rs::boot::{self, ServeParams, StartMode};
use loco_rs::environment::Environment;

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
        let environment = Environment::from(loco_rs::environment::resolve_from_env());
        let mut loco_config = McbApp::load_config(&environment).await?;

        // Inject CLI mode flags into Loco config settings.
        if (self.server || self.stdio)
            && let Some(ref mut settings) = loco_config.settings
            && let Some(mcp) = settings.pointer_mut("/mcp")
            && let Some(mcp_obj) = mcp.as_object_mut()
        {
            if self.server {
                mcp_obj.insert("no_stdio".to_owned(), serde_json::json!(true));
            }
            if self.stdio {
                mcp_obj.insert("stdio_only".to_owned(), serde_json::json!(true));
            }
        }

        let boot_result =
            McbApp::boot(StartMode::server_only(), &environment, loco_config.clone()).await?;

        // Allow SERVER_PORT env var to override the config file port (used for E2E testing).
        let port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(loco_config.server.port);
        let serve = ServeParams {
            port,
            binding: loco_config.server.binding.clone(),
        };
        boot::start::<McbApp>(boot_result, serve, self.stdio).await?;

        Ok(())
    }
}
