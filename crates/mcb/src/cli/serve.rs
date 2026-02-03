//! Serve command - runs the MCP server

use clap::Args;
use std::path::PathBuf;

/// Arguments for the serve command
#[derive(Args, Debug, Clone)]
pub struct ServeArgs {
    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Run as server daemon (HTTP + optional stdio)
    ///
    /// When this flag is set, MCB runs as a server daemon that accepts
    /// connections from MCB clients. Without this flag, MCB checks the
    /// config file to determine if it should run in standalone or client mode.
    #[arg(long, help = "Run as server daemon")]
    pub server: bool,
}

impl ServeArgs {
    /// Execute the serve command
    pub async fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        mcb_server::run(self.config.as_deref(), self.server).await
    }
}
