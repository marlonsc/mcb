//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Transport layer types
//!
//! Re-exports canonical MCP protocol types from `mcb_domain::protocol`.
//! All transport code MUST use these types instead of defining new ones.

// Re-export canonical domain protocol types for backward compatibility.
pub use mcb_domain::protocol::{JSONRPC_VERSION, McpError, McpRequest, McpResponse};
