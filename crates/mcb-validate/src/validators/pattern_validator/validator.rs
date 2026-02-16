use crate::config::PatternRulesConfig;
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_crate_file;
use crate::{Result, ValidationConfig};

use super::async_check::check_async_traits;
use super::di::check_arc_usage;
use super::result_check::check_result_types;
use super::violation::PatternViolation;

/// Validates code patterns across the workspace.
///
/// checks:
/// - Async trait usage
/// - Dependency Injection patterns (Arc<dyn T>)
/// - Result type usage
pub struct PatternValidator {
    config: ValidationConfig,
    rules: PatternRulesConfig,
}

crate::impl_rules_validator_new!(PatternValidator, patterns);

impl PatternValidator {
    /// Creates a new validator with the given configuration.
    #[must_use]
    pub fn with_config(config: ValidationConfig, rules: &PatternRulesConfig) -> Self {
        Self {
            config,
            rules: rules.clone(),
        }
    }

    /// Runs all pattern validations.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_all(&self) -> Result<Vec<PatternViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }

        let mut violations = Vec::new();

        // Compile regex for DI check once
        let arc_pattern = compile_regex("Arc<([A-Z][a-zA-Z0-9_]*)>")?;

        for_each_crate_file(
            &self.config,
            Some(LanguageId::Rust),
            |entry, _src_dir, _crate_name| {
                let path = &entry.absolute_path;
                let content = std::fs::read_to_string(path)?;

                // Async Trait Check
                violations.extend(check_async_traits(path, &content)?);

                // DI Usage Check
                violations.extend(check_arc_usage(
                    path,
                    &content,
                    &arc_pattern,
                    &self.rules.allowed_concrete_types,
                    &self.rules.provider_trait_suffixes,
                ));

                // Result Type Check
                violations.extend(check_result_types(path, &content)?);

                Ok(())
            },
        )?;

        Ok(violations)
    }
}

crate::impl_validator!(
    PatternValidator,
    "pattern",
    "Validates code patterns (DI, Async, Result types)"
);
