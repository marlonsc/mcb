//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use mcb_domain::error::Error;

fn format_error(label: &str, detail: impl std::fmt::Display) -> String {
    format!("{label}: {detail}")
}

fn log_and_format_error(log: &str, label: &str, detail: impl std::fmt::Display) -> String {
    tracing::error!(category = %label, "{log}");
    format_error(label, detail)
}

fn log_and_static_error(log: &str, message: &str) -> String {
    tracing::error!("{log}");
    message.to_owned()
}

fn map_client_error(error: &Error) -> Option<String> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let message = match error {
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
        _ => {
            tracing::trace!(mapper = "client", "skipped unmatched variant");
            return None;
        }
    };
    Some(message)
}

fn map_provider_error(error: &Error) -> Option<String> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let message = match error {
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
        _ => {
            tracing::trace!(mapper = "provider", "skipped unmatched variant");
            return None;
        }
    };
    Some(message)
}

fn map_config_error(error: &Error) -> Option<String> {
    if let Error::Config { message } | Error::Configuration { message, .. } = error {
        return Some(format_error("Configuration error", message));
    }
    if let Error::ConfigMissing(field) = error {
        return Some(format!("Missing configuration: {field}"));
    }
    if let Error::ConfigInvalid { key, message } = error {
        return Some(format!("Invalid configuration for '{key}': {message}"));
    }
    if let Error::Authentication { message, .. } = error {
        return Some(format_error("Authentication error", message));
    }
    tracing::trace!(mapper = "config", "skipped unmatched variant");
    None
}

fn map_system_error(error: &Error) -> Option<String> {
    if let Error::Cache { message } = error {
        return Some(log_and_format_error(
            "cache operation failed",
            "Cache error",
            message,
        ));
    }
    if let Error::Infrastructure { message, .. } = error {
        return Some(log_and_format_error(
            "infrastructure error",
            "Infrastructure error",
            message,
        ));
    }
    if let Error::Internal { message } = error {
        return Some(log_and_format_error(
            "internal error",
            "Internal error",
            message,
        ));
    }
    tracing::trace!("map_system_error skipped variant");
    None
}

fn map_encoding_error(error: &Error) -> Option<String> {
    if let Error::Json { source } = error {
        return Some(log_and_format_error(
            "JSON processing failed",
            "JSON error",
            source,
        ));
    }
    if let Error::Utf8(_) = error {
        return Some(log_and_static_error(
            "encoding error",
            "Encoding error: invalid UTF-8",
        ));
    }
    if let Error::Base64(_) = error {
        return Some(log_and_static_error(
            "encoding error",
            "Encoding error: invalid base64",
        ));
    }
    tracing::trace!("map_encoding_error skipped variant");
    None
}

fn map_io_error(error: &Error) -> Option<String> {
    if let Error::IoSimple { source } = error {
        return Some(log_and_format_error(
            "I/O operation failed",
            "I/O error",
            source.kind(),
        ));
    }
    if let Error::Io { message, .. } = error {
        return Some(log_and_format_error(
            "I/O operation failed",
            "I/O error",
            message,
        ));
    }
    tracing::trace!("map_io_error skipped variant");
    None
}

fn map_generic_error(error: &Error) -> Option<String> {
    if let Error::Generic(e) = error {
        return Some(log_and_format_error(
            "operation failed",
            "Operation failed",
            e,
        ));
    }
    if let Error::Browse(e) = error {
        return Some(log_and_format_error(
            "browse operation failed",
            "Browse error",
            e,
        ));
    }
    if let Error::Highlight(e) = error {
        return Some(log_and_format_error(
            "highlight operation failed",
            "Highlight error",
            e,
        ));
    }
    tracing::trace!("map_generic_error skipped variant");
    None
}

pub(super) fn map_error_message(error: &Error) -> Option<String> {
    map_client_error(error)
        .or_else(|| map_provider_error(error))
        .or_else(|| map_config_error(error))
        .or_else(|| map_system_error(error))
        .or_else(|| map_encoding_error(error))
        .or_else(|| map_io_error(error))
        .or_else(|| map_generic_error(error))
}
