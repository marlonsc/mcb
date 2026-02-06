//! Error handling types

use thiserror::Error;

/// Result type alias for operations that can fail
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the MCP Context Browser
#[derive(Error, Debug)]
pub enum Error {
    /// I/O operation error (simple form)
    #[error("I/O error: {source}")]
    IoSimple {
        /// The underlying I/O error
        #[from]
        source: std::io::Error,
    },

    /// I/O operation error (with context)
    #[error("I/O error: {message}")]
    Io {
        /// Description of the I/O error
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// JSON parsing or serialization error
    #[error("JSON parsing error: {source}")]
    Json {
        /// The underlying JSON error
        #[from]
        source: serde_json::Error,
    },

    /// Generic error from external sources
    #[error("Generic error: {0}")]
    Generic(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// UTF-8 encoding/decoding error
    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Base64 decoding error
    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    // /// Generic string-based error
    // #[error("String error: {0}")]
    // String(String),
    /// Invalid regular expression pattern
    #[error("Invalid regex pattern '{pattern}': {message}")]
    InvalidRegex {
        /// The regex pattern that failed to compile
        pattern: String,
        /// The compilation error message
        message: String,
    },

    /// Resource not found error
    #[error("Not found: {resource}")]
    NotFound {
        /// The resource that was not found
        resource: String,
    },

    /// Invalid argument provided to a function
    #[error("Invalid argument: {message}")]
    InvalidArgument {
        /// Description of the invalid argument
        message: String,
    },

    /// Vector database operation error
    #[error("Vector database error: {message}")]
    VectorDb {
        /// Description of the vector database error
        message: String,
    },

    /// Embedding provider operation error
    #[error("Embedding provider error: {message}")]
    Embedding {
        /// Description of the embedding provider error
        message: String,
    },

    /// Configuration-related error (simple form)
    #[error("Configuration error: {message}")]
    Config {
        /// Description of the configuration error
        message: String,
    },

    /// Configuration-related error (with source)
    #[error("Configuration error: {message}")]
    Configuration {
        /// Description of the configuration error
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Missing configuration field
    #[error("Missing configuration: {0}")]
    ConfigMissing(String),

    /// Invalid configuration value
    #[error("Invalid configuration for '{key}': {message}")]
    ConfigInvalid {
        /// The configuration key that is invalid
        key: String,
        /// Reason why it is invalid
        message: String,
    },

    /// Authentication-related error
    #[error("Authentication error: {message}")]
    Authentication {
        /// Description of the authentication error
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Network-related error
    #[error("Network error: {message}")]
    Network {
        /// Description of the network error
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Database-related error
    #[error("Database error: {message}")]
    Database {
        /// Description of the database error
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Internal system error
    #[error("Internal error: {message}")]
    Internal {
        /// Description of the internal error
        message: String,
    },

    /// Cache operation error
    #[error("Cache error: {message}")]
    Cache {
        /// Description of the cache error
        message: String,
    },

    /// Infrastructure operation error
    #[error("Infrastructure error: {message}")]
    Infrastructure {
        /// Description of the infrastructure error
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// VCS operation error
    #[error("VCS error: {message}")]
    Vcs {
        /// Description of the VCS error
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// VCS repository not found
    #[error("Repository not found: {path}")]
    RepositoryNotFound {
        /// Path to the repository that was not found
        path: String,
    },

    /// VCS branch not found
    #[error("Branch not found: {name}")]
    BranchNotFound {
        /// Name of the branch that was not found
        name: String,
    },

    /// Observation storage operation error
    #[error("Observation storage error: {message}")]
    ObservationStorage {
        /// Description of the observation storage error
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Observation not found
    #[error("Observation not found: {id}")]
    ObservationNotFound {
        /// ID of the observation
        id: String,
    },

    /// Duplicate observation
    #[error("Duplicate observation: {content_hash}")]
    DuplicateObservation {
        /// Content hash of duplicate
        content_hash: String,
    },

    /// Browse operation error
    #[error("Browse error: {0}")]
    Browse(#[from] crate::ports::browse::BrowseError),

    /// Highlighting operation error
    #[error("Highlighting error: {0}")]
    Highlight(#[from] crate::ports::browse::HighlightError),
}

// Basic error creation methods
impl Error {
    /// Create a generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic(message.into().into())
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

// I/O error creation methods
impl Error {
    /// Create an I/O error
    pub fn io<S: Into<String>>(message: S) -> Self {
        Self::Io {
            message: message.into(),
            source: None,
        }
    }

    /// Create an I/O error with source
    pub fn io_with_source<S: Into<String>, E: std::error::Error + Send + Sync + 'static>(
        message: S,
        source: E,
    ) -> Self {
        Self::Io {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

// Configuration error creation methods
impl Error {
    /// Create a configuration error (simple)
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a configuration error (with source)
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
            source: None,
        }
    }

    /// Create a configuration error with source
    pub fn configuration_with_source<
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::Configuration {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

// Authentication error creation methods
impl Error {
    /// Create an authentication error
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication {
            message: message.into(),
            source: None,
        }
    }

    /// Create an authentication error with source
    pub fn authentication_with_source<
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::Authentication {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

// Network error creation methods
impl Error {
    /// Create a network error
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
            source: None,
        }
    }

    /// Create a network error with source
    pub fn network_with_source<S: Into<String>, E: std::error::Error + Send + Sync + 'static>(
        message: S,
        source: E,
    ) -> Self {
        Self::Network {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

// Database error creation methods
impl Error {
    /// Create a database error
    pub fn database<S: Into<String>>(message: S) -> Self {
        Self::Database {
            message: message.into(),
            source: None,
        }
    }

    /// Create a database error with source
    pub fn database_with_source<S: Into<String>, E: std::error::Error + Send + Sync + 'static>(
        message: S,
        source: E,
    ) -> Self {
        Self::Database {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

// Internal and infrastructure error creation methods
impl Error {
    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create a cache error
    pub fn cache<S: Into<String>>(message: S) -> Self {
        Self::Cache {
            message: message.into(),
        }
    }

    /// Create an infrastructure error
    pub fn infrastructure<S: Into<String>>(message: S) -> Self {
        Self::Infrastructure {
            message: message.into(),
            source: None,
        }
    }

    /// Create an infrastructure error with source
    pub fn infrastructure_with_source<
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::Infrastructure {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

// VCS error creation methods
impl Error {
    /// Create a VCS error
    pub fn vcs<S: Into<String>>(message: S) -> Self {
        Self::Vcs {
            message: message.into(),
            source: None,
        }
    }

    /// Create a VCS error with source
    pub fn vcs_with_source<S: Into<String>, E: std::error::Error + Send + Sync + 'static>(
        message: S,
        source: E,
    ) -> Self {
        Self::Vcs {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a repository not found error
    pub fn repository_not_found<S: Into<String>>(path: S) -> Self {
        Self::RepositoryNotFound { path: path.into() }
    }

    /// Create a branch not found error
    pub fn branch_not_found<S: Into<String>>(name: S) -> Self {
        Self::BranchNotFound { name: name.into() }
    }
}

// Observation storage error creation methods
impl Error {
    /// Create an observation storage error
    pub fn memory<S: Into<String>>(message: S) -> Self {
        Self::ObservationStorage {
            message: message.into(),
            source: None,
        }
    }

    /// Create an observation storage error with source
    pub fn memory_with_source<S: Into<String>, E: std::error::Error + Send + Sync + 'static>(
        message: S,
        source: E,
    ) -> Self {
        Self::ObservationStorage {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an observation not found error
    pub fn observation_not_found<S: Into<String>>(id: S) -> Self {
        Self::ObservationNotFound { id: id.into() }
    }

    /// Create a duplicate observation error
    pub fn duplicate_observation<S: Into<String>>(content_hash: S) -> Self {
        Self::DuplicateObservation {
            content_hash: content_hash.into(),
        }
    }
}

// Note: OS-specific and external crate error conversions are excluded for domain purity.
// The infrastructure layer is responsible for these conversions.
