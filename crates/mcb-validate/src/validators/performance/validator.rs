//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::config::PerformanceRulesConfig;
use crate::{Result, ValidationConfig};

use super::loop_checks::{validate_allocation_in_loops, validate_clone_in_loops};
use super::pattern_checks::{
    validate_arc_mutex_overuse, validate_inefficient_iterators, validate_inefficient_strings,
};
use super::violation::PerformanceViolation;

/// Performance pattern validator
pub struct PerformanceValidator {
    pub(crate) config: ValidationConfig,
    pub(crate) rules: PerformanceRulesConfig,
}

crate::impl_rules_validator_new!(PerformanceValidator, performance);

impl PerformanceValidator {
    /// Create a validator with custom configuration
    #[must_use]
    pub fn with_config(config: ValidationConfig, rules: &PerformanceRulesConfig) -> Self {
        Self {
            config,
            rules: rules.clone(),
        }
    }

    /// Run all performance validations
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning, reading, or regex compilation fails.
    pub fn validate_all(&self) -> Result<Vec<PerformanceViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();
        violations.extend(validate_clone_in_loops(self)?);
        violations.extend(validate_allocation_in_loops(self)?);
        violations.extend(validate_arc_mutex_overuse(self)?);
        violations.extend(validate_inefficient_iterators(self)?);
        violations.extend(validate_inefficient_strings(self)?);
        Ok(violations)
    }

    /// Check if a crate should be skipped based on configuration.
    pub(crate) fn should_skip_crate(&self, src_dir: &std::path::Path) -> bool {
        let Some(path_str) = src_dir.to_str() else {
            return false;
        };
        self.rules
            .excluded_crates
            .iter()
            .any(|excluded| path_str.contains(excluded))
    }
}

crate::impl_validator!(
    PerformanceValidator,
    "performance",
    "Validates performance patterns (clones, allocations, Arc/Mutex usage)"
);
