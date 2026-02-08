//! Error Boundary Validation
//!
//! Validates Clean Architecture error handling patterns:
//! - Layer error wrapping (domain wraps infrastructure errors)
//! - Context preservation across layers
//! - Error type placement (right layer)

use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig, ValidationError};

/// Error boundary violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorBoundaryViolation {
    /// Error crossing layer without context
    MissingErrorContext {
        /// File containing the violation.
        file: PathBuf,
        /// Line number where the error propagation occurred.
        line: usize,
        /// Description of the error pattern detected (e.g., "?" operator usage).
        error_pattern: String,
        /// Recommended fix for the violation.
        suggestion: String,
        /// Severity of the violation.
        severity: Severity,
    },
    /// Infrastructure error type used in domain layer
    WrongLayerError {
        /// File containing the violation.
        file: PathBuf,
        /// Line number where the incorrect error type was used.
        line: usize,
        /// The infrastructure error type detected (e.g., "std::io::Error").
        error_type: String,
        /// The architectural layer where the violation occurred.
        layer: String,
        /// Severity of the violation.
        severity: Severity,
    },
    /// Internal error details leaked to external API
    LeakedInternalError {
        /// File containing the violation.
        file: PathBuf,
        /// Line number where the internal error leak occurred.
        line: usize,
        /// Description of the leak pattern detected (e.g., Debug formatting).
        pattern: String,
        /// Severity of the violation.
        severity: Severity,
    },
}

impl ErrorBoundaryViolation {
    /// Returns the severity level of this violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

impl std::fmt::Display for ErrorBoundaryViolation {
    /// Formats the violation as a human-readable string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingErrorContext {
                file,
                line,
                error_pattern,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Missing error context: {}:{} - {} ({})",
                    file.display(),
                    line,
                    error_pattern,
                    suggestion
                )
            }
            Self::WrongLayerError {
                file,
                line,
                error_type,
                layer,
                ..
            } => {
                write!(
                    f,
                    "Wrong layer error: {}:{} - {} in {}",
                    file.display(),
                    line,
                    error_type,
                    layer
                )
            }
            Self::LeakedInternalError {
                file,
                line,
                pattern,
                ..
            } => {
                write!(
                    f,
                    "Leaked internal error: {}:{} - {}",
                    file.display(),
                    line,
                    pattern
                )
            }
        }
    }
}

impl Violation for ErrorBoundaryViolation {
    /// Returns the unique identifier for this violation type
    fn id(&self) -> &str {
        match self {
            Self::MissingErrorContext { .. } => "ERR001",
            Self::WrongLayerError { .. } => "ERR002",
            Self::LeakedInternalError { .. } => "ERR003",
        }
    }

    /// Returns the violation category
    fn category(&self) -> ViolationCategory {
        ViolationCategory::ErrorBoundary
    }

    /// Returns the severity level of this violation
    fn severity(&self) -> Severity {
        match self {
            Self::MissingErrorContext { severity, .. }
            | Self::WrongLayerError { severity, .. }
            | Self::LeakedInternalError { severity, .. } => *severity,
        }
    }

    /// Returns the file path where the violation was detected
    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::MissingErrorContext { file, .. }
            | Self::WrongLayerError { file, .. }
            | Self::LeakedInternalError { file, .. } => Some(file),
        }
    }

    /// Returns the line number where the violation was detected
    fn line(&self) -> Option<usize> {
        match self {
            Self::MissingErrorContext { line, .. }
            | Self::WrongLayerError { line, .. }
            | Self::LeakedInternalError { line, .. } => Some(*line),
        }
    }

    /// Returns a suggestion for fixing this violation
    fn suggestion(&self) -> Option<String> {
        match self {
            Self::MissingErrorContext { suggestion, .. } => Some(suggestion.clone()),
            Self::WrongLayerError {
                error_type, layer, ..
            } => Some(format!(
                "Wrap {error_type} in a domain error type instead of using it directly in {layer}"
            )),
            Self::LeakedInternalError { pattern, .. } => Some(format!(
                "Replace {pattern} with a sanitized error response that doesn't expose internal details"
            )),
        }
    }
}

/// Error boundary validator
pub struct ErrorBoundaryValidator {
    config: ValidationConfig,
}

impl ErrorBoundaryValidator {
    /// Creates a new error boundary validator with default configuration
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Creates a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Runs all error boundary validations and returns detected violations
    pub fn validate_all(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_error_context()?);
        violations.extend(self.validate_layer_error_types()?);
        violations.extend(self.validate_leaked_errors()?);
        Ok(violations)
    }

    /// Detects error propagation without context (missing `.context()` or `.map_err()`)
    pub fn validate_error_context(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();

        // Pattern: ? operator without .context() or .with_context()
        // This is a heuristic - we look for lines with ? but no context method
        let question_mark_pattern = Regex::new(r"\?\s*;?\s*$")
            .map_err(|e| ValidationError::InvalidRegex(format!("question mark pattern: {e}")))?;
        let context_pattern = Regex::new(r"\.(context|with_context|map_err|ok_or_else)\s*\(")
            .map_err(|e| ValidationError::InvalidRegex(format!("context pattern: {e}")))?;

        // Files that are likely error boundary crossing points
        let boundary_paths = ["handlers/", "adapters/", "services/"];

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path_str = entry.path().to_string_lossy();

                // Skip test files
                if path_str.contains("/tests/") {
                    continue;
                }

                // Only check boundary files
                let is_boundary = boundary_paths.iter().any(|p| path_str.contains(p));
                if !is_boundary {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for ? without context
                    if question_mark_pattern.is_match(trimmed) && !context_pattern.is_match(trimmed)
                    {
                        // Skip simple Result propagation
                        if trimmed.starts_with("return ") || trimmed.contains("Ok(") {
                            continue;
                        }

                        violations.push(ErrorBoundaryViolation::MissingErrorContext {
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            error_pattern: trimmed.chars().take(60).collect(),
                            suggestion: "Add .context() or .map_err() for better error messages"
                                .to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detects infrastructure error types used in domain layer (layer boundary violation)
    pub fn validate_layer_error_types(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();

        // Infrastructure error types that shouldn't appear in domain
        let infra_errors = [
            (r"std::io::Error", "std::io::Error"),
            (r"reqwest::Error", "reqwest::Error"),
            (r"sqlx::Error", "sqlx::Error"),
            (r"tokio::.*Error", "tokio Error"),
            (r"hyper::Error", "hyper::Error"),
            (r"serde_json::Error", "serde_json::Error"),
        ];

        let compiled_errors: Vec<_> = infra_errors
            .iter()
            .filter_map(|(p, desc)| Regex::new(p).ok().map(|r| (r, *desc)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path_str = entry.path().to_string_lossy();

                // Skip test files
                if path_str.contains("/tests/") {
                    continue;
                }

                // Only check domain layer files (uses directory convention, not hardcoded crate names)
                let is_domain = path_str.contains("/domain/");
                if !is_domain {
                    continue;
                }

                // Skip error definition files
                let file_name = entry
                    .path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if file_name == "error.rs" || file_name.starts_with("error") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for infrastructure error types
                    for (pattern, desc) in &compiled_errors {
                        if pattern.is_match(line) {
                            violations.push(ErrorBoundaryViolation::WrongLayerError {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                error_type: desc.to_string(),
                                layer: "domain".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detects internal error details leaked to API responses (information disclosure)
    pub fn validate_leaked_errors(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();

        // Patterns that indicate internal errors being exposed
        let leak_patterns = [
            (
                r#"format!\s*\(\s*"\{\:?\?\}""#,
                "Debug formatting in response",
            ),
            (
                r"\b(?:err|error|e)\b\.to_string\(\)",
                "Error .to_string() in response",
            ),
            (
                r#"serde_json::json!\s*\(\s*\{\s*"error"\s*:\s*format!"#,
                "Internal error in JSON response",
            ),
        ];

        let compiled_leaks: Vec<_> = leak_patterns
            .iter()
            .filter_map(|(p, desc)| Regex::new(p).ok().map(|r| (r, *desc)))
            .collect();

        // Only check handler files (API boundary)
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path_str = entry.path().to_string_lossy();

                // Skip test files
                if path_str.contains("/tests/") {
                    continue;
                }

                // Only check handler files
                let is_handler =
                    path_str.contains("/handlers/") || path_str.contains("_handler.rs");
                if !is_handler {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for leak patterns
                    for (pattern, desc) in &compiled_leaks {
                        if pattern.is_match(line) {
                            violations.push(ErrorBoundaryViolation::LeakedInternalError {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                pattern: desc.to_string(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl_validator!(
    ErrorBoundaryValidator,
    "error_boundary",
    "Validates error handling patterns across layer boundaries"
);
