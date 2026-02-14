//! Implementation Quality Validation
//!
//! Detects false, incomplete, or low-quality implementations:
//! - Empty method bodies (return Ok(()), None, `Vec::new()`)
//! - Hardcoded return values (return true, return 0)
//! - Pass-through wrappers without transformation
//! - Log-only methods (no actual logic)
//! - Default-only trait implementations

use std::path::PathBuf;

use crate::Severity;
use crate::violation_trait::{Violation, ViolationCategory};

define_violations! {
    no_display,
    dynamic_severity,
    ViolationCategory::Implementation,
    pub enum ImplementationViolation {
        /// Method body is empty or returns trivial value
        #[violation(
            id = "IMPL001",
            severity = Warning,
            suggestion = "Replace trivial return '{pattern}' with actual implementation logic"
        )]
        EmptyMethodBody {
            file: PathBuf,
            line: usize,
            method_name: String,
            pattern: String,
            severity: Severity,
        },
        /// Method returns hardcoded value bypassing logic
        #[violation(
            id = "IMPL002",
            severity = Warning,
            suggestion = "Replace hardcoded '{return_value}' with computed value based on actual logic"
        )]
        HardcodedReturnValue {
            file: PathBuf,
            line: usize,
            method_name: String,
            return_value: String,
            severity: Severity,
        },
        /// Wrapper that just delegates without adding value
        #[violation(
            id = "IMPL003",
            severity = Info,
            suggestion = "Add value to this wrapper or consider removing it if '{delegated_to}' delegation is sufficient"
        )]
        PassThroughWrapper {
            file: PathBuf,
            line: usize,
            struct_name: String,
            method_name: String,
            delegated_to: String,
            severity: Severity,
        },
        /// Method body only contains logging/tracing
        #[violation(
            id = "IMPL004",
            severity = Warning,
            suggestion = "Add actual business logic; logging alone does not constitute implementation"
        )]
        LogOnlyMethod {
            file: PathBuf,
            line: usize,
            method_name: String,
            severity: Severity,
        },
        /// Stub implementation using todo!/unimplemented!
        #[violation(
            id = "IMPL005",
            severity = Warning,
            suggestion = "Replace {macro_type}!() with actual implementation"
        )]
        StubMacro {
            file: PathBuf,
            line: usize,
            method_name: String,
            macro_type: String,
            severity: Severity,
        },
        /// Match arm with empty catch-all
        #[violation(
            id = "IMPL006",
            severity = Warning,
            suggestion = "Handle the catch-all case explicitly or log unhandled variants"
        )]
        EmptyCatchAll {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
    }
}

impl ImplementationViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
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
