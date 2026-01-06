//! Error handling types

use thiserror::Error;

/// Result type alias for operations that can fail
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the MCP Context Browser
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("JSON parsing error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    #[error("Generic error: {message}")]
    Generic { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String },

    #[error("Vector database error: {message}")]
    VectorDb { message: String },

    #[error("Embedding provider error: {message}")]
    Embedding { message: String },
}

impl Error {
    /// Create a generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create an invalid argument error
    pub fn invalid_argument<S: Into<String>>(message: S) -> Self {
        Self::InvalidArgument {
            message: message.into(),
        }
    }

    /// Create a vector database error
    pub fn vector_db<S: Into<String>>(message: S) -> Self {
        Self::VectorDb {
            message: message.into(),
        }
    }

    /// Create an embedding provider error
    pub fn embedding<S: Into<String>>(message: S) -> Self {
        Self::Embedding {
            message: message.into(),
        }
    }
}
