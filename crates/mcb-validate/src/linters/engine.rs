//! Linter Engine Module
//!
//! Unified linter interface for coordinating multiple linters.

use std::path::Path;

use super::parsers::find_project_root;
use super::runners::{ClippyLinter, RuffLinter};
use super::types::{LintViolation, LinterType};
use crate::Result;

/// Unified linter interface
pub struct LinterEngine {
    /// List of linters to be used during check
    enabled_linters: Vec<LinterType>,
    /// Specific lint codes to enable (for Clippy lints that are "allow" by default)
    lint_codes: Vec<String>,
}

impl LinterEngine {
    /// Create a new linter engine with standard linters (Ruff and Clippy)
    #[must_use]
    pub fn new() -> Self {
        Self {
            enabled_linters: vec![LinterType::Ruff, LinterType::Clippy],
            lint_codes: Vec::new(),
        }
    }

    /// Create a new linter engine with a custom list of linters
    #[must_use]
    pub fn with_linters(linters: Vec<LinterType>) -> Self {
        Self {
            enabled_linters: linters,
            lint_codes: Vec::new(),
        }
    }

    /// Create engine with specific lint codes to enable
    #[must_use]
    pub fn with_lint_codes(linters: Vec<LinterType>, lint_codes: Vec<String>) -> Self {
        Self {
            enabled_linters: linters,
            lint_codes,
        }
    }

    /// Execute all enabled linters against the provided files
    pub async fn check_files(&self, files: &[&Path]) -> Result<Vec<LintViolation>> {
        let mut all_violations = Vec::new();

        // Check if Ruff is available and run it
        if self.enabled_linters.contains(&LinterType::Ruff)
            && let Ok(violations) = RuffLinter::check_files(files).await
        {
            all_violations.extend(violations);
        }

        // For Clippy, we need to check if any Rust files are present
        if self.enabled_linters.contains(&LinterType::Clippy) {
            let has_rust_files = files.iter().any(|f| {
                LinterType::Clippy
                    .matches_extension(f.extension().and_then(std::ffi::OsStr::to_str))
            });
            if has_rust_files {
                // Find project root (simplified - assumes files are in a Cargo project)
                if let Some(project_root) = find_project_root(files) {
                    // Pass lint codes to enable specific warnings
                    if let Ok(violations) =
                        ClippyLinter::check_project_with_lints(&project_root, &self.lint_codes)
                            .await
                    {
                        all_violations.extend(violations);
                    }
                }
            }
        }

        Ok(all_violations)
    }

    /// Map a linter-specific code to a custom rule ID
    #[must_use]
    pub fn map_lint_to_rule(&self, lint_code: &str) -> Option<&'static str> {
        match lint_code {
            // Ruff mappings
            "F401" => Some("QUAL005"), // Unused imports

            // Clippy mappings
            "clippy::unwrap_used" => Some("QUAL001"), // Unwrap usage

            _ => None,
        }
    }

    /// Get the list of enabled linters
    #[must_use]
    pub fn enabled_linters(&self) -> &[super::types::LinterType] {
        &self.enabled_linters
    }
}

impl Default for LinterEngine {
    fn default() -> Self {
        Self::new()
    }
}
