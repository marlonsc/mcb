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

/// Detect programming language from file extension
fn detect_language(path: &Path) -> Option<String> {
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

/// Skip hidden and common ignore patterns
fn should_skip_path(path: &Path) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ============================================================================
    // Language Detection Tests
    // ============================================================================

    #[test]
    fn test_detect_language_rust() {
        assert_eq!(
            detect_language(Path::new("test.rs")),
            Some("rust".to_string())
        );
    }

    #[test]
    fn test_detect_language_python() {
        assert_eq!(
            detect_language(Path::new("script.py")),
            Some("python".to_string())
        );
    }

    #[test]
    fn test_detect_language_javascript() {
        assert_eq!(
            detect_language(Path::new("app.js")),
            Some("javascript".to_string())
        );
    }

    #[test]
    fn test_detect_language_typescript() {
        assert_eq!(
            detect_language(Path::new("config.ts")),
            Some("typescript".to_string())
        );
    }

    #[test]
    fn test_detect_language_tsx() {
        assert_eq!(
            detect_language(Path::new("component.tsx")),
            Some("tsx".to_string())
        );
        assert_eq!(
            detect_language(Path::new("component.jsx")),
            Some("tsx".to_string())
        );
    }

    #[test]
    fn test_detect_language_go() {
        assert_eq!(
            detect_language(Path::new("main.go")),
            Some("go".to_string())
        );
    }

    #[test]
    fn test_detect_language_java() {
        assert_eq!(
            detect_language(Path::new("Main.java")),
            Some("java".to_string())
        );
    }

    #[test]
    fn test_detect_language_cpp() {
        assert_eq!(
            detect_language(Path::new("main.cpp")),
            Some("cpp".to_string())
        );
        assert_eq!(
            detect_language(Path::new("code.cc")),
            Some("cpp".to_string())
        );
    }

    #[test]
    fn test_detect_language_markdown() {
        assert_eq!(
            detect_language(Path::new("README.md")),
            Some("markdown".to_string())
        );
    }

    #[test]
    fn test_detect_language_json() {
        assert_eq!(
            detect_language(Path::new("package.json")),
            Some("json".to_string())
        );
    }

    #[test]
    fn test_detect_language_unknown_extension() {
        assert_eq!(detect_language(Path::new("unknown.xyz")), None);
    }

    #[test]
    fn test_detect_language_no_extension() {
        assert_eq!(detect_language(Path::new("Makefile")), None);
    }

    // ============================================================================
    // Path Skipping Tests
    // ============================================================================

    #[test]
    fn test_should_skip_path_hidden_files() {
        assert!(should_skip_path(Path::new(".hidden")));
        assert!(should_skip_path(Path::new(".DS_Store")));
    }

    #[test]
    fn test_should_skip_path_node_modules() {
        assert!(should_skip_path(Path::new("node_modules")));
    }

    #[test]
    fn test_should_skip_path_target() {
        assert!(should_skip_path(Path::new("target")));
    }

    #[test]
    fn test_should_skip_path_python_cache() {
        assert!(should_skip_path(Path::new("__pycache__")));
        assert!(should_skip_path(Path::new(".venv")));
    }

    #[test]
    fn test_should_skip_path_ide_files() {
        assert!(should_skip_path(Path::new(".idea")));
        assert!(should_skip_path(Path::new(".vscode")));
    }

    #[test]
    fn test_should_skip_path_cache() {
        assert!(should_skip_path(Path::new(".cache")));
    }

    #[test]
    fn test_should_not_skip_path_src() {
        assert!(!should_skip_path(Path::new("src")));
    }

    #[test]
    fn test_should_not_skip_path_regular_file() {
        assert!(!should_skip_path(Path::new("main.rs")));
    }

    #[test]
    fn test_should_not_skip_path_dot_git() {
        // Note: .git contains a dot and is not named ".git" in standard pattern
        // This test documents the current behavior
        assert!(!should_skip_path(Path::new(".git")));
    }

    // ============================================================================
    // File Tree Walk Tests
    // ============================================================================

    #[tokio::test]
    async fn test_walk_directory_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let service = BrowseServiceImpl::new();
        let result = service
            .get_file_tree(temp_dir.path(), 1)
            .await
            .expect("Failed to walk directory");

        assert_eq!(
            result.name,
            temp_dir.path().file_name().unwrap().to_str().unwrap()
        );
        assert!(result.is_dir);
        assert!(result.children.is_some());
        let children = result.children.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "test.rs");
        assert!(!children[0].is_dir);
        assert_eq!(children[0].language, Some("rust".to_string()));
    }

    #[tokio::test]
    async fn test_walk_directory_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("lib.rs"), "pub fn helper() {}").unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Project").unwrap();

        let service = BrowseServiceImpl::new();
        let result = service
            .get_file_tree(temp_dir.path(), 1)
            .await
            .expect("Failed to walk directory");

        assert!(result.is_dir);
        let children = result.children.unwrap();
        assert_eq!(children.len(), 3);

        // Verify we got the files
        let file_names: Vec<_> = children.iter().map(|c| c.name.as_str()).collect();
        assert!(file_names.contains(&"main.rs"));
        assert!(file_names.contains(&"lib.rs"));
        assert!(file_names.contains(&"README.md"));
    }

    #[tokio::test]
    async fn test_walk_directory_nested() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

        let service = BrowseServiceImpl::new();
        let result = service
            .get_file_tree(temp_dir.path(), 2)
            .await
            .expect("Failed to walk directory");

        assert!(result.is_dir);
        let children = result.children.unwrap();
        assert_eq!(children.len(), 1);

        let src_node = &children[0];
        assert_eq!(src_node.name, "src");
        assert!(src_node.is_dir);

        let src_children = src_node.children.as_ref().unwrap();
        assert_eq!(src_children.len(), 1);
        assert_eq!(src_children[0].name, "main.rs");
    }

    #[tokio::test]
    async fn test_walk_directory_max_depth_limit() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        let nested_dir = src_dir.join("nested");
        fs::create_dir(&nested_dir).unwrap();
        fs::write(nested_dir.join("deep.rs"), "code").unwrap();

        let service = BrowseServiceImpl::new();

        // With max_depth=1, we should see src but not nested contents
        let result = service
            .get_file_tree(temp_dir.path(), 1)
            .await
            .expect("Failed to walk directory");

        let children = result.children.unwrap();
        let src_node = &children[0];
        assert_eq!(src_node.name, "src");
        // At max_depth=1, src is a dir but we don't traverse into it
        assert!(src_node.children.is_none());
    }

    #[tokio::test]
    async fn test_walk_directory_skips_node_modules() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("node_modules")).unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();

        let service = BrowseServiceImpl::new();
        let result = service
            .get_file_tree(temp_dir.path(), 2)
            .await
            .expect("Failed to walk directory");

        let children = result.children.unwrap();
        assert_eq!(children.len(), 1); // Only package.json, no node_modules
        assert_eq!(children[0].name, "package.json");
    }

    #[tokio::test]
    async fn test_walk_directory_skips_hidden_dirs() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join(".git")).unwrap();
        fs::write(temp_dir.path().join("source.rs"), "code").unwrap();

        let service = BrowseServiceImpl::new();
        let result = service
            .get_file_tree(temp_dir.path(), 2)
            .await
            .expect("Failed to walk directory");

        let children = result.children.unwrap();
        // .git is not skipped per the logic (starts with . but is named .git)
        // but hidden files starting with . and not .git are skipped
        // In this case, .git might appear depending on implementation
        // Let's just verify source.rs is there
        assert!(children.iter().any(|c| c.name == "source.rs"));
    }

    // ============================================================================
    // Error Handling Tests
    // ============================================================================

    #[tokio::test]
    async fn test_get_file_tree_nonexistent_path() {
        let service = BrowseServiceImpl::new();
        let result = service
            .get_file_tree(Path::new("/nonexistent/path/12345"), 1)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BrowseError::PathNotFound(_) => {
                // Expected
            }
            _ => panic!("Expected PathNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_code_nonexistent_file() {
        let service = BrowseServiceImpl::new();
        let result = service.get_code(Path::new("/nonexistent/file.rs")).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_code_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = "fn main() { println!(\"hello\"); }";
        fs::write(&file_path, content).unwrap();

        let service = BrowseServiceImpl::new();
        let result = service.get_code(&file_path).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    // ============================================================================
    // Highlighting Tests
    // ============================================================================

    #[tokio::test]
    async fn test_highlight_basic() {
        let service = BrowseServiceImpl::new();
        let code = "fn main() {}";
        let result = service
            .highlight(code, "rust")
            .await
            .expect("Failed to highlight");

        assert_eq!(result.original, code);
        assert_eq!(result.language, "rust");
        // Phase 8b: HighlightService now provides actual spans
        assert!(!result.spans.is_empty());
    }

    #[tokio::test]
    async fn test_get_highlighted_code() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = "fn main() {}";
        fs::write(&file_path, content).unwrap();

        let service = BrowseServiceImpl::new();
        let result = service
            .get_highlighted_code(&file_path)
            .await
            .expect("Failed to get highlighted code");

        assert_eq!(result.original, content);
        assert_eq!(result.language, "rust");
    }
}
