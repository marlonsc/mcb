//! AST Utilities Errors
//!
//! Defines error types for AST traversal and analysis operations.

use thiserror::Error;

/// AST utilities error types
#[derive(Error, Debug)]
pub enum AstError {
    /// Tree-sitter parsing failed
    #[error("Failed to parse source: {reason}")]
    ParseFailed {
        /// Reason for failure
        reason: String,
    },

    /// Language is not supported for tree-sitter parsing
    #[error("Unsupported language for tree-sitter: {language}")]
    UnsupportedLanguage {
        /// The unsupported language
        language: String,
    },

    /// Node not found during traversal
    #[error("Node not found: {node_type}")]
    NodeNotFound {
        /// The expected node type
        node_type: String,
    },

    /// Symbol extraction failed
    #[error("Failed to extract symbol: {reason}")]
    SymbolExtractionFailed {
        /// Reason for failure
        reason: String,
    },
}

/// Result type alias for AST utilities operations
pub type Result<T> = std::result::Result<T, AstError>;
