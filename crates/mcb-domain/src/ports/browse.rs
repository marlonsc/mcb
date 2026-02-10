//! Browse and Highlight Port Definitions
//!
//! Traits for browsing file trees and syntax highlighting.
//! These define the contract for the browse service implementation.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::error::Result;
use crate::value_objects::browse::{FileNode, HighlightedCode};

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

/// Browse service trait (agnóstico interface)
#[async_trait::async_trait]
pub trait BrowseServiceInterface: Send + Sync {
    /// Get file tree from given root path
    async fn get_file_tree(&self, root: &Path, max_depth: usize) -> Result<FileNode>;

    /// Get raw code from file
    async fn get_code(&self, path: &Path) -> Result<String>;

    /// Highlight code with given language
    async fn highlight(&self, code: &str, language: &str) -> Result<HighlightedCode>;

    /// Get code with highlighting
    async fn get_highlighted_code(&self, path: &Path) -> Result<HighlightedCode>;
}

/// Highlight service trait (agnóstico interface)
#[async_trait::async_trait]
pub trait HighlightServiceInterface: Send + Sync {
    /// Highlight code with given language
    ///
    /// Returns structured highlight spans with byte offsets.
    /// Falls back to empty spans if highlighting fails.
    async fn highlight(&self, code: &str, language: &str) -> Result<HighlightedCode>;
}
