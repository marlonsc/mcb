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

/// CLI subcommand handlers and arguments.
pub mod cli;
/// Loco initializers for the MCP server.
pub mod initializers;
/// Loco application hooks and specialized application logic.
pub mod loco_app;

pub use crate::loco_app::McbApp;
pub use mcb_server::McpServer;
