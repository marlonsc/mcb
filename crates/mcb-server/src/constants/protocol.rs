//! MCP and JSON-RPC protocol constants.

/// JSON-RPC protocol version string.
pub const JSONRPC_VERSION: &str = "2.0";

/// HTTP Content-Type for JSON responses (re-exported from domain).
pub use mcb_domain::constants::http::CONTENT_TYPE_JSON;

/// HTTP MCP endpoint path.
pub const MCP_ENDPOINT_PATH: &str = "/mcp";

/// Custom HTTP header for execution flow mode.
pub const HTTP_HEADER_EXECUTION_FLOW: &str = "X-Execution-Flow";

/// Execution flow mode: client-hybrid.
pub const EXECUTION_FLOW_HYBRID: &str = "client-hybrid";
