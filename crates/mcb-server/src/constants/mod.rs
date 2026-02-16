//! Server-specific constants
//!
//! Contains constants specific to the MCP server implementation,
//! including JSON-RPC error codes, file names, and protocol-related values.

/// Display formatting and presentation constants.
pub mod display;
/// Default limits and thresholds for MCP tool handlers and admin endpoints.
pub mod limits;
/// VCS impact analysis constants.
pub mod vcs;

// ============================================================================
// JSON-RPC ERROR CODES (Standard)
// ============================================================================

/// JSON-RPC Method not found error code
pub const JSONRPC_METHOD_NOT_FOUND: i32 = -32601;

/// JSON-RPC Parse error code
pub const JSONRPC_PARSE_ERROR: i32 = -32700;

/// JSON-RPC Invalid request error code
pub const JSONRPC_INVALID_REQUEST: i32 = -32600;

/// JSON-RPC Invalid params error code
pub const JSONRPC_INVALID_PARAMS: i32 = -32602;

/// JSON-RPC Internal error code
pub const JSONRPC_INTERNAL_ERROR: i32 = -32603;

// ============================================================================
// BROWSE / ADMIN
// ============================================================================

/// Maximum number of file paths to return when building collection tree
pub const LIST_FILE_PATHS_LIMIT: usize = 10_000;

/// Valid configuration sections for admin config updates
pub const VALID_SECTIONS: &[&str] = &[
    "server",
    "logging",
    "cache",
    "metrics",
    "limits",
    "resilience",
];

// HIGHLIGHT_NAMES lives in mcb_infrastructure::constants::highlight (Single Source of Truth)
