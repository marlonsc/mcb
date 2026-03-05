//! Utility error types for mcb-utils (Layer 0).
//!
//! Self-contained errors with zero domain knowledge.
//! Consumer crates convert via `From<UtilsError>` in their own error modules.

/// Errors produced by utility functions in mcb-utils.
///
/// Each variant maps to a category of failure that utility code can encounter.
/// Domain crates implement `From<UtilsError>` to convert into their own error types.
#[derive(Debug, thiserror::Error)]
pub enum UtilsError {
    /// Standard I/O errors (file open, read, write).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid or non-UTF-8 path, strip-prefix failure, canonicalization failure.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Generic parse failure (regex compilation, format issues).
    #[error("Parse error: {0}")]
    Parse(String),

    /// System clock or time conversion errors.
    #[error("Time error: {0}")]
    Time(String),

    /// Hash computation or digest errors.
    #[error("Hash error: {0}")]
    Hash(String),

    /// Base64, hex, or other encoding/decoding errors.
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Directory traversal, file-type detection, or recursive scan errors.
    #[error("Filesystem error: {0}")]
    Filesystem(String),

    /// Regex compilation or matching errors.
    #[error("Regex error: {0}")]
    Regex(String),

    /// Catch-all for errors that don't fit other categories.
    #[error("{0}")]
    Other(String),
}

/// Convenience alias used throughout mcb-utils.
pub type Result<T> = std::result::Result<T, UtilsError>;
