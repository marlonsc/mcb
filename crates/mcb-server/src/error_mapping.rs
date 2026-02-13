use mcb_domain::error::Error;
// TODO(REF003): Missing test file for crates/mcb-server/src/error_mapping.rs.
// Tracking: REF003

use rmcp::model::{CallToolResult, Content, ErrorData as McpError};

/// Maps domain-specific errors to sanitized, opaque MCP errors.
///
/// This function ensures that internal details, such as database schema issues
/// or stack traces, are not leaked to external clients. It provides a clean,
/// user-friendly error message for common fixable errors and a generic
/// internal error for all others.
///
/// # Arguments
///
/// * `e` - The [`Error`] from the domain layer to be mapped.
///
/// # Returns
///
/// A sanitized [`McpError`] ready for transmission.
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

/// Builds a contextual tool-call error response with categorization and sanitization.
///
/// This function maps internal domain errors into a structured [`CallToolResult`]
/// that can be returned by the MCP server tools. It categorizes errors into
/// client-fixable (e.g., Not Found) and provider-specific issues (e.g., Database),
/// while ensuring that no sensitive infrastructure information is exposed.
///
/// # Arguments
///
/// * `e` - An object that can be converted into a domain [`Error`].
///
/// # Returns
///
/// A [`CallToolResult`] containing the categorized and sanitized error message.
///
/// # Examples
///
/// ```rust
/// let err = Error::NotFound { resource: "repo".to_string() };
/// let result = to_contextual_tool_error(err);
/// assert!(result.is_error);
/// ```
// TODO(KISS005): Function exceeds recommended line count (107 lines, max 50).
// Decompose into smaller category-based mapping functions.
pub fn to_contextual_tool_error(e: impl Into<Error>) -> CallToolResult {
    let error: Error = e.into();
    // TODO(SOLID002): Match block contains 29 arms (max 15).
    // Use visitor pattern or delegate mapping to maintain OCP.
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
