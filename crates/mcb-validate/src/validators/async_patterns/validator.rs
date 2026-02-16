use crate::{Result, ValidationConfig};

use super::block_on::validate_block_on_usage;
use super::blocking::validate_blocking_in_async;
use super::mutex::validate_mutex_types;
use super::spawn::validate_spawn_patterns;
use super::violation::AsyncViolation;

/// Async pattern validator
pub struct AsyncPatternValidator {
    config: ValidationConfig,
}

crate::impl_simple_validator_new!(AsyncPatternValidator);

impl AsyncPatternValidator {
    /// Run all async validations
    ///
    /// # Errors
    ///
    /// Returns an error if any validation check fails.
    pub fn validate_all(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();
        violations.extend(validate_blocking_in_async(&self.config)?);
        violations.extend(validate_block_on_usage(&self.config)?);
        violations.extend(validate_mutex_types(&self.config)?);
        violations.extend(validate_spawn_patterns(&self.config)?);
        Ok(violations)
    }
}

crate::impl_validator!(
    AsyncPatternValidator,
    "async_patterns",
    "Validates async patterns (blocking calls, mutex types, spawn patterns)"
);
