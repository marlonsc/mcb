//! KISS Principle Validation
//!
//! Validates code simplicity by detecting overly complex structures:
//! - Structs with too many fields
//! - Functions with too many parameters
//! - Overly complex builders
//! - Deep nesting
//! - Long functions

use std::path::PathBuf;

use crate::config::KISSRulesConfig;
use crate::thresholds::thresholds;
use crate::{Result, ValidationConfig};

mod checks;
mod helpers;
mod violations;

pub use self::violations::KissViolation;

/// Validates code against KISS (Keep It Simple, Stupid) principles.
///
/// Checks struct field counts, function parameter counts, builder complexity,
/// nesting depth, and function length against configurable thresholds.
pub struct KissValidator {
    /// Configuration for validation scans
    config: ValidationConfig,
    /// KISS-specific rule configuration
    rules: KISSRulesConfig,
    /// Maximum allowed fields per struct
    max_struct_fields: usize,
    /// Maximum allowed parameters per function
    max_function_params: usize,
    /// Maximum allowed optional fields per builder
    max_builder_fields: usize,
    /// Maximum allowed nesting depth
    max_nesting_depth: usize,
    /// Maximum allowed lines per function
    max_function_lines: usize,
}

impl KissValidator {
    /// Creates a new KISS validator with configuration loaded from the workspace root.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.kiss)
    }

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

    /// Runs all KISS validations and returns detected violations.
    pub fn validate_all(&self) -> Result<Vec<KissViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();
        violations.extend(self.validate_struct_fields()?);
        violations.extend(self.validate_function_params()?);
        violations.extend(self.validate_builder_complexity()?);
        violations.extend(self.validate_nesting_depth()?);
        violations.extend(self.validate_function_length()?);
        Ok(violations)
    }
}

crate::impl_validator!(
    KissValidator,
    "kiss",
    "Validates KISS principle (Keep It Simple, Stupid)"
);
