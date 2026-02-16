use mcb_domain::error::Error;

fn format_error(label: &str, detail: impl std::fmt::Display) -> String {
    format!("{label}: {detail}")
}

fn log_and_format_error(log: &str, label: &str, detail: impl std::fmt::Display) -> String {
    tracing::error!(category = label, detail = %detail, "{log}");
    format_error(label, detail)
}

fn log_and_static_error(log: &str, message: &str) -> String {
    tracing::error!(detail = message, "{log}");
    message.to_owned()
}

fn map_client_error(error: &Error) -> Option<String> {
    if let Error::NotFound { resource } = error {
        return Some(format!("Not found: {resource}"));
    }
    if let Error::InvalidArgument { message } = error {
        return Some(format!("Invalid argument: {message}"));
    }
    if let Error::ObservationNotFound { id } = error {
        return Some(format!("Observation not found: {id}"));
    }
    if let Error::DuplicateObservation { content_hash } = error {
        return Some(format!("Duplicate observation: {content_hash}"));
    }
    if let Error::RepositoryNotFound { path } = error {
        return Some(format!("Repository not found: {path}"));
    }
    if let Error::BranchNotFound { name } = error {
        return Some(format!("Branch not found: {name}"));
    }
    if let Error::InvalidRegex { pattern, message } = error {
        return Some(format!("Invalid regex pattern '{pattern}': {message}"));
    }
    tracing::trace!("map_client_error skipped variant: {error}");
    None
}

fn map_provider_error(error: &Error) -> Option<String> {
    if let Error::Database { message, .. } = error {
        return Some(log_and_format_error(
            "database operation failed",
            "Database error",
            message,
        ));
    }
    if let Error::VectorDb { message } = error {
        return Some(log_and_format_error(
            "vector database operation failed",
            "Vector database error",
            message,
        ));
    }
    if let Error::Embedding { message } = error {
        return Some(log_and_format_error(
            "embedding operation failed",
            "Embedding error",
            message,
        ));
    }
    if let Error::Network { message, .. } = error {
        return Some(log_and_format_error(
            "network operation failed",
            "Network error",
            message,
        ));
    }
    if let Error::ObservationStorage { message, .. } = error {
        return Some(log_and_format_error(
            "observation storage failed",
            "Memory storage error",
            message,
        ));
    }
    if let Error::Vcs { message, .. } = error {
        return Some(log_and_format_error(
            "VCS operation failed",
            "VCS error",
            message,
        ));
    }
    tracing::trace!("map_provider_error skipped variant: {error}");
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
    tracing::trace!("map_config_error skipped variant: {error}");
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
    tracing::trace!("map_system_error skipped variant: {error}");
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
    tracing::trace!("map_encoding_error skipped variant: {error}");
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
    tracing::trace!("map_io_error skipped variant: {error}");
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
    tracing::trace!("map_generic_error skipped variant: {error}");
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
