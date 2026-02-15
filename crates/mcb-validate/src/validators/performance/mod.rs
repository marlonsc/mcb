//! Performance Pattern Validation
//!
//! This module provides the `PerformanceValidator` which identifies common performance
//! anti-patterns in Rust code. It focuses on identifying clone abuse, unnecessary
//! allocations in loops, and suboptimal synchronization patterns.
//!
//!
//! Detects performance anti-patterns that PMAT and Clippy might miss:
//! - Clone abuse (redundant clones, clones in loops)
//! - Allocation patterns (Vec/String in loops)
//! - Arc/Mutex overuse
//! - Inefficient iterator patterns

pub mod constants;
mod loop_checks;
mod loops;
mod pattern_checks;
mod violation;

use std::path::PathBuf;

use crate::config::PerformanceRulesConfig;
use crate::{Result, ValidationConfig};

pub use self::violation::PerformanceViolation;
use loop_checks::{validate_allocation_in_loops, validate_clone_in_loops};
use pattern_checks::{
    validate_arc_mutex_overuse, validate_inefficient_iterators, validate_inefficient_strings,
};

/// Performance pattern validator
pub struct PerformanceValidator {
    pub(crate) config: ValidationConfig,
    pub(crate) rules: PerformanceRulesConfig,
}

impl PerformanceValidator {
    /// Create a new performance validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.performance)
    }

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
