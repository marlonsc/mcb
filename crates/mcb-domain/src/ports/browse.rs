//! Browse and Highlight Port Definitions
//!
//! Traits for browsing file trees and syntax highlighting.
//! These define the contract for the browse service implementation.

use crate::value_objects::browse::{FileNode, HighlightedCode};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Error type for browse operations
#[derive(Error, Debug)]
pub enum BrowseError {
    /// Specified path was not found
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    /// I/O error during browsing
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Highlighting failed for a file
    #[error("Highlighting failed: {0}")]
    HighlightingFailed(String),
}

/// Result type for browse operations
pub type BrowseResult<T> = std::result::Result<T, BrowseError>;

/// Error type for highlighting operations
#[derive(Error, Debug)]
pub enum HighlightError {
    /// Invalid configuration for highlighting
    #[error("Highlighting configuration error: {0}")]
    ConfigurationError(String),

    /// Language is not supported by the highlighter
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// Highlighting execution failed
    #[error("Highlighting failed: {0}")]
    HighlightingFailed(String),
}

/// Result type for highlighting operations
pub type HighlightResult<T> = std::result::Result<T, HighlightError>;

/// Browse service trait (agnóstico interface)
#[async_trait::async_trait]
pub trait BrowseService: Send + Sync {
    /// Get file tree from given root path
    async fn get_file_tree(&self, root: &Path, max_depth: usize) -> BrowseResult<FileNode>;

    /// Get raw code from file
    async fn get_code(&self, path: &Path) -> BrowseResult<String>;

    /// Highlight code with given language
    async fn highlight(&self, code: &str, language: &str) -> HighlightResult<HighlightedCode>;

    /// Get code with highlighting
    async fn get_highlighted_code(&self, path: &Path) -> BrowseResult<HighlightedCode>;
}

/// Highlight service trait (agnóstico interface)
#[async_trait::async_trait]
pub trait HighlightService: Send + Sync {
    /// Highlight code with given language
    ///
    /// Returns structured highlight spans with byte offsets.
    /// Falls back to empty spans if highlighting fails.
    async fn highlight(&self, code: &str, language: &str) -> HighlightResult<HighlightedCode>;
}
