use std::path::PathBuf;

use crate::Severity;
use crate::traits::violation::{Violation, ViolationCategory};

crate::define_violations! {
    dynamic_severity,
    ViolationCategory::Kiss,
    pub enum KissViolation {
        #[violation(
            id = "KISS001",
            severity = Warning,
            message = "KISS: Struct {struct_name} has too many fields: {file}:{line} ({field_count} fields, max: {max_allowed})",
            suggestion = "Split '{struct_name}' into smaller structs or use composition. {field_count} fields exceeds the maximum of {max_allowed}."
        )]
        StructTooManyFields {
            file: PathBuf,
            line: usize,
            struct_name: String,
            field_count: usize,
            max_allowed: usize,
            severity: Severity,
        },
        #[violation(
            id = "KISS002",
            severity = Warning,
            message = "KISS: Function {function_name} has too many parameters: {file}:{line} ({param_count} params, max: {max_allowed})",
            suggestion = "Refactor '{function_name}' to use a config/options struct instead of {param_count} parameters. Maximum allowed is {max_allowed}."
        )]
        FunctionTooManyParams {
            file: PathBuf,
            line: usize,
            function_name: String,
            param_count: usize,
            max_allowed: usize,
            severity: Severity,
        },
        #[violation(
            id = "KISS003",
            severity = Warning,
            message = "KISS: Builder {builder_name} is too complex: {file}:{line} ({optional_field_count} optional fields, max: {max_allowed})",
            suggestion = "Split '{builder_name}' into smaller builders or use builder composition. {optional_field_count} optional fields exceeds the maximum of {max_allowed}."
        )]
        BuilderTooComplex {
            file: PathBuf,
            line: usize,
            builder_name: String,
            optional_field_count: usize,
            max_allowed: usize,
            severity: Severity,
        },
        #[violation(
            id = "KISS004",
            severity = Warning,
            message = "KISS: Deep nesting at {file}:{line} ({nesting_level} levels, max: {max_allowed}) - {context}",
            suggestion = "Extract nested logic into separate functions using early returns or guard clauses. Nesting depth {nesting_level} exceeds the maximum of {max_allowed}."
        )]
        DeepNesting {
            file: PathBuf,
            line: usize,
            nesting_level: usize,
            max_allowed: usize,
            context: String,
            severity: Severity,
        },
        #[violation(
            id = "KISS005",
            severity = Warning,
            message = "KISS: Function {function_name} is too long: {file}:{line} ({line_count} lines, max: {max_allowed})",
            suggestion = "Break '{function_name}' into smaller, focused functions. {line_count} lines exceeds the maximum of {max_allowed}."
        )]
        FunctionTooLong {
            file: PathBuf,
            line: usize,
            function_name: String,
            line_count: usize,
            max_allowed: usize,
            severity: Severity,
        },
    }
}

impl KissViolation {
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}
