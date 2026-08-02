//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use mcb_domain::error::Error;
use mcb_domain::{error, trace};

fn format_error(label: &str, detail: impl std::fmt::Display) -> String {
    format!("{label}: {detail}")
}

fn map_client_error(error: &Error) -> Option<String> {
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
        Error::IoSimple { .. }
        | Error::Io { .. }
        | Error::Json { .. }
        | Error::Generic(_)
        | Error::Utf8(_)
        | Error::Base64(_)
        | Error::VectorDb { .. }
        | Error::Embedding { .. }
        | Error::Config { .. }
        | Error::Configuration { .. }
        | Error::ConfigMissing(_)
        | Error::ConfigInvalid { .. }
        | Error::Authentication { .. }
        | Error::Network { .. }
        | Error::Database { .. }
        | Error::Internal { .. }
        | Error::Cache { .. }
        | Error::Infrastructure { .. }
        | Error::Vcs { .. }
        | Error::ObservationStorage { .. }
        | Error::Browse(_)
        | Error::Highlight(_) => {
            trace!("ErrorMapping", "client mapper skipped unmatched variant");
            return None;
        }
    };
    Some(message)
}

fn map_provider_error(error: &Error) -> Option<String> {
    if let Error::Database { message, .. } = error {
        error!("Database", "database operation failed", message);
        return Some(format_error("Database error", message));
    }
    if let Error::VectorDb { message } = error {
        error!("VectorDb", "vector database operation failed", message);
        return Some(format_error("Vector database error", message));
    }
    if let Error::Embedding { message } = error {
        error!("Embedding", "embedding operation failed", message);
        return Some(format_error("Embedding error", message));
    }
    if let Error::Network { message, .. } = error {
        error!("Network", "network operation failed", message);
        return Some(format_error("Network error", message));
    }
    if let Error::ObservationStorage { message, .. } = error {
        error!("Memory", "observation storage failed", message);
        return Some(format_error("Memory storage error", message));
    }
    if let Error::Vcs { message, .. } = error {
        error!("Vcs", "VCS operation failed", message);
        return Some(format_error("VCS error", message));
    }
    trace!("ErrorMapping", "provider mapper skipped unmatched variant");
    None
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
    trace!("ErrorMapping", "config mapper skipped unmatched variant");
    None
}

fn map_system_error(error: &Error) -> Option<String> {
    if let Error::Cache { message } = error {
        error!("Cache", "cache operation failed", message);
        return Some(format_error("Cache error", message));
    }
    if let Error::Infrastructure { message, .. } = error {
        error!("Infrastructure", "infrastructure error", message);
        return Some(format_error("Infrastructure error", message));
    }
    if let Error::Internal { message } = error {
        error!("Internal", "internal error", message);
        return Some(format_error("Internal error", message));
    }
    trace!("ErrorMapping", "map_system_error skipped variant");
    None
}

fn map_encoding_error(error: &Error) -> Option<String> {
    if let Error::Json { source } = error {
        error!("Json", "JSON processing failed", source);
        return Some(format_error("JSON error", source));
    }
    if let Error::Utf8(_) = error {
        error!("ErrorMapping", "Encoding error: invalid UTF-8");
        return Some("Encoding error: invalid UTF-8".to_owned());
    }
    if let Error::Base64(_) = error {
        error!("ErrorMapping", "Encoding error: invalid base64");
        return Some("Encoding error: invalid base64".to_owned());
    }
    trace!("ErrorMapping", "map_encoding_error skipped variant");
    None
}

fn map_io_error(error: &Error) -> Option<String> {
    if let Error::IoSimple { source } = error {
        let kind = source.kind();
        error!("Io", "I/O operation failed", &kind);
        return Some(format_error("I/O error", kind));
    }
    if let Error::Io { message, .. } = error {
        error!("Io", "I/O operation failed", message);
        return Some(format_error("I/O error", message));
    }
    trace!("ErrorMapping", "map_io_error skipped variant");
    None
}

fn map_generic_error(error: &Error) -> Option<String> {
    if let Error::Generic(e) = error {
        error!("Generic", "operation failed", e);
        return Some(format_error("Operation failed", e));
    }
    if let Error::Browse(e) = error {
        error!("Browse", "browse operation failed", e);
        return Some(format_error("Browse error", e));
    }
    if let Error::Highlight(e) = error {
        error!("Highlight", "highlight operation failed", e);
        return Some(format_error("Highlight error", e));
    }
    trace!("ErrorMapping", "map_generic_error skipped variant");
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
