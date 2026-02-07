//! Tests for linter integration modules
//!
//! Uses shared constants for severity levels, rule codes, and file extensions.

use mcb_validate::linters::*;

use crate::test_constants::*;

#[test]
fn test_linter_engine_creation() {
    let engine = LinterEngine::new();
    assert!(!engine.enabled_linters().is_empty());
}

#[test]
fn test_ruff_severity_mapping() {
    assert_eq!(
        mcb_validate::linters::parsers::map_ruff_severity(RUFF_CODE_ERROR),
        SEVERITY_ERROR
    );
    assert_eq!(
        mcb_validate::linters::parsers::map_ruff_severity(RUFF_CODE_WARNING),
        SEVERITY_WARNING
    );
    assert_eq!(
        mcb_validate::linters::parsers::map_ruff_severity(RUFF_CODE_INFO),
        SEVERITY_INFO
    );
}

#[test]
fn test_clippy_level_mapping() {
    assert_eq!(
        mcb_validate::linters::parsers::map_clippy_level(SEVERITY_ERROR),
        SEVERITY_ERROR
    );
    assert_eq!(
        mcb_validate::linters::parsers::map_clippy_level(SEVERITY_WARNING),
        SEVERITY_WARNING
    );
    assert_eq!(
        mcb_validate::linters::parsers::map_clippy_level(CLIPPY_LEVEL_NOTE),
        SEVERITY_INFO
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
    assert_eq!(LinterType::Ruff.supported_extension(), RUFF_EXTENSION);
    assert_eq!(LinterType::Clippy.supported_extension(), CLIPPY_EXTENSION);
}

#[test]
fn test_linter_type_matches_extension() {
    assert!(LinterType::Ruff.matches_extension(Some(RUFF_EXTENSION)));
    assert!(!LinterType::Ruff.matches_extension(Some(CLIPPY_EXTENSION)));
    assert!(LinterType::Clippy.matches_extension(Some(CLIPPY_EXTENSION)));
    assert!(!LinterType::Clippy.matches_extension(Some(RUFF_EXTENSION)));
    assert!(!LinterType::Ruff.matches_extension(None));
}
