//! Server-specific constants
//!
//! Contains constants specific to the MCP server implementation,
//! including JSON-RPC error codes, file names, and protocol-related values.

/// Admin constants
pub mod admin_config;
/// Display formatting and presentation constants.
pub mod display;
/// JSON response field name constants.
pub mod fields;
/// Git reference constants for VCS handlers.
pub mod git;
/// JSON-RPC error codes
pub mod json_rpc;
/// Default limits and thresholds for MCP tool handlers and admin endpoints.
pub mod limits;
/// MCP and JSON-RPC protocol constants.
pub mod protocol;
/// MCP tool name constants.
pub mod tools;
/// VCS impact analysis constants.
pub mod vcs;

pub use admin_config::*;
pub use json_rpc::*;
