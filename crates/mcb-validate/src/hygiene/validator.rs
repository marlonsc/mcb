//! # Test Organization and Quality Validation
//!
//! This module provides the infrastructure for validating test hygiene and quality
//! across the codebase. It ensures that tests are organized according to the
//! project's standards (e.g., unit tests in `tests/unit/`) and that they maintain
//! high quality (e.g., meaningful assertions, no raw unwraps).
//!
//! This validator delegates specific checks to specialized modules in the `hygiene` directory.

use std::path::PathBuf;

use crate::{Result, ValidationConfig};

/// Test hygiene violation types
use super::violation::HygieneViolation;

/// Validates test organization and quality across a codebase.
///
/// Checks for:
/// - Inline test modules in src/ (should be in tests/)
/// - Test file naming conventions
/// - Test function naming conventions
/// - Test quality (assertions, trivial tests, etc.)
pub struct HygieneValidator {
    config: ValidationConfig,
}

impl HygieneValidator {
    /// Create a new test validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Creates a validator with custom configuration for multi-directory support.
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Runs all test organization validations and returns violations found.
    pub fn validate_all(&self) -> Result<Vec<HygieneViolation>> {
        let mut violations = Vec::new();
        violations.extend(super::inline_tests::validate_no_inline_tests(&self.config)?);
        violations.extend(super::directory::validate_test_directory_structure(
            &self.config,
        )?);
        violations.extend(super::naming::validate_test_naming(&self.config)?);
        violations.extend(super::function_naming::validate_test_function_naming(
            &self.config,
        )?);
        violations.extend(super::quality::validate_test_quality(&self.config)?);
        Ok(violations)
    }

    // Deprecated methods removed.
}

impl_validator!(
    HygieneValidator,
    "hygiene",
    "Validates test hygiene and quality"
);
