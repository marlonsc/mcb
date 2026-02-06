//! Browse and Highlight Port Definitions
//!
//! Traits for browsing file trees and syntax highlighting.
//! These define the contract for the browse service implementation.

use crate::error::Result;
use crate::value_objects::browse::{FileNode, HighlightedCode};
use std::path::Path;

/// Browse service trait (agnóstico interface)
#[async_trait::async_trait]
pub trait BrowseService: Send + Sync {
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
pub trait HighlightService: Send + Sync {
    /// Highlight code with given language
    ///
    /// Returns structured highlight spans with byte offsets.
    /// Falls back to empty spans if highlighting fails.
    async fn highlight(&self, code: &str, language: &str) -> Result<HighlightedCode>;
}
