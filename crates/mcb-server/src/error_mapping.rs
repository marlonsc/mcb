use mcb_domain::error::Error;
use rmcp::model::{CallToolResult, Content, ErrorData as McpError};

/// Maps domain errors to sanitized MCP errors suitable for external clients.
pub fn to_opaque_mcp_error(e: Error) -> McpError {
    match &e {
        Error::NotFound { .. } => McpError::invalid_params(e.to_string(), None),
        Error::InvalidArgument { .. } => McpError::invalid_params(e.to_string(), None),
        other => {
            tracing::error!(error = %other, "operation failed");
            McpError::internal_error("internal server error", None)
        }
    }
}

/// Builds an opaque tool-call error response without leaking internals.
pub fn to_opaque_tool_error(e: impl std::fmt::Display) -> CallToolResult {
    tracing::error!(error = %e, "tool operation failed");
    CallToolResult::error(vec![Content::text("internal error")])
}
