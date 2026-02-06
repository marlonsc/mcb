//! Browse Service - Agnóstico file tree and code highlighting
//!
//! Provides trait-based interface for browsing file trees and highlighting code.
//! Designed for multiple renderers: Web (Phase 8a), TUI (Phase 9), etc.

use mcb_domain::ports::browse::{
    BrowseError, BrowseResult, BrowseService, HighlightResult, HighlightService,
};
use mcb_domain::value_objects::browse::{FileNode, HighlightedCode};
use std::path::Path;
use std::sync::Arc;

/// Concrete browse service implementation
pub struct BrowseServiceImpl {
    highlight_service: Arc<dyn HighlightService>,
}

impl BrowseServiceImpl {
    pub fn new(highlight_service: Arc<dyn HighlightService>) -> Self {
        Self { highlight_service }
    }
}

#[async_trait::async_trait]
impl BrowseService for BrowseServiceImpl {
    async fn get_file_tree(&self, root: &Path, max_depth: usize) -> BrowseResult<FileNode> {
        if !root.exists() {
            return Err(BrowseError::PathNotFound(root.to_path_buf()));
        }

        self.walk_directory_boxed(root, 0, max_depth).await
    }

    async fn get_code(&self, path: &Path) -> BrowseResult<String> {
        if !path.exists() {
            return Err(BrowseError::PathNotFound(path.to_path_buf()));
        }

        tokio::fs::read_to_string(path)
            .await
            .map_err(BrowseError::Io)
    }

    async fn highlight(&self, code: &str, language: &str) -> HighlightResult<HighlightedCode> {
        self.highlight_service.highlight(code, language).await
    }

    async fn get_highlighted_code(&self, path: &Path) -> BrowseResult<HighlightedCode> {
        let code = self.get_code(path).await?;
        let lang = detect_language(path).unwrap_or_else(|| "text".to_string());
        self.highlight(&code, &lang)
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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = BrowseResult<FileNode>> + Send + '_>>
    {
        let path = path.to_path_buf();
        Box::pin(async move {
            let metadata = tokio::fs::metadata(&path).await.map_err(BrowseError::Io)?;
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("root")
                .to_string();

            let is_dir = metadata.is_dir();
            let language = if is_dir { None } else { detect_language(&path) };

            let children = if is_dir && depth < max_depth {
                let mut entries = tokio::fs::read_dir(&path).await.map_err(BrowseError::Io)?;
                let mut children = Vec::new();

                while let Ok(Some(entry)) = entries.next_entry().await {
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
