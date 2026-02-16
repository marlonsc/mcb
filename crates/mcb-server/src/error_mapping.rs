mod groups;

use mcb_domain::error::Error;
use rmcp::model::{CallToolResult, Content, ErrorData as McpError};

/// Logs the underlying error server-side and returns a generic internal error.
///
/// Use this instead of `McpError::internal_error(e.to_string(), None)` so that
/// underlying error details are never leaked to MCP clients.
pub fn safe_internal_error(context: &str, error: &dyn std::fmt::Display) -> McpError {
    tracing::error!(context = context, error = %error, "internal operation failed");
    McpError::internal_error("internal server error", None)
}

/// Maps a domain error to an opaque MCP error.
///
/// # Behavior
/// Returns client-fixable errors for `NotFound` and `InvalidArgument`; all
/// other variants are converted to a generic internal error.
///
/// # Security
/// Never exposes provider/internal details to external callers.
pub fn to_opaque_mcp_error(e: &Error) -> McpError {
    if matches!(e, Error::NotFound { .. } | Error::InvalidArgument { .. }) {
        McpError::invalid_params(e.to_string(), None)
    } else {
        tracing::error!(error = %e, "operation failed");
        McpError::internal_error("internal server error", None)
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
        tracing::error!(error = %error, "unmapped error variant");
        "Internal error".to_owned()
    });
    CallToolResult::error(vec![Content::text(message)])
}
