mod groups;

use mcb_domain::error::Error;
use rmcp::model::{CallToolResult, Content, ErrorData as McpError};

/// Maps a domain error to an opaque MCP error.
///
/// # Behavior
/// Returns client-fixable errors for `NotFound` and `InvalidArgument`; all
/// other variants are converted to a generic internal error.
///
/// # Security
/// Never exposes provider/internal details to external callers.
pub fn to_opaque_mcp_error(e: Error) -> McpError {
    match &e {
        Error::NotFound { .. } | Error::InvalidArgument { .. } => {
            McpError::invalid_params(e.to_string(), None)
        }
        _ => {
            tracing::error!("operation failed");
            McpError::internal_error("internal server error", None)
        }
    }
}

/// Builds a contextual MCP tool error response from a domain error.
///
/// # Behavior
/// Applies categorized mappers (client, provider, config, system, encoding,
/// IO, generic) and returns the first matching sanitized message.
///
/// # Security
/// Logs internal details server-side and returns safe text only.
pub fn to_contextual_tool_error(e: impl Into<Error>) -> CallToolResult {
    let error: Error = e.into();
    let message = groups::map_error_message(&error).unwrap_or_else(|| {
        tracing::error!("unmapped error variant: {error}");
        "Internal error".to_string()
    });
    CallToolResult::error(vec![Content::text(message)])
}
