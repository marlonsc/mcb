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
use crate::define_violations;
use crate::traits::violation::ViolationCategory;

define_violations! {
    dynamic_severity,
    ViolationCategory::Implementation,
    pub enum ImplementationViolation {
        /// Method body is empty or returns trivial value
        #[violation(
            id = "IMPL001",
            severity = Warning,
            message = "Empty method body: {file}:{line} - {method_name}() returns {pattern}",
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
            message = "Hardcoded return: {file}:{line} - {method_name}() always returns {return_value}",
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
            message = "Pass-through wrapper: {file}:{line} - {struct_name}::{method_name}() only delegates to {delegated_to}",
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
            message = "Log-only method: {file}:{line} - {method_name}() only contains logging, no logic",
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
            message = "Stub implementation: {file}:{line} - {method_name}() uses {macro_type}!()",
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
            message = "Empty catch-all: {file}:{line} - match arm '_ => {}' silently ignores cases: {context}",
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
