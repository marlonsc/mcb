//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Transport layer types
//!
//! Re-exports canonical MCP protocol types from `mcb_domain::protocol`.
//! All transport code MUST use these types instead of defining new ones.

// Crate-internal re-exports of canonical domain protocol types.
pub(crate) use mcb_domain::protocol::{McpError, McpRequest, McpResponse};
