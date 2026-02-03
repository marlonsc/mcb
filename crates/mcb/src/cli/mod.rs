//! CLI module for MCP Context Browser
//!
//! Provides subcommand handling for the `mcb` binary:
//! - `serve` - Run as MCP server (default)
//! - `validate` - Run architecture validation

pub mod serve;
pub mod validate;

pub use serve::ServeArgs;
pub use validate::ValidateArgs;
