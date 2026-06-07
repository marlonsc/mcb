//! Protocol types — domain-level MCP/JSON-RPC data structures.
//!
//! This module provides the canonical MCP protocol types used across the
//! entire workspace. All crates MUST import these from `mcb_domain::protocol`
//! instead of defining their own.
//!
//! **Documentation**: [docs/modules/domain.md](../../../docs/modules/domain.md)

/// MCP JSON-RPC protocol types (request, response, error).
pub mod mcp_types;

pub use mcp_types::{JSONRPC_VERSION, McpError, McpRequest, McpResponse};
