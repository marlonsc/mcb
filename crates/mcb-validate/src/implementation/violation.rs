//! Implementation Quality Validation
//!
//! Detects false, incomplete, or low-quality implementations:
//! - Empty method bodies (return Ok(()), None, `Vec::new()`)
//! - Hardcoded return values (return true, return 0)
//! - Pass-through wrappers without transformation
//! - Log-only methods (no actual logic)
//! - Default-only trait implementations

use crate::violation_trait::{Violation, ViolationCategory};
use crate::Severity;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Implementation quality violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationViolation {
    /// Method body is empty or returns trivial value
    EmptyMethodBody {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the empty method.
        method_name: String,
        /// The trivial return pattern detected (e.g., "Ok(())").
        pattern: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Method returns hardcoded value bypassing logic
    HardcodedReturnValue {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the method returning a hardcoded value.
        method_name: String,
        /// The hardcoded value being returned.
        return_value: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Wrapper that just delegates without adding value
    PassThroughWrapper {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the struct containing the wrapper.
        struct_name: String,
        /// Name of the wrapper method.
        method_name: String,
        /// The target being delegated to.
        delegated_to: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Method body only contains logging/tracing
    LogOnlyMethod {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the method containing only logging.
        method_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Stub implementation using todo!/unimplemented!
    StubMacro {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the stubbed method.
        method_name: String,
        /// The macro used (todo!, unimplemented!).
        macro_type: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Match arm with empty catch-all
    EmptyCatchAll {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Context of the match arm.
        context: String,
        /// Severity level of the violation.
        severity: Severity,
    },
}

impl ImplementationViolation {
    /// Returns the severity level of the violation.
    pub fn severity(&self) -> Severity {
        match self {
            Self::EmptyMethodBody { severity, .. }
            | Self::HardcodedReturnValue { severity, .. }
            | Self::PassThroughWrapper { severity, .. }
            | Self::LogOnlyMethod { severity, .. }
            | Self::StubMacro { severity, .. }
            | Self::EmptyCatchAll { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for ImplementationViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMethodBody {
                file,
                line,
                method_name,
                pattern,
                ..
            } => {
                write!(
                    f,
                    "Empty method body: {}:{} - {}() returns {}",
                    file.display(),
                    line,
                    method_name,
                    pattern
                )
            }
            Self::HardcodedReturnValue {
                file,
                line,
                method_name,
                return_value,
                ..
            } => {
                write!(
                    f,
                    "Hardcoded return: {}:{} - {}() always returns {}",
                    file.display(),
                    line,
                    method_name,
                    return_value
                )
            }
            Self::PassThroughWrapper {
                file,
                line,
                struct_name,
                method_name,
                delegated_to,
                ..
            } => {
                write!(
                    f,
                    "Pass-through wrapper: {}:{} - {}::{}() only delegates to {}",
                    file.display(),
                    line,
                    struct_name,
                    method_name,
                    delegated_to
                )
            }
            Self::LogOnlyMethod {
                file,
                line,
                method_name,
                ..
            } => {
                write!(
                    f,
                    "Log-only method: {}:{} - {}() only contains logging, no logic",
                    file.display(),
                    line,
                    method_name
                )
            }
            Self::StubMacro {
                file,
                line,
                method_name,
                macro_type,
                ..
            } => {
                write!(
                    f,
                    "Stub implementation: {}:{} - {}() uses {}!()",
                    file.display(),
                    line,
                    method_name,
                    macro_type
                )
            }
            Self::EmptyCatchAll {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "Empty catch-all: {}:{} - match arm '_ => {{}}' silently ignores cases: {}",
                    file.display(),
                    line,
                    context
                )
            }
        }
    }
}

impl Violation for ImplementationViolation {
    fn id(&self) -> &str {
        match self {
            Self::EmptyMethodBody { .. } => "IMPL001",
            Self::HardcodedReturnValue { .. } => "IMPL002",
            Self::PassThroughWrapper { .. } => "IMPL003",
            Self::LogOnlyMethod { .. } => "IMPL004",
            Self::StubMacro { .. } => "IMPL005",
            Self::EmptyCatchAll { .. } => "IMPL006",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Implementation
    }

    fn severity(&self) -> Severity {
        match self {
            Self::EmptyMethodBody { severity, .. }
            | Self::HardcodedReturnValue { severity, .. }
            | Self::PassThroughWrapper { severity, .. }
            | Self::LogOnlyMethod { severity, .. }
            | Self::StubMacro { severity, .. }
            | Self::EmptyCatchAll { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::EmptyMethodBody { file, .. }
            | Self::HardcodedReturnValue { file, .. }
            | Self::PassThroughWrapper { file, .. }
            | Self::LogOnlyMethod { file, .. }
            | Self::StubMacro { file, .. }
            | Self::EmptyCatchAll { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::EmptyMethodBody { line, .. }
            | Self::HardcodedReturnValue { line, .. }
            | Self::PassThroughWrapper { line, .. }
            | Self::LogOnlyMethod { line, .. }
            | Self::StubMacro { line, .. }
            | Self::EmptyCatchAll { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::EmptyMethodBody { pattern, .. } => Some(format!(
                "Replace trivial return '{pattern}' with actual implementation logic"
            )),
            Self::HardcodedReturnValue { return_value, .. } => Some(format!(
                "Replace hardcoded '{return_value}' with computed value based on actual logic"
            )),
            Self::PassThroughWrapper { delegated_to, .. } => Some(format!(
                "Add value to this wrapper or consider removing it if '{delegated_to}' delegation is sufficient"
            )),
            Self::LogOnlyMethod { .. } => Some(
                "Add actual business logic; logging alone does not constitute implementation"
                    .to_string(),
            ),
            Self::StubMacro { macro_type, .. } => Some(format!(
                "Replace {macro_type}!() with actual implementation"
            )),
            Self::EmptyCatchAll { .. } => {
                Some("Handle the catch-all case explicitly or log unhandled variants".to_string())
            }
        }
    }
}
