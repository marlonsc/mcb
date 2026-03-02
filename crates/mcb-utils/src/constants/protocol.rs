//! MCP and JSON-RPC protocol constants.

/// JSON-RPC Parse error code.
pub const JSONRPC_PARSE_ERROR: i32 = -32700;

/// JSON-RPC Internal error code.
pub const JSONRPC_INTERNAL_ERROR: i32 = -32603;

/// JSON-RPC protocol version string.
pub const JSONRPC_VERSION: &str = "2.0";

/// HTTP MCP endpoint path.
pub const MCP_ENDPOINT_PATH: &str = "/mcp";

/// Custom HTTP header for execution flow mode.
pub const HTTP_HEADER_EXECUTION_FLOW: &str = "X-Execution-Flow";

/// Execution flow mode: client-hybrid.
pub const EXECUTION_FLOW_HYBRID: &str = "client-hybrid";
