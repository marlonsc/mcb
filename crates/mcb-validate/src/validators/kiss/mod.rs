//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! KISS Principle Validation
//!
//! Validates code simplicity by detecting overly complex structures:
//! - Structs with too many fields
//! - Functions with too many parameters
//! - Overly complex builders
//! - Deep nesting
//! - Long functions

mod checks;
mod counting;

use std::path::PathBuf;

use crate::config::KISSRulesConfig;
use crate::thresholds::thresholds;
use crate::{Severity, ValidationConfig};
use mcb_domain::ports::validation::ViolationCategory;

crate::define_validator! {
    name: "kiss",
    description: "Validates KISS principle (Keep It Simple, Stupid)",


    /// Validates code against KISS (Keep It Simple, Stupid) principles.
    ///
    /// Checks struct field counts, function parameter counts, builder complexity,
    /// nesting depth, and function length against configurable thresholds.
    pub struct KissValidator {
        /// Configuration for validation scans
        pub(crate) config: ValidationConfig,
        /// KISS-specific rule configuration
        pub(crate) rules: KISSRulesConfig,
        /// Maximum allowed fields per struct
        pub(crate) max_struct_fields: usize,
        /// Maximum allowed parameters per function
        pub(crate) max_function_params: usize,
        /// Maximum allowed optional fields per builder
        pub(crate) max_builder_fields: usize,
        /// Maximum allowed nesting depth
        pub(crate) max_nesting_depth: usize,
        /// Maximum allowed lines per function
        pub(crate) max_function_lines: usize,
    }

    violations: dynamic_severity, ViolationCategory::Kiss,
    pub enum KissViolation {
        /// Struct has too many fields, violating simplicity.
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
        /// Function has too many parameters, suggesting a need for a config struct.
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
        /// Builder struct has too many optional fields.
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
        /// Code block has excessive nesting depth.
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
        /// Function body exceeds maximum allowed line count.
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

    checks: [
        struct_fields => Self::validate_struct_fields,
        function_params => Self::validate_function_params,
        builder_complexity => Self::validate_builder_complexity,
        nesting_depth => Self::validate_nesting_depth,
        function_length => Self::validate_function_length
    ],
    enabled = |s: &Self| s.rules.enabled
}

crate::impl_rules_validator_new!(KissValidator, kiss);

impl KissValidator {
    /// Creates a new KISS validator with explicit configuration and rules.
    #[must_use]
    pub fn with_config(config: ValidationConfig, rules: &KISSRulesConfig) -> Self {
        let t = thresholds();
        Self {
            config,
            rules: rules.clone(),
            max_struct_fields: t.max_struct_fields,
            max_function_params: t.max_function_params,
            max_builder_fields: t.max_builder_fields,
            max_nesting_depth: t.max_nesting_depth,
            max_function_lines: t.max_function_lines,
        }
    }

    /// Overrides the maximum allowed struct fields threshold.
    #[must_use]
    pub fn with_max_struct_fields(mut self, max: usize) -> Self {
        self.max_struct_fields = max;
        self
    }

    /// Overrides the maximum allowed function parameters threshold.
    #[must_use]
    pub fn with_max_function_params(mut self, max: usize) -> Self {
        self.max_function_params = max;
        self
    }
}

#[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
static VALIDATOR_ENTRY: mcb_domain::registry::validation::ValidatorEntry =
    mcb_domain::registry::validation::ValidatorEntry {
        name: "kiss",
        description: "Validates KISS principle (Keep It Simple, Stupid)",
        build: |root| {
            Ok(Box::new(KissValidator::new(root))
                as Box<dyn mcb_domain::ports::validation::Validator>)
        },
    };
