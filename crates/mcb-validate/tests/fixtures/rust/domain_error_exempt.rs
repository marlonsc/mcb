// Domain error definition module — this file should be EXEMPT from
// layer error type checks because error.rs is where domain errors
// are defined and infra errors are wrapped into domain-specific types.
//
// NOTE: This file must be named `error.rs` or `error_*.rs` to trigger
// the exemption logic in `validate_layer_error_types()`.

use std::fmt;

/// Domain error type that properly wraps infrastructure errors.
/// This is the correct Clean Architecture pattern: the domain defines
/// its own error enum and wraps lower-level errors behind domain semantics.
#[derive(Debug)]
pub enum DomainError {
    /// Wraps IO errors for persistence operations
    Io(std::io::Error),

    /// Wraps database errors — referencing sqlx::Error here is intentional
    /// because this is the *mapping* layer between infra and domain.
    Database(sqlx::Error),

    /// Wraps serialization errors for data format issues
    Serialization(serde_json::Error),

    /// Wraps HTTP errors for external service communication
    Http(reqwest::Error),

    /// Wraps hyper errors for low-level HTTP transport
    Transport(hyper::Error),

    /// Pure domain validation error (no infra dependency)
    Validation(String),

    /// Entity not found
    NotFound { entity: String, id: String },

    /// Business rule violation
    BusinessRule { rule: String, message: String },
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Database(e) => write!(f, "Database error: {e}"),
            Self::Serialization(e) => write!(f, "Serialization error: {e}"),
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Transport(e) => write!(f, "Transport error: {e}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::NotFound { entity, id } => write!(f, "{entity} not found: {id}"),
            Self::BusinessRule { rule, message } => {
                write!(f, "Business rule '{rule}' violated: {message}")
            }
        }
    }
}

impl From<std::io::Error> for DomainError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for DomainError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}
