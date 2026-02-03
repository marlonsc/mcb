//! Linter Types Module
//!
//! Core types and data structures for linter integration.

/// Unified linter violation format
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LintViolation {
    pub rule: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: String,
    pub category: String,
}

/// Supported linter types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinterType {
    Ruff,
    Clippy,
}

impl LinterType {
    pub fn command(&self) -> &'static str {
        match self {
            LinterType::Ruff => "ruff",
            LinterType::Clippy => "cargo",
        }
    }

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

    pub fn parse_output(&self, output: &str) -> Vec<LintViolation> {
        match self {
            LinterType::Ruff => crate::linters::parsers::parse_ruff_output(output),
            LinterType::Clippy => crate::linters::parsers::parse_clippy_output(output),
        }
    }
}

/// Ruff violation format
#[derive(serde::Deserialize)]
pub struct RuffViolation {
    pub code: String,
    pub message: String,
    pub filename: String,
    pub location: RuffLocation,
}

#[derive(serde::Deserialize)]
pub struct RuffLocation {
    pub row: usize,
    pub column: usize,
}

/// Clippy output format (JSON lines with reason field)
#[derive(serde::Deserialize)]
pub struct ClippyOutput {
    pub reason: String,
    pub message: ClippyMessageContent,
}

#[derive(serde::Deserialize)]
pub struct ClippyMessageContent {
    pub message: String,
    pub code: Option<ClippyCode>,
    pub level: String,
    pub spans: Vec<ClippySpan>,
}

/// Clippy code is nested: {"code": "`clippy::unwrap_used`", "explanation": null}
#[derive(serde::Deserialize)]
pub struct ClippyCode {
    pub code: String,
    pub explanation: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct ClippySpan {
    pub file_name: String,
    pub line_start: usize,
    pub column_start: usize,
    #[serde(default)]
    pub is_primary: bool,
}
