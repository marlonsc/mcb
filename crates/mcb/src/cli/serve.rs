use clap::Args;
use loco_rs::app::Hooks;
use loco_rs::boot::{self, ServeParams, StartMode};
use loco_rs::environment::Environment;
use mcb_server::McbApp;
use mcb_server::loco_app::create_mcp_server;
use mcb_server::tools::ExecutionFlow;
use mcb_server::transport::stdio::StdioServerExt;

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
            McbApp::boot(StartMode::ServerOnly, &environment, loco_config.clone()).await?;
        if self.stdio {
            // Stdio-only mode: create MCP server directly from Loco context, no HTTP.
            let server = create_mcp_server(&boot_result.app_context, ExecutionFlow::StdioOnly).await?;
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
