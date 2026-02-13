use mcb_domain::error::Error;
// TODO(REF003): Missing test file for crates/mcb-server/src/error_mapping.rs.
// Tracking: REF003

use rmcp::model::{CallToolResult, Content, ErrorData as McpError};

fn format_error(label: &str, detail: impl std::fmt::Display) -> String {
    format!("{label}: {detail}")
}

fn log_and_format_error(log: &str, label: &str, detail: impl std::fmt::Display) -> String {
    tracing::error!("{log}");
    format_error(label, detail)
}

fn log_and_static_error(log: &str, message: &str) -> String {
    tracing::error!("{log}");
    message.to_string()
}

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
/// use mcb_domain::error::Error;
/// use mcb_server::error_mapping::to_contextual_tool_error;
///
/// let err = Error::NotFound { resource: "repo".to_string() };
/// let result = to_contextual_tool_error(err);
/// assert_eq!(result.is_error, Some(true));
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
            log_and_format_error("database operation failed", "Database error", message)
        }
        Error::VectorDb { message } => log_and_format_error(
            "vector database operation failed",
            "Vector database error",
            message,
        ),
        Error::Embedding { message } => {
            log_and_format_error("embedding operation failed", "Embedding error", message)
        }
        Error::Network { message, .. } => {
            log_and_format_error("network operation failed", "Network error", message)
        }
        Error::ObservationStorage { message, .. } => log_and_format_error(
            "observation storage failed",
            "Memory storage error",
            message,
        ),
        Error::Vcs { message, .. } => {
            log_and_format_error("VCS operation failed", "VCS error", message)
        }

        // Config errors — category + reason
        Error::Config { message } | Error::Configuration { message, .. } => {
            format_error("Configuration error", message)
        }
        Error::ConfigMissing(field) => format!("Missing configuration: {field}"),
        Error::ConfigInvalid { key, message } => {
            format!("Invalid configuration for '{key}': {message}")
        }
        Error::Authentication { message, .. } => format_error("Authentication error", message),

        // Infrastructure/system errors — log full details, return category
        Error::Cache { message } => {
            log_and_format_error("cache operation failed", "Cache error", message)
        }
        Error::Infrastructure { message, .. } => {
            log_and_format_error("infrastructure error", "Infrastructure error", message)
        }
        Error::Internal { message } => {
            log_and_format_error("internal error", "Internal error", message)
        }

        // Serialization/encoding errors — sanitized
        Error::Json { source } => {
            log_and_format_error("JSON processing failed", "JSON error", source)
        }
        Error::Utf8(_) => log_and_static_error("encoding error", "Encoding error: invalid UTF-8"),
        Error::Base64(_) => {
            log_and_static_error("encoding error", "Encoding error: invalid base64")
        }

        // I/O errors — sanitized (no file paths leaked)
        Error::IoSimple { source } => {
            log_and_format_error("I/O operation failed", "I/O error", source.kind())
        }
        Error::Io { message, .. } => {
            log_and_format_error("I/O operation failed", "I/O error", message)
        }

        // Generic / Browse / Highlight — log and sanitize
        Error::Generic(e) => log_and_format_error("operation failed", "Operation failed", e),
        Error::Browse(e) => log_and_format_error("browse operation failed", "Browse error", e),
        Error::Highlight(e) => {
            log_and_format_error("highlight operation failed", "Highlight error", e)
        }
    };
    CallToolResult::error(vec![Content::text(message)])
}
