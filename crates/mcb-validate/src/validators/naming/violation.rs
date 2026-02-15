use std::path::PathBuf;

use crate::traits::violation::{Severity, Violation, ViolationCategory};

define_violations! {
    dynamic_severity,
    ViolationCategory::Naming,
    pub enum NamingViolation {
        /// Bad struct/enum/trait name (should be CamelCase)
        #[violation(
            id = "NAME001",
            severity = Warning,
            message = "Bad type name: {file}:{line} - {name} (expected {expected_case})",
            suggestion = "Rename '{name}' to {expected_case} format"
        )]
        BadTypeName {
            file: PathBuf,
            line: usize,
            name: String,
            expected_case: String,
            severity: Severity,
        },
        /// Bad function/method name (should be `snake_case`)
        #[violation(
            id = "NAME002",
            severity = Warning,
            message = "Bad function name: {file}:{line} - {name} (expected {expected_case})",
            suggestion = "Rename '{name}' to {expected_case} format"
        )]
        BadFunctionName {
            file: PathBuf,
            line: usize,
            name: String,
            expected_case: String,
            severity: Severity,
        },
        /// Bad constant name (should be `SCREAMING_SNAKE_CASE`)
        #[violation(
            id = "NAME003",
            severity = Warning,
            message = "Bad constant name: {file}:{line} - {name} (expected {expected_case})",
            suggestion = "Rename '{name}' to {expected_case} format"
        )]
        BadConstantName {
            file: PathBuf,
            line: usize,
            name: String,
            expected_case: String,
            severity: Severity,
        },
        /// Bad module/file name (should be `snake_case`)
        #[violation(
            id = "NAME004",
            severity = Warning,
            message = "Bad module name: {path} (expected {expected_case})",
            suggestion = "Rename module/file to {expected_case} format"
        )]
        BadModuleName {
            path: PathBuf,
            expected_case: String,
            severity: Severity,
        },

        /// File suffix doesn't match component type
        #[violation(
            id = "NAME005",
            severity = Warning,
            message = "Bad file suffix: {path} ({component_type}) has suffix '{current_suffix}' but expected '{expected_suffix}'",
            suggestion = "Add '{expected_suffix}' suffix to file name"
        )]
        BadFileSuffix {
            path: PathBuf,
            component_type: String,
            current_suffix: String,
            expected_suffix: String,
            severity: Severity,
        },

        /// File name doesn't follow CA naming convention
        #[violation(
            id = "NAME006",
            severity = Warning,
            message = "CA naming: {path} ({detected_type}): {issue} - {suggestion}",
            suggestion = "{suggestion}"
        )]
        BadCaNaming {
            path: PathBuf,
            detected_type: String,
            issue: String,
            suggestion: String,
            severity: Severity,
        },
    }
}

impl NamingViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    #[must_use]
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}
