//! Async Pattern Validation
//!
//! Detects async-specific anti-patterns based on Tokio documentation:
//! - Blocking in async (`std::thread::sleep`, `std::sync::Mutex` in async)
//! - `block_on()` in async context
//! - Spawn patterns (missing `JoinHandle` handling)
//! - Wrong mutex types in async code

mod block_on;
mod blocking;
mod mutex;
mod spawn;
mod violation;

use std::path::PathBuf;

use crate::{Result, ValidationConfig};

pub use self::violation::AsyncViolation;
use block_on::validate_block_on_usage;
use blocking::validate_blocking_in_async;
use mutex::validate_mutex_types;
use spawn::validate_spawn_patterns;

/// Async pattern validator
pub struct AsyncPatternValidator {
    config: ValidationConfig,
}

impl AsyncPatternValidator {
    /// Create a new async pattern validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration
    #[must_use]
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all async validations
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
