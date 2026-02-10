//! Linter Types Module
//!
//! Core types and data structures for linter integration.

/// Unified structure representing a code violation found by any linter.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LintViolation {
    /// The rule identifier (e.g., "E501", "clippy::unwrap_used").
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
    pub fn command(&self) -> &'static str {
        match self {
            LinterType::Ruff => "ruff",
            LinterType::Clippy => "cargo",
        }
    }

    /// Returns the file extension targeted by this linter.
    pub fn supported_extension(&self) -> &'static str {
        match self {
            LinterType::Ruff => "py",
            LinterType::Clippy => "rs",
        }
    }

    /// Checks if a file extension matches the linter's target type.
    pub fn matches_extension(&self, ext: Option<&str>) -> bool {
        ext == Some(self.supported_extension())
    }

    /// Generates the command-line arguments for running the linter on specific files.
    pub fn args(&self, files: &[&std::path::Path]) -> Vec<String> {
        match self {
            LinterType::Ruff => {
                let mut args = vec!["check".to_string(), "--output-format=json".to_string()];
                for file in files {
                    args.push(file.to_string_lossy().to_string());
                }
                args
            }
            LinterType::Clippy => {
                vec![
                    "clippy".to_string(),
                    "--message-format=json".to_string(),
                    "--".to_string(),
                ]
            }
        }
    }

    /// Parses the raw stdout output from the linter into a unified violation list.
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
    /// The string identifier (e.g., "clippy::unwrap_used").
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
