//! Centralized public API surface for mcb-server.
//! Re-exports only; module declarations stay in lib.rs.

pub use crate::composition::build_mcp_server_bootstrap;
pub use crate::mcp_server::McpServer;
pub use crate::state::McbState;
