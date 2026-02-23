//! Linter Types Module
//!
//! Core types and data structures for linter integration.

use std::path::PathBuf;

use crate::constants::engines::{LINTER_CMD_CARGO, LINTER_CMD_RUFF};
use crate::constants::severities::{
    CATEGORY_ARCHITECTURE, CATEGORY_ASYNC, CATEGORY_CLEAN_ARCHITECTURE, CATEGORY_CONFIGURATION,
    CATEGORY_DI, CATEGORY_DOCUMENTATION, CATEGORY_ERROR_BOUNDARY, CATEGORY_IMPLEMENTATION,
    CATEGORY_KISS, CATEGORY_MIGRATION, CATEGORY_NAMING, CATEGORY_ORGANIZATION,
    CATEGORY_PERFORMANCE, CATEGORY_PMAT, CATEGORY_REFACTORING, CATEGORY_SOLID, CATEGORY_TESTING,
    CATEGORY_WEB_FRAMEWORK, SEVERITY_ERROR, SEVERITY_INFO,
};
use crate::traits::violation::{Severity, Violation, ViolationCategory};
use derive_more::Display;

/// Unified structure representing a code violation found by any linter.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Display)]
#[display("[{rule}] {message}")]
pub struct LintViolation {
    /// The rule identifier (e.g., "E501", "`clippy::unwrap_used`").
    pub rule: String,
    /// The file path where the violation occurred.
    pub file: String,
    /// The line number of the violation (1-based).
    pub line: usize,
    /// The column number of the violation (1-based).
    pub column: usize,
    /// The descriptive message explaining the violation.
    pub message: String,
    /// The severity level (e.g., "error", "warning").
    pub severity: String,
    /// The category of the violation (e.g., "style", "correctness").
    pub category: String,
    /// Cached `PathBuf` for `Violation::file()` trait method.
    #[serde(skip)]
    pub file_path_cache: Option<PathBuf>,
}

impl LintViolation {
    /// Materializes the cached `PathBuf` from the `file` string field.
    pub fn ensure_file_path(&mut self) {
        if self.file_path_cache.is_none() {
            self.file_path_cache = Some(PathBuf::from(&self.file));
        }
    }

    fn parsed_severity(&self) -> Severity {
        match self.severity.to_ascii_lowercase().as_str() {
            SEVERITY_ERROR => Severity::Error,
            SEVERITY_INFO => Severity::Info,
            _ => Severity::Warning,
        }
    }

    fn parsed_category(&self) -> ViolationCategory {
        match self.category.to_ascii_lowercase().as_str() {
            CATEGORY_ARCHITECTURE | CATEGORY_CLEAN_ARCHITECTURE => ViolationCategory::Architecture,
            CATEGORY_ORGANIZATION => ViolationCategory::Organization,
            CATEGORY_SOLID => ViolationCategory::Solid,
            CATEGORY_DI => ViolationCategory::DependencyInjection,
            CATEGORY_CONFIGURATION => ViolationCategory::Configuration,
            CATEGORY_WEB_FRAMEWORK => ViolationCategory::WebFramework,
            CATEGORY_PERFORMANCE => ViolationCategory::Performance,
            CATEGORY_ASYNC => ViolationCategory::Async,
            CATEGORY_DOCUMENTATION => ViolationCategory::Documentation,
            CATEGORY_TESTING => ViolationCategory::Testing,
            CATEGORY_NAMING => ViolationCategory::Naming,
            CATEGORY_KISS => ViolationCategory::Kiss,
            CATEGORY_REFACTORING | CATEGORY_MIGRATION => ViolationCategory::Refactoring,
            CATEGORY_ERROR_BOUNDARY => ViolationCategory::ErrorBoundary,
            CATEGORY_IMPLEMENTATION => ViolationCategory::Implementation,
            CATEGORY_PMAT => ViolationCategory::Pmat,
            _ => ViolationCategory::Quality,
        }
    }
}

impl Violation for LintViolation {
    fn id(&self) -> &str {
        &self.rule
    }

    fn category(&self) -> ViolationCategory {
        self.parsed_category()
    }

    fn severity(&self) -> Severity {
        self.parsed_severity()
    }

    fn file(&self) -> Option<&PathBuf> {
        self.file_path_cache.as_ref()
    }

    fn line(&self) -> Option<usize> {
        Some(self.line)
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

/// Supported external linter tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinterType {
    /// The Ruff Python linter.
    Ruff,
    /// The Rust Clippy linter.
    Clippy,
}

impl LinterType {
    /// Returns the executable command name for the linter.
    #[must_use]
    pub fn command(&self) -> &'static str {
        match self {
            LinterType::Ruff => LINTER_CMD_RUFF,
            LinterType::Clippy => LINTER_CMD_CARGO,
        }
    }

    /// Returns the file extension targeted by this linter.
    #[must_use]
    pub fn supported_extension(&self) -> &'static str {
        match self {
            LinterType::Ruff => "py",
            LinterType::Clippy => "rs",
        }
    }

    /// Checks if a file extension matches the linter's target type.
    #[must_use]
    pub fn matches_extension(&self, ext: Option<&str>) -> bool {
        ext == Some(self.supported_extension())
    }

    /// Generates the command-line arguments for running the linter on specific files.
    #[must_use]
    pub fn args(&self, files: &[&std::path::Path]) -> Vec<String> {
        match self {
            LinterType::Ruff => {
                let mut args = vec!["check".to_owned(), "--output-format=json".to_owned()];
                for file in files {
                    if let Some(file_str) = file.to_str() {
                        args.push(file_str.to_owned());
                    }
                }
                args
            }
            LinterType::Clippy => {
                vec![
                    "clippy".to_owned(),
                    "--message-format=json".to_owned(),
                    "--".to_owned(),
                ]
            }
        }
    }

    /// Parses the raw stdout output from the linter into a unified violation list.
    #[must_use]
    pub fn parse_output(&self, output: &str) -> Vec<LintViolation> {
        match self {
            LinterType::Ruff => crate::linters::parsers::parse_ruff_output(output),
            LinterType::Clippy => crate::linters::parsers::parse_clippy_output(output),
        }
    }
}

/// Represents a violation reported by the Ruff linter.
#[derive(serde::Deserialize)]
pub struct RuffViolation {
    /// The violation code.
    pub code: String,
    /// The violation message.
    pub message: String,
    /// The filename.
    pub filename: String,
    /// The location of the violation.
    pub location: RuffLocation,
}

/// Represents the location of a Ruff violation.
#[derive(serde::Deserialize)]
pub struct RuffLocation {
    /// The row number.
    pub row: usize,
    /// The column number.
    pub column: usize,
}

/// Represents a single JSON output line from Cargo Clippy.
#[derive(serde::Deserialize)]
pub struct ClippyOutput {
    /// The type of message (e.g., "compiler-message").
    pub reason: String,
    /// The content of the message.
    pub message: ClippyMessageContent,
}

/// Content of a Clippy compiler message.
#[derive(serde::Deserialize)]
pub struct ClippyMessageContent {
    /// The warning/error message.
    pub message: String,
    /// The associated error code info.
    pub code: Option<ClippyCode>,
    /// The severity level.
    pub level: String,
    /// List of source code spans associated with the message.
    pub spans: Vec<ClippySpan>,
}

/// Code identifier info for a Clippy message.
#[derive(serde::Deserialize)]
pub struct ClippyCode {
    /// The string identifier (e.g., "`clippy::unwrap_used`").
    pub code: String,
    /// Optional explanation of the code.
    pub explanation: Option<String>,
}

/// Source code span info for a Clippy message.
#[derive(serde::Deserialize)]
pub struct ClippySpan {
    /// The file name.
    pub file_name: String,
    /// The starting line number.
    pub line_start: usize,
    /// The starting column number.
    pub column_start: usize,
    /// Whether this is the primary span for the message.
    #[serde(default)]
    pub is_primary: bool,
}
