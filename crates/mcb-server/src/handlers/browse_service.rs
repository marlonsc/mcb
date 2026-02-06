//! Browse Service - Agnóstico file tree and code highlighting
//!
//! Provides trait-based interface for browsing file trees and highlighting code.
//! Designed for multiple renderers: Web (Phase 8a), TUI (Phase 9), etc.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

use super::HighlightService;

/// Browse service errors
#[derive(Debug, Error)]
pub enum BrowseError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Tree parsing failed: {0}")]
    TreeParsingFailed(String),

    #[error("Highlighting failed: {0}")]
    HighlightingFailed(String),
}

pub type Result<T> = std::result::Result<T, BrowseError>;

/// File tree node structure (agnóstico, used by all renderers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<usize>,
}

/// Highlighted code spans (agnóstico format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub category: HighlightCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HighlightCategory {
    Keyword,
    String,
    Comment,
    Function,
    Variable,
    Type,
    Number,
    Operator,
    Punctuation,
    Other,
}

/// Highlighted code result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightedCode {
    pub original: String,
    pub spans: Vec<HighlightSpan>,
    pub language: String,
}

/// Browse service trait (agnóstico interface)
pub trait BrowseService: Send + Sync {
    /// Get file tree from given root path
    fn get_file_tree(
        &self,
        root: &Path,
        max_depth: usize,
    ) -> impl std::future::Future<Output = Result<FileNode>> + Send;

    /// Get raw code from file
    fn get_code(&self, path: &Path) -> impl std::future::Future<Output = Result<String>> + Send;

    /// Highlight code with given language
    fn highlight(
        &self,
        code: &str,
        language: &str,
    ) -> impl std::future::Future<Output = Result<HighlightedCode>> + Send;

    /// Get code with highlighting
    fn get_highlighted_code(
        &self,
        path: &Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<HighlightedCode>> + Send + '_>>
    {
        let path = path.to_path_buf();
        Box::pin(async move {
            let code = self.get_code(&path).await?;
            let lang = detect_language(&path).unwrap_or_else(|| "text".to_string());
            self.highlight(&code, &lang).await
        })
    }
}

/// Concrete browse service implementation
pub struct BrowseServiceImpl {
    highlight_service: Arc<super::HighlightServiceImpl>,
}

impl Default for BrowseServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowseServiceImpl {
    pub fn new() -> Self {
        Self {
            highlight_service: Arc::new(super::HighlightServiceImpl::new()),
        }
    }

    pub fn with_highlight_service(highlight_service: Arc<super::HighlightServiceImpl>) -> Self {
        Self { highlight_service }
    }
}

impl BrowseService for BrowseServiceImpl {
    async fn get_file_tree(&self, root: &Path, max_depth: usize) -> Result<FileNode> {
        if !root.exists() {
            return Err(BrowseError::PathNotFound(root.to_path_buf()));
        }

        self.walk_directory_boxed(root, 0, max_depth).await
    }

    async fn get_code(&self, path: &Path) -> Result<String> {
        if !path.exists() {
            return Err(BrowseError::PathNotFound(path.to_path_buf()));
        }

        tokio::fs::read_to_string(path)
            .await
            .map_err(BrowseError::Io)
    }

    async fn highlight(&self, code: &str, language: &str) -> Result<HighlightedCode> {
        self.highlight_service
            .highlight(code, language)
            .await
            .map_err(|e| BrowseError::HighlightingFailed(e.to_string()))
    }
}

impl BrowseServiceImpl {
    fn walk_directory_boxed(
        &self,
        path: &Path,
        depth: usize,
        max_depth: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FileNode>> + Send + '_>> {
        let path = path.to_path_buf();
        Box::pin(async move {
            let metadata = tokio::fs::metadata(&path).await?;
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("root")
                .to_string();

            let is_dir = metadata.is_dir();
            let language = if is_dir { None } else { detect_language(&path) };

            let children = if is_dir && depth < max_depth {
                let mut entries = tokio::fs::read_dir(&path).await?;
                let mut children = Vec::new();

                while let Some(entry) = entries.next_entry().await? {
                    let entry_path = entry.path();

                    if should_skip_path(&entry_path) {
                        continue;
                    }

                    match self
                        .walk_directory_boxed(&entry_path, depth + 1, max_depth)
                        .await
                    {
                        Ok(node) => children.push(node),
                        Err(_) => continue,
                    }
                }

                if children.is_empty() {
                    None
                } else {
                    Some(children)
                }
            } else {
                None
            };

            Ok(FileNode {
                path,
                name,
                is_dir,
                children,
                language,
                lines: None,
            })
        })
    }
}

/// Detect programming language from file extension.
/// Public for unit tests in tests/unit/browse_service_tests.rs.
pub fn detect_language(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    let lang = match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" | "jsx" => "tsx",
        "java" => "java",
        "go" => "go",
        "rb" => "ruby",
        "php" => "php",
        "kt" => "kotlin",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "cs" => "csharp",
        "swift" => "swift",
        "md" | "markdown" => "markdown",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        _ => return None,
    };
    Some(lang.to_string())
}

/// Skip hidden and common ignore patterns.
/// Public for unit tests in tests/unit/browse_service_tests.rs.
pub fn should_skip_path(path: &Path) -> bool {
    let name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return true,
    };

    if name.starts_with('.') && name != ".git" {
        return true;
    }

    matches!(
        name,
        "node_modules"
            | "target"
            | ".cache"
            | ".venv"
            | "__pycache__"
            | ".idea"
            | ".vscode"
            | ".DS_Store"
    )
}
