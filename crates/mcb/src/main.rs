//! MCP Context Browser - Entry Point
//!
//! Binary entry point for the MCP Context Browser server.
//! Lives in the `mcb` facade crate to avoid doc output filename collision
//! with the `mcb` library crate (cargo issue #6313).
//!
//! ## Operating Modes
//!
//! | Mode | Command | Description |
//! | ------ | --------- | ------------- |
//! | **Standalone** | `mcb serve` (config: `mode.type = "standalone"`) | Local providers, stdio transport |
//! | **Server** | `mcb serve --server` | HTTP daemon, accepts client connections |
//! | **Client** | `mcb serve` (config: `mode.type = "client"`) | Connects to server via HTTP |
//! | **Validate** | `mcb validate [path]` | Run architecture validation |
//!
//! ## Subcommands
//!
//! - `mcb serve` - Start MCP server
//! - `mcb validate` - Run architecture validation

// Force-link mcb-providers to ensure linkme inventory registrations are included
mod cli;

use clap::{Parser, Subcommand};
use cli::{ServeArgs, ValidateArgs};

/// Command line interface for MCP Context Browser
#[derive(Parser, Debug)]
#[command(name = "mcb")]
#[command(about = "MCP Context Browser - Semantic Code Search Server")]
#[command(version)]
pub struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Serve(args) => args.execute().await,
        Command::Validate(args) => {
            let result = args.execute()?;

            if result.failed() {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
