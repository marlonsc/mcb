//! Tests for linter integration modules

use mcb_validate::linters::*;

#[test]
fn test_linter_engine_creation() {
    let engine = LinterEngine::new();
    assert!(!engine.enabled_linters().is_empty());
}

#[test]
fn test_ruff_severity_mapping() {
    assert_eq!(
        mcb_validate::linters::parsers::map_ruff_severity("F401"),
        "error"
    );
    assert_eq!(
        mcb_validate::linters::parsers::map_ruff_severity("W291"),
        "warning"
    );
    assert_eq!(
        mcb_validate::linters::parsers::map_ruff_severity("I001"),
        "info"
    );
}

#[test]
fn test_clippy_level_mapping() {
    assert_eq!(
        mcb_validate::linters::parsers::map_clippy_level("error"),
        "error"
    );
    assert_eq!(
        mcb_validate::linters::parsers::map_clippy_level("warning"),
        "warning"
    );
    assert_eq!(
        mcb_validate::linters::parsers::map_clippy_level("note"),
        "info"
    );
}

#[tokio::test]
async fn test_linter_engine_execution() {
    let engine = LinterEngine::new();

    let result = engine.check_files(&[]).await;
    assert!(result.is_ok());
}

#[test]
fn test_linter_type_supported_extension() {
    assert_eq!(LinterType::Ruff.supported_extension(), "py");
    assert_eq!(LinterType::Clippy.supported_extension(), "rs");
}

#[test]
fn test_linter_type_matches_extension() {
    assert!(LinterType::Ruff.matches_extension(Some("py")));
    assert!(!LinterType::Ruff.matches_extension(Some("rs")));
    assert!(LinterType::Clippy.matches_extension(Some("rs")));
    assert!(!LinterType::Clippy.matches_extension(Some("py")));
    assert!(!LinterType::Ruff.matches_extension(None));
}
