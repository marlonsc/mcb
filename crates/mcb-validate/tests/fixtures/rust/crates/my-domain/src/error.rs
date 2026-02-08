//! Domain error types.
//!
//! This module defines the domain's own error hierarchy, wrapping infrastructure
//! errors into domain-specific variants. Files named `error.rs` are exempt from
//! layer-error-type checks because wrapping is their job.

use thiserror::Error;

/// Domain-specific error type.
///
/// Wraps infrastructure errors into domain-meaningful variants.
/// This is the CORRECT place for From<std::io::Error> etc.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("External service error: {0}")]
    External(String),
}

/// Converts a reqwest error into a domain error.
///
/// This is CORRECT — error.rs is the right place to handle
/// infrastructure → domain error conversion.
impl From<reqwest::Error> for DomainError {
    fn from(e: reqwest::Error) -> Self {
        DomainError::External(e.to_string())
    }
}
