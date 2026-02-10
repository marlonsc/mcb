use mcb_domain::error::Error;
use rmcp::model::{CallToolResult, Content, ErrorData as McpError};

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

pub fn to_opaque_tool_error(e: impl std::fmt::Display) -> CallToolResult {
    tracing::error!(error = %e, "tool operation failed");
    CallToolResult::error(vec![Content::text("internal error")])
}
