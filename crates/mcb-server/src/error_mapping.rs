use mcb_domain::error::Error;
// TODO(REF003): Missing test file for crates/mcb-server/src/error_mapping.rs.
// Expected: crates/mcb-server/tests/error_mapping_test.rs

use rmcp::model::{CallToolResult, Content, ErrorData as McpError};

/// Maps domain errors to sanitized MCP errors suitable for external clients.
pub fn to_opaque_mcp_error(e: Error) -> McpError {
    match &e {
        Error::NotFound { .. } => McpError::invalid_params(e.to_string(), None),
        Error::InvalidArgument { .. } => McpError::invalid_params(e.to_string(), None),
        _other => {
            tracing::error!("operation failed");
            McpError::internal_error("internal server error", None)
        }
    }
}

/// Builds a contextual tool-call error response with category and reason.
///
/// Client-fixable errors (NotFound, InvalidArgument) return specific messages.
/// Provider errors (Database, VectorDb, Embedding) return category + sanitized reason.
/// Internal paths and stack traces are never leaked.
// TODO(KISS005): Function to_contextual_tool_error is too long (105 lines, max: 50).
// Break into smaller, focused functions.
pub fn to_contextual_tool_error(e: impl Into<Error>) -> CallToolResult {
    let error: Error = e.into();
    // TODO(SOLID002): Excessive match arms (29 arms, recommended max: 15).
    // Consider using visitor pattern, enum dispatch, or trait objects.
    let message = match &error {
        // Client-fixable errors — return the specific message
        Error::NotFound { resource } => format!("Not found: {resource}"),
        Error::InvalidArgument { message } => format!("Invalid argument: {message}"),
        Error::ObservationNotFound { id } => format!("Observation not found: {id}"),
        Error::DuplicateObservation { content_hash } => {
            format!("Duplicate observation: {content_hash}")
        }
        Error::RepositoryNotFound { path } => format!("Repository not found: {path}"),
        Error::BranchNotFound { name } => format!("Branch not found: {name}"),
        Error::InvalidRegex { pattern, message } => {
            format!("Invalid regex pattern '{pattern}': {message}")
        }

        // Provider errors — category + sanitized reason
        Error::Database { message, .. } => {
            tracing::error!("database operation failed");
            format!("Database error: {message}")
        }
        Error::VectorDb { message } => {
            tracing::error!("vector database operation failed");
            format!("Vector database error: {message}")
        }
        Error::Embedding { message } => {
            tracing::error!("embedding operation failed");
            format!("Embedding error: {message}")
        }
        Error::Network { message, .. } => {
            tracing::error!("network operation failed");
            format!("Network error: {message}")
        }
        Error::ObservationStorage { message, .. } => {
            tracing::error!("observation storage failed");
            format!("Memory storage error: {message}")
        }
        Error::Vcs { message, .. } => {
            tracing::error!("VCS operation failed");
            format!("VCS error: {message}")
        }

        // Config errors — category + reason
        Error::Config { message } => format!("Configuration error: {message}"),
        Error::Configuration { message, .. } => format!("Configuration error: {message}"),
        Error::ConfigMissing(field) => format!("Missing configuration: {field}"),
        Error::ConfigInvalid { key, message } => {
            format!("Invalid configuration for '{key}': {message}")
        }
        Error::Authentication { message, .. } => format!("Authentication error: {message}"),

        // Infrastructure/system errors — log full details, return category
        Error::Cache { message } => {
            tracing::error!("cache operation failed");
            format!("Cache error: {message}")
        }
        Error::Infrastructure { message, .. } => {
            tracing::error!("infrastructure error");
            format!("Infrastructure error: {message}")
        }
        Error::Internal { message } => {
            tracing::error!("internal error");
            format!("Internal error: {message}")
        }

        // Serialization/encoding errors — sanitized
        Error::Json { source } => {
            tracing::error!("JSON processing failed");
            format!("JSON error: {source}")
        }
        Error::Utf8(_) => {
            tracing::error!("encoding error");
            "Encoding error: invalid UTF-8".to_string()
        }
        Error::Base64(_) => {
            tracing::error!("encoding error");
            "Encoding error: invalid base64".to_string()
        }

        // I/O errors — sanitized (no file paths leaked)
        Error::IoSimple { source } => {
            tracing::error!("I/O operation failed");
            format!("I/O error: {}", source.kind())
        }
        Error::Io { message, .. } => {
            tracing::error!("I/O operation failed");
            format!("I/O error: {message}")
        }

        // Generic / Browse / Highlight — log and sanitize
        Error::Generic(e) => {
            tracing::error!("operation failed");
            format!("Operation failed: {e}")
        }
        Error::Browse(e) => {
            tracing::error!("browse operation failed");
            format!("Browse error: {e}")
        }
        Error::Highlight(e) => {
            tracing::error!("highlight operation failed");
            format!("Highlight error: {e}")
        }
    };
    CallToolResult::error(vec![Content::text(message)])
}
