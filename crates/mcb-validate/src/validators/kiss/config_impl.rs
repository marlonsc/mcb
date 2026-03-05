//! KISS validator configuration and registration.

use crate::ValidationConfig;
use crate::config::KISSRulesConfig;
use crate::thresholds::thresholds;

use super::KissValidator;

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

mcb_domain::register_validator!(
    mcb_utils::constants::validate::VALIDATOR_KISS,
    "Validates KISS principle (Keep It Simple, Stupid)",
    |root| {
        Ok(Box::new(KissValidator::new(root)) as Box<dyn mcb_domain::ports::validation::Validator>)
    }
);
