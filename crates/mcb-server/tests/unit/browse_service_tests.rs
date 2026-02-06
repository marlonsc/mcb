//! Unit tests for browse service (language detection, path skip, file tree).
//!
//! Moved from inline tests in src/handlers/browse_service.rs.

use mcb_domain::ports::browse::{BrowseError, BrowseService};
use mcb_server::handlers::browse_service::{BrowseServiceImpl, detect_language, should_skip_path};
use mcb_server::handlers::highlight_service::HighlightServiceImpl;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

// ============================================================================
// Language detection tests
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
// Path skipping tests
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
    assert!(!should_skip_path(Path::new(".git")));
}

// ============================================================================
// File tree walk tests
// ============================================================================

#[tokio::test]
async fn test_walk_directory_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, "fn main() {}").unwrap();

    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));
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

    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));
    let result = service
        .get_file_tree(temp_dir.path(), 1)
        .await
        .expect("Failed to walk directory");

    assert!(result.is_dir);
    let children = result.children.unwrap();
    assert_eq!(children.len(), 3);

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

    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));
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

    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));

    let result = service
        .get_file_tree(temp_dir.path(), 1)
        .await
        .expect("Failed to walk directory");

    let children = result.children.unwrap();
    let src_node = &children[0];
    assert_eq!(src_node.name, "src");
    assert!(src_node.children.is_none());
}

#[tokio::test]
async fn test_walk_directory_skips_node_modules() {
    let temp_dir = TempDir::new().unwrap();
    fs::create_dir(temp_dir.path().join("node_modules")).unwrap();
    fs::write(temp_dir.path().join("package.json"), "{}").unwrap();

    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));
    let result = service
        .get_file_tree(temp_dir.path(), 2)
        .await
        .expect("Failed to walk directory");

    let children = result.children.unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name, "package.json");
}

#[tokio::test]
async fn test_walk_directory_skips_hidden_dirs() {
    let temp_dir = TempDir::new().unwrap();
    fs::create_dir(temp_dir.path().join(".git")).unwrap();
    fs::write(temp_dir.path().join("source.rs"), "code").unwrap();

    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));
    let result = service
        .get_file_tree(temp_dir.path(), 2)
        .await
        .expect("Failed to walk directory");

    let children = result.children.unwrap();
    assert!(children.iter().any(|c| c.name == "source.rs"));
}

// ============================================================================
// Error handling tests
// ============================================================================

#[tokio::test]
async fn test_get_file_tree_nonexistent_path() {
    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));
    let result = service
        .get_file_tree(Path::new("/nonexistent/path/12345"), 1)
        .await;

    assert!(result.is_err());
    assert!(result.is_err());
    match result.unwrap_err() {
        BrowseError::PathNotFound(_) => {}
        e => panic!("Expected PathNotFound error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_get_code_nonexistent_file() {
    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));
    let result = service.get_code(Path::new("/nonexistent/file.rs")).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_code_success() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    let content = "fn main() { println!(\"hello\"); }";
    fs::write(&file_path, content).unwrap();

    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));
    let result = service.get_code(&file_path).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), content);
}

// ============================================================================
// Highlighting tests
// ============================================================================

#[tokio::test]
async fn test_highlight_basic() {
    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));
    let code = "fn main() {}";
    let result = service
        .highlight(code, "rust")
        .await
        .expect("Failed to highlight");

    assert_eq!(result.original, code);
    assert_eq!(result.language, "rust");
    assert!(!result.spans.is_empty());
}

#[tokio::test]
async fn test_get_highlighted_code() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    let content = "fn main() {}";
    fs::write(&file_path, content).unwrap();

    let service = BrowseServiceImpl::new(Arc::new(HighlightServiceImpl::new()));
    let result = service
        .get_highlighted_code(&file_path)
        .await
        .expect("Failed to get highlighted code");

    assert_eq!(result.original, content);
    assert_eq!(result.language, "rust");
}
