//! Linter Integration Module
//!
//! Integrates external linters (Ruff, Clippy) as first-layer validation
//! that feeds into the unified violation reporting system.

pub mod engine;
pub mod executor;
pub mod parsers;
pub mod runners;
pub mod types;

// Re-export public types and interfaces
pub use engine::LinterEngine;
pub use executor::YamlRuleExecutor;
pub use runners::{ClippyLinter, RuffLinter};
pub use types::{LintViolation, LinterType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linter_engine_creation() {
        let engine = LinterEngine::new();
        assert!(!engine.enabled_linters().is_empty());
    }

    #[test]
    fn test_ruff_severity_mapping() {
        assert_eq!(super::parsers::map_ruff_severity("F401"), "error");
        assert_eq!(super::parsers::map_ruff_severity("W291"), "warning");
        assert_eq!(super::parsers::map_ruff_severity("I001"), "info");
    }

    #[test]
    fn test_clippy_level_mapping() {
        assert_eq!(super::parsers::map_clippy_level("error"), "error");
        assert_eq!(super::parsers::map_clippy_level("warning"), "warning");
        assert_eq!(super::parsers::map_clippy_level("note"), "info");
    }

    #[tokio::test]
    async fn test_linter_engine_execution() {
        let engine = LinterEngine::new();

        // Test with non-existent files (should not panic)
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
}
