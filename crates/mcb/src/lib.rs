//! MCB binary crate.
//!
//! Command-line entry point and re-export surface for the Memory Context Browser.
//! Supports runtime server modes (HTTP and stdio) plus management subcommands.
//!
//! ## Typical usage
//! ```bash
//! mcb serve --transport http
//! mcb serve --transport stdio
//! ```
pub mod cli;

pub use mcb_server::McpServer;

pub use crate::loco_app::McbApp;
/// Loco initializers for the MCP server.
pub mod initializers;
/// Loco application hook implementation.
pub mod loco_app;
