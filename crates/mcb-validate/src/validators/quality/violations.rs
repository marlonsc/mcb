use crate::define_violations;
use crate::{Severity, traits::violation::ViolationCategory};
use std::path::PathBuf;

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
