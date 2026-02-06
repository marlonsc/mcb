//! Server-specific constants
//!
//! Contains constants specific to the MCP server implementation,
//! including JSON-RPC error codes, file names, and protocol-related values.

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

// ============================================================================
// VCS REGISTRY
// ============================================================================

/// VCS repository registry file name (stored under ~/.config/mcb)
pub const VCS_REGISTRY_FILENAME: &str = "vcs_repository_registry.json";

/// VCS registry lock file name for serialized writes
pub const VCS_LOCK_FILENAME: &str = "vcs_repository_registry.lock";

// ============================================================================
// COLLECTION MAPPING
// ============================================================================

/// Collection name mapping file name (~/.config/mcb)
pub const COLLECTION_MAPPING_FILENAME: &str = "collection_mapping.json";

/// Collection mapping lock file for atomic operations
pub const COLLECTION_MAPPING_LOCK_FILENAME: &str = "collection_mapping.lock";

// ============================================================================
// HIGHLIGHT SERVICE
// ============================================================================

/// Tree-sitter highlight capture names (order must match HighlightConfiguration)
pub const HIGHLIGHT_NAMES: [&str; 13] = [
    "keyword",
    "function",
    "string",
    "comment",
    "type",
    "variable",
    "constant",
    "operator",
    "attribute",
    "number",
    "punctuation",
    "property",
    "tag",
];
