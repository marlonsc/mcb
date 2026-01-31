//! MCP Context Browser - Entry Point
//!
//! Binary entry point for the MCP Context Browser server.
//! Lives in the `mcb` facade crate to avoid doc output filename collision
//! with the `mcb` library crate (cargo issue #6313).
//!
//! ## Operating Modes
//!
//! | Mode | Command | Description |
//! |------|---------|-------------|
//! | **Standalone** | `mcb serve` (config: `mode.type = "standalone"`) | Local providers, stdio transport |
//! | **Server** | `mcb serve --server` | HTTP daemon, accepts client connections |
//! | **Client** | `mcb serve` (config: `mode.type = "client"`) | Connects to server via HTTP |
//! | **Validate** | `mcb validate [path]` | Run architecture validation |
//!
//! ## Subcommands
//!
//! - `mcb serve` - Start MCP server (default when no subcommand)
//! - `mcb validate` - Run architecture validation
//!
//! ## Backwards Compatibility
//!
//! - Bare `mcb` (no subcommand) defaults to `serve`
//! - `mcb --server` continues to work (deprecated, use `mcb serve --server`)

// Force-link mcb-providers to ensure linkme inventory registrations are included
extern crate mcb_providers;

mod cli;

use clap::{Parser, Subcommand};
use cli::{ServeArgs, ValidateArgs};

/// Command line interface for MCP Context Browser
#[derive(Parser, Debug)]
#[command(name = "mcb")]
#[command(about = "MCP Context Browser - Semantic Code Search Server")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    // Legacy flags for backwards compatibility (when no subcommand is used)
    /// Path to configuration file (deprecated: use `mcb serve --config`)
    #[arg(short, long, global = true)]
    pub config: Option<std::path::PathBuf>,

    /// Run as server daemon (deprecated: use `mcb serve --server`)
    #[arg(long, global = true)]
    pub server: bool,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start MCP server (default when no subcommand)
    #[command(alias = "server")]
    Serve(ServeArgs),

    /// Run architecture validation
    Validate(ValidateArgs),
}

/// Main entry point for the MCP Context Browser
///
/// Dispatches to the appropriate mode based on CLI subcommand:
/// - No subcommand / `serve`: Run as MCP server
/// - `validate`: Run architecture validation
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Serve(args)) => args.execute().await,
        Some(Command::Validate(args)) => {
            let result = args.execute()?;

            // Exit code: 1 if validation failed
            // Normal mode: errors only
            // Strict mode: errors OR warnings
            if result.failed() {
                std::process::exit(1);
            }
            Ok(())
        }
        None => {
            // No subcommand: default to serve with legacy flags
            let args = ServeArgs {
                config: cli.config,
                server: cli.server,
            };
            args.execute().await
        }
    }
}
