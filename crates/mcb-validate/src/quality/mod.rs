use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::thresholds::thresholds;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig, define_violations};

mod comments;
mod dead_code;
mod metrics;
mod panic;
mod unwrap;
mod utils;

pub struct QualityValidator {
    pub(crate) config: ValidationConfig,
    pub(crate) max_file_lines: usize,
    pub(crate) excluded_paths: Vec<String>,
}

impl QualityValidator {
    /// Creates a new instance of the quality validator for the given workspace.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Creates a new validator instance using a provided configuration.
    pub fn with_config(config: ValidationConfig) -> Self {
        // Load file configuration to get quality rules
        let file_config = crate::config::FileConfig::load(&config.workspace_root);
        Self {
            config,
            max_file_lines: thresholds().max_file_lines,
            excluded_paths: file_config.rules.quality.excluded_paths,
        }
    }

    /// Configures the maximum allowed lines per file.
    #[must_use]
    pub fn with_max_file_lines(mut self, max: usize) -> Self {
        self.max_file_lines = max;
        self
    }

    /// Executes all configured quality checks and returns any violations found.
    pub fn validate_all(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        violations.extend(unwrap::validate(self)?);
        violations.extend(panic::validate(self)?);
        violations.extend(metrics::validate(self)?);
        violations.extend(comments::validate(self)?);
        violations.extend(dead_code::validate(self)?);
        Ok(violations)
    }
}

define_violations! {
    dynamic_severity,
    ViolationCategory::Quality,
    pub enum QualityViolation {
        /// Indicates usage of `unwrap()` in production code, which poses a panic risk.
        #[violation(
            id = "QUAL001",
            severity = Warning,
            message = "unwrap() in production: {file}:{line} - {context}",
            suggestion = "Use `?` operator or handle the error explicitly"
        )]
        UnwrapInProduction {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
        /// Indicates usage of `expect()` in production code, which poses a panic risk.
        #[violation(
            id = "QUAL002",
            severity = Warning,
            message = "expect() in production: {file}:{line} - {context}",
            suggestion = "Use `?` operator or handle the error explicitly"
        )]
        ExpectInProduction {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
        /// Indicates usage of `panic!()` macro in production code.
        #[violation(
            id = "QUAL003",
            severity = Warning,
            message = "panic!() in production: {file}:{line} - {context}",
            suggestion = "Return an error instead of panicking"
        )]
        PanicInProduction {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
        /// Indicates a file that exceeds the maximum allowed line count.
        #[violation(
            id = "QUAL004",
            severity = Warning,
            message = "File too large: {file} has {lines} lines (max: {max_allowed})",
            suggestion = "Split file into smaller modules (max {max_allowed} lines)"
        )]
        FileTooLarge {
            file: PathBuf,
            lines: usize,
            max_allowed: usize,
            severity: Severity,
        },
        /// Indicates presence of pending task comments (tracked via `PENDING_LABEL_*` constants).
        #[violation(
            id = "QUAL005",
            severity = Info,
            message = "Pending: {file}:{line} - {content}",
            suggestion = "Address the pending comment or create an issue to track it"
        )]
        TodoComment {
            file: PathBuf,
            line: usize,
            content: String,
            severity: Severity,
        },
        /// Indicates usage of `allow(dead_code)` attribute, which is not permitted.
        #[violation(
            id = "QUAL020",
            severity = Warning,
            message = "{file}:{line} - {item_name} (allow(dead_code) not permitted)",
            suggestion = "Remove #[allow(dead_code)] and fix or remove the dead code; justifications are not permitted"
        )]
        DeadCodeAllowNotPermitted {
            file: PathBuf,
            line: usize,
            item_name: String,
            severity: Severity,
        },
        /// Indicates a struct field that is defined but never used.
        #[violation(
            id = "QUAL021",
            severity = Warning,
            message = "Unused struct field: {file}:{line} - Field '{field_name}' in struct '{struct_name}' is unused",
            suggestion = "Remove the unused field or document why it's kept (e.g., for serialization format versioning)"
        )]
        UnusedStructField {
            file: PathBuf,
            line: usize,
            struct_name: String,
            field_name: String,
            severity: Severity,
        },
        /// Indicates a function that is marked as dead code and appears uncalled.
        #[violation(
            id = "QUAL022",
            severity = Warning,
            message = "Dead function: {file}:{line} - Function '{function_name}' marked as dead but appears uncalled",
            suggestion = "Remove the dead function or connect it to the public API if it's intended for future use"
        )]
        DeadFunctionUncalled {
            file: PathBuf,
            line: usize,
            function_name: String,
            severity: Severity,
        },
    }
}

impl_validator!(
    QualityValidator,
    "quality",
    "Validates code quality (no unwrap/expect)"
);
