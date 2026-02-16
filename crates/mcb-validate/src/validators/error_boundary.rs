//! Error Boundary Validation
//!
//! Validates Clean Architecture error handling patterns:
//! - Layer error wrapping (domain wraps infrastructure errors)
//! - Context preservation across layers
//! - Error type placement (right layer)

use crate::constants::architecture::{
    ARCH_PATH_ADAPTERS, ARCH_PATH_DOMAIN, ARCH_PATH_HANDLERS, ARCH_PATH_SERVICES,
};
use crate::constants::common::{
    CFG_TEST_MARKER, COMMENT_PREFIX, HANDLER_FILE_SUFFIX, SHORT_PREVIEW_LENGTH, TEST_DIR_FRAGMENT,
};
use crate::filters::LanguageId;
use std::path::{Path, PathBuf};

use crate::define_violations;
use crate::pattern_registry::{compile_regex, compile_regex_pairs};
use crate::scan::for_each_scan_file;
use crate::traits::violation::ViolationCategory;
use crate::{Result, Severity, ValidationConfig};

define_violations! {
    dynamic_severity,
    ViolationCategory::ErrorBoundary,
    pub enum ErrorBoundaryViolation {
        /// Error crossing layer without context
        #[violation(
            id = "ERR001",
            severity = Warning,
            message = "Missing error context: {file}:{line} - {error_pattern} ({suggestion})",
            suggestion = "{suggestion}"
        )]
        MissingErrorContext {
            file: PathBuf,
            line: usize,
            error_pattern: String,
            suggestion: String,
            severity: Severity,
        },
        /// Infrastructure error type used in domain layer
        #[violation(
            id = "ERR002",
            severity = Warning,
            message = "Wrong layer error: {file}:{line} - {error_type} in {layer}",
            suggestion = "Wrap {error_type} in a domain error type instead of using it directly in {layer}"
        )]
        WrongLayerError {
            file: PathBuf,
            line: usize,
            error_type: String,
            layer: String,
            severity: Severity,
        },
        /// Internal error details leaked to external API
        #[violation(
            id = "ERR003",
            severity = Error,
            message = "Leaked internal error: {file}:{line} - {pattern}",
            suggestion = "Replace {pattern} with a sanitized error response that doesn't expose internal details"
        )]
        LeakedInternalError {
            file: PathBuf,
            line: usize,
            pattern: String,
            severity: Severity,
        },
    }
}

/// Error boundary validator
pub struct ErrorBoundaryValidator {
    config: ValidationConfig,
}

crate::impl_simple_validator_new!(ErrorBoundaryValidator);

impl ErrorBoundaryValidator {
    /// Runs all error boundary validations and returns detected violations
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_all(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_error_context()?);
        violations.extend(self.validate_layer_error_types()?);
        violations.extend(self.validate_leaked_errors()?);
        Ok(violations)
    }

    fn scan_relevant_lines<FileFilter, LineHandler>(
        &self,
        file_filter: FileFilter,
        mut line_handler: LineHandler,
    ) -> Result<()>
    where
        FileFilter: Fn(&Path, &str) -> bool,
        LineHandler: FnMut(&PathBuf, usize, &str, &str),
    {
        for_each_scan_file(
            &self.config,
            Some(LanguageId::Rust),
            false,
            |entry, _src_dir| {
                let path = &entry.absolute_path;
                let Some(path_str) = path.to_str() else {
                    return Ok(());
                };

                if path_str.contains(TEST_DIR_FRAGMENT) || !file_filter(path, path_str) {
                    return Ok(());
                }

                let content = std::fs::read_to_string(path)?;
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    if trimmed.starts_with(COMMENT_PREFIX) {
                        continue;
                    }

                    if trimmed.contains(CFG_TEST_MARKER) {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    line_handler(path, line_num + 1, line, trimmed);
                }

                Ok(())
            },
        )
    }

    /// Detects error propagation without context (missing `.context()` or `.map_err()`)
    ///
    /// # Errors
    ///
    /// Returns an error if regex compilation or source file reading fails.
    pub fn validate_error_context(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();

        // Pattern: ? operator without .context() or .with_context()
        // This is a heuristic - we look for lines with ? but no context method
        let question_mark_pattern = compile_regex(r"\?\s*;?\s*$")?;
        let context_pattern = compile_regex(r"\.(context|with_context|map_err|ok_or_else)\s*\(")?;

        // Files that are likely error boundary crossing points
        let boundary_paths = [ARCH_PATH_HANDLERS, ARCH_PATH_ADAPTERS, ARCH_PATH_SERVICES];

        self.scan_relevant_lines(
            |_path, path_str| boundary_paths.iter().any(|p| path_str.contains(p)),
            |path, line_num, _line, trimmed| {
                if question_mark_pattern.is_match(trimmed)
                    && !context_pattern.is_match(trimmed)
                    && !trimmed.starts_with("return ")
                    && !trimmed.contains("Ok(")
                {
                    violations.push(ErrorBoundaryViolation::MissingErrorContext {
                        file: path.clone(),
                        line: line_num,
                        error_pattern: trimmed.chars().take(SHORT_PREVIEW_LENGTH).collect(),
                        suggestion: "Add .context() or .map_err() for better error messages"
                            .to_owned(),
                        severity: Severity::Info,
                    });
                }
            },
        )?;

        Ok(violations)
    }

    /// Detects infrastructure error types used in domain layer (layer boundary violation)
    ///
    /// # Errors
    ///
    /// Returns an error if regex compilation or source file reading fails.
    pub fn validate_layer_error_types(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();

        // Infrastructure error types that shouldn't appear in domain
        let infra_errors = [
            ("std::io::Error", "std::io::Error"),
            ("reqwest::Error", "reqwest::Error"),
            ("sqlx::Error", "sqlx::Error"),
            ("tokio::.*Error", "tokio Error"),
            ("hyper::Error", "hyper::Error"),
            ("serde_json::Error", "serde_json::Error"),
        ];

        let compiled_errors = compile_regex_pairs(&infra_errors)?;

        self.scan_relevant_lines(
            |path, path_str| {
                if !path_str.contains(ARCH_PATH_DOMAIN) {
                    return false;
                }
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                file_name != "error.rs" && !file_name.starts_with("error")
            },
            |path, line_num, line, _trimmed| {
                for (pattern, desc) in &compiled_errors {
                    if pattern.is_match(line) {
                        violations.push(ErrorBoundaryViolation::WrongLayerError {
                            file: path.clone(),
                            line: line_num,
                            error_type: desc.to_string(),
                            layer: "domain".to_owned(),
                            severity: Severity::Warning,
                        });
                    }
                }
            },
        )?;

        Ok(violations)
    }

    /// Detects internal error details leaked to API responses (information disclosure)
    ///
    /// # Errors
    ///
    /// Returns an error if regex compilation or source file reading fails.
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

        let compiled_leaks = compile_regex_pairs(&leak_patterns)?;

        self.scan_relevant_lines(
            |_path, path_str| {
                path_str.contains(ARCH_PATH_HANDLERS) || path_str.contains(HANDLER_FILE_SUFFIX)
            },
            |path, line_num, line, _trimmed| {
                for (pattern, desc) in &compiled_leaks {
                    if pattern.is_match(line) {
                        violations.push(ErrorBoundaryViolation::LeakedInternalError {
                            file: path.clone(),
                            line: line_num,
                            pattern: desc.to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            },
        )?;

        Ok(violations)
    }
}

crate::impl_validator!(
    ErrorBoundaryValidator,
    "error_boundary",
    "Validates error handling patterns across layer boundaries"
);
