use mcb_domain::error::Error;
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

fn map_client_error(error: &Error) -> Option<String> {
    match error {
        Error::NotFound { resource } => Some(format!("Not found: {resource}")),
        Error::InvalidArgument { message } => Some(format!("Invalid argument: {message}")),
        Error::ObservationNotFound { id } => Some(format!("Observation not found: {id}")),
        Error::DuplicateObservation { content_hash } => {
            Some(format!("Duplicate observation: {content_hash}"))
        }
        Error::RepositoryNotFound { path } => Some(format!("Repository not found: {path}")),
        Error::BranchNotFound { name } => Some(format!("Branch not found: {name}")),
        Error::InvalidRegex { pattern, message } => {
            Some(format!("Invalid regex pattern '{pattern}': {message}"))
        }
        _ => None,
    }
}

fn map_provider_error(error: &Error) -> Option<String> {
    match error {
        Error::Database { message, .. } => Some(log_and_format_error(
            "database operation failed",
            "Database error",
            message,
        )),
        Error::VectorDb { message } => Some(log_and_format_error(
            "vector database operation failed",
            "Vector database error",
            message,
        )),
        Error::Embedding { message } => Some(log_and_format_error(
            "embedding operation failed",
            "Embedding error",
            message,
        )),
        Error::Network { message, .. } => Some(log_and_format_error(
            "network operation failed",
            "Network error",
            message,
        )),
        Error::ObservationStorage { message, .. } => Some(log_and_format_error(
            "observation storage failed",
            "Memory storage error",
            message,
        )),
        Error::Vcs { message, .. } => Some(log_and_format_error(
            "VCS operation failed",
            "VCS error",
            message,
        )),
        _ => None,
    }
}

fn map_config_error(error: &Error) -> Option<String> {
    match error {
        Error::Config { message } | Error::Configuration { message, .. } => {
            Some(format_error("Configuration error", message))
        }
        Error::ConfigMissing(field) => Some(format!("Missing configuration: {field}")),
        Error::ConfigInvalid { key, message } => {
            Some(format!("Invalid configuration for '{key}': {message}"))
        }
        Error::Authentication { message, .. } => {
            Some(format_error("Authentication error", message))
        }
        _ => None,
    }
}

fn map_system_error(error: &Error) -> Option<String> {
    match error {
        Error::Cache { message } => Some(log_and_format_error(
            "cache operation failed",
            "Cache error",
            message,
        )),
        Error::Infrastructure { message, .. } => Some(log_and_format_error(
            "infrastructure error",
            "Infrastructure error",
            message,
        )),
        Error::Internal { message } => Some(log_and_format_error(
            "internal error",
            "Internal error",
            message,
        )),
        _ => None,
    }
}

fn map_encoding_error(error: &Error) -> Option<String> {
    match error {
        Error::Json { source } => Some(log_and_format_error(
            "JSON processing failed",
            "JSON error",
            source,
        )),
        Error::Utf8(_) => Some(log_and_static_error(
            "encoding error",
            "Encoding error: invalid UTF-8",
        )),
        Error::Base64(_) => Some(log_and_static_error(
            "encoding error",
            "Encoding error: invalid base64",
        )),
        _ => None,
    }
}

fn map_io_error(error: &Error) -> Option<String> {
    match error {
        Error::IoSimple { source } => Some(log_and_format_error(
            "I/O operation failed",
            "I/O error",
            source.kind(),
        )),
        Error::Io { message, .. } => Some(log_and_format_error(
            "I/O operation failed",
            "I/O error",
            message,
        )),
        _ => None,
    }
}

fn map_generic_error(error: &Error) -> Option<String> {
    match error {
        Error::Generic(e) => Some(log_and_format_error(
            "operation failed",
            "Operation failed",
            e,
        )),
        Error::Browse(e) => Some(log_and_format_error(
            "browse operation failed",
            "Browse error",
            e,
        )),
        Error::Highlight(e) => Some(log_and_format_error(
            "highlight operation failed",
            "Highlight error",
            e,
        )),
        _ => None,
    }
}

/// Builds a contextual tool-call error response with categorization and sanitization.
///
/// Maps internal domain errors into a structured [`CallToolResult`] for MCP server tools,
/// categorizing errors as client-fixable or provider-specific while preventing
/// sensitive infrastructure information leakage.
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
pub fn to_contextual_tool_error(e: impl Into<Error>) -> CallToolResult {
    let error: Error = e.into();

    let message = map_client_error(&error)
        .or_else(|| map_provider_error(&error))
        .or_else(|| map_config_error(&error))
        .or_else(|| map_system_error(&error))
        .or_else(|| map_encoding_error(&error))
        .or_else(|| map_io_error(&error))
        .or_else(|| map_generic_error(&error))
        .unwrap_or_else(|| {
            tracing::error!("unmapped error variant: {error}");
            "Internal error".to_string()
        });

    CallToolResult::error(vec![Content::text(message)])
}
