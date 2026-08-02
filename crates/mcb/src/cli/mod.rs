//! CLI module for MCP Context Browser
//!
//! Provides subcommand handling for the `mcb` binary:
//! - `serve` - Run as MCP server (default)
//! - `validate` - Run architecture validation

/// MCP server subcommand.
pub mod serve;
/// Architecture validation subcommand.
pub mod validate;

pub use serve::ServeArgs;
pub use validate::ValidateArgs;
