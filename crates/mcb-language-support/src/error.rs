//! Language Support Errors
//!
//! Defines error types for language detection, parsing, and chunking operations.

use thiserror::Error;

/// Language support error types
#[derive(Error, Debug)]
pub enum LanguageError {
    /// Language could not be detected from file
    #[error("Could not detect language for file: {path}")]
    DetectionFailed {
        /// Path that could not be detected
        path: String,
    },

    /// Language is not supported
    #[error("Unsupported language: {language}")]
    UnsupportedLanguage {
        /// The unsupported language identifier
        language: String,
    },

    /// Parsing failed
    #[error("Failed to parse {path}: {reason}")]
    ParseFailed {
        /// Path that failed to parse
        path: String,
        /// Reason for failure
        reason: String,
    },

    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Chunking strategy error
    #[error("Chunking error for {path}: {reason}")]
    ChunkingFailed {
        /// Path that failed to chunk
        path: String,
        /// Reason for failure
        reason: String,
    },
}

/// Result type alias for language support operations
pub type Result<T> = std::result::Result<T, LanguageError>;
