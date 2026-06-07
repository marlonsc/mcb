//! Tests for linter integration modules
//!
//! Uses shared constants for severity levels, rule codes, and file extensions.

use mcb_validate::linters::*;
use rstest::rstest;

use crate::utils::test_constants::*;

#[rstest]
fn linter_engine_creation() {
    let engine = LinterEngine::new();
    assert!(!engine.enabled_linters().is_empty());
}

#[rstest]
#[case("ruff", RUFF_CODE_ERROR, SEVERITY_ERROR)]
#[case("ruff", RUFF_CODE_WARNING, SEVERITY_WARNING)]
#[case("ruff", RUFF_CODE_INFO, SEVERITY_INFO)]
#[case("clippy", SEVERITY_ERROR, SEVERITY_ERROR)]
#[case("clippy", SEVERITY_WARNING, SEVERITY_WARNING)]
#[case("clippy", CLIPPY_LEVEL_NOTE, SEVERITY_INFO)]
fn linter_level_mapping(#[case] linter: &str, #[case] input_level: &str, #[case] expected: &str) {
    let actual = if linter == "ruff" {
        mcb_validate::linters::parsers::map_ruff_severity(input_level)
    } else {
        mcb_validate::linters::parsers::map_clippy_level(input_level)
    };
    assert_eq!(actual, expected);
}

#[rstest]
#[tokio::test]
async fn linter_engine_execution() {
    let engine = LinterEngine::new();

    let result = engine.check_files(&[]).await;
    let output = result.expect("linter should execute successfully");
    assert_eq!(output.len(), 0);
}

#[rstest]
#[case(LinterType::Ruff, RUFF_EXTENSION)]
#[case(LinterType::Clippy, CLIPPY_EXTENSION)]
fn linter_type_supported_extension(#[case] linter: LinterType, #[case] expected: &str) {
    assert_eq!(linter.supported_extension(), expected);
}

#[rstest]
#[case(LinterType::Ruff, Some(RUFF_EXTENSION), true)]
#[case(LinterType::Ruff, Some(CLIPPY_EXTENSION), false)]
#[case(LinterType::Clippy, Some(CLIPPY_EXTENSION), true)]
#[case(LinterType::Clippy, Some(RUFF_EXTENSION), false)]
#[case(LinterType::Ruff, None, false)]
fn linter_type_matches_extension(
    #[case] linter: LinterType,
    #[case] extension: Option<&str>,
    #[case] expected: bool,
) {
    assert_eq!(linter.matches_extension(extension), expected);
}
