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

/// Re-export of the domain layer.
pub mod domain {
    pub use mcb_domain::*;
}

/// Re-export of the server layer.
pub mod server {
    pub use mcb_server::*;
}

/// Re-export of the infrastructure layer.
pub mod infrastructure {
    pub use mcb_infrastructure::*;
}

pub use crate::loco_app::McbApp;
pub use domain::*;
pub use server::McpServer;
/// Loco initializers for the MCP server.
pub mod initializers;
/// Loco application hook implementation.
pub mod loco_app;
