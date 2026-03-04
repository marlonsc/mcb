//! MCP JSON-RPC protocol types (domain-level)
//!
//! These are pure data types for the JSON-RPC-based MCP protocol.
//! They live in the domain layer because they represent the protocol
//! contract — no transport logic, no HTTP dependencies.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

use serde::{Deserialize, Serialize};

/// JSON-RPC version constant.
pub const JSONRPC_VERSION: &str = "2.0";

/// MCP request payload (JSON-RPC format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    /// JSON-RPC method
    pub method: String,
    /// Request parameters
    pub params: Option<serde_json::Value>,
    /// Request ID
    pub id: Option<serde_json::Value>,
}

/// MCP response payload (JSON-RPC format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// JSON-RPC version
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,
    /// Response result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
    /// Request ID
    pub id: Option<serde_json::Value>,
}

fn default_jsonrpc() -> String {
    JSONRPC_VERSION.to_owned()
}

/// MCP error response (JSON-RPC format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
}

impl McpResponse {
    /// Create a success response.
    #[must_use]
    pub fn from_success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response.
    pub fn from_error(
        id: Option<serde_json::Value>,
        code: i32,
        message: impl Into<String>,
    ) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            result: None,
            error: Some(McpError {
                code,
                message: message.into(),
            }),
            id,
        }
    }
}
