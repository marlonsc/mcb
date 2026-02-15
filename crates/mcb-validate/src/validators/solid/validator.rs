//! Solid validator implementation
//!
//! This module provides the `SolidValidator` which enforces SOLID design principles:
//! - Single Responsibility Principle (SRP)
//! - Open/Closed Principle (OCP)
//! - Liskov Substitution Principle (LSP)
//! - Interface Segregation Principle (ISP)
//! - Dependency Inversion Principle (DIP)

use std::path::PathBuf;

use super::violation::SolidViolation;
use super::{isp, lsp, ocp, srp};
use crate::thresholds::thresholds;
use crate::{Result, ValidationConfig};

/// SOLID principles validator
pub struct SolidValidator {
    /// Configuration for validation scans
    config: ValidationConfig,
    /// Maximum number of methods allowed in a trait
    max_trait_methods: usize,
    /// Maximum number of lines allowed in a struct definition
    max_struct_lines: usize,
    /// Maximum number of arms allowed in a match expression
    max_match_arms: usize,
    /// Maximum number of methods allowed in an impl block
    max_impl_methods: usize,
}

impl SolidValidator {
    /// Create a new SOLID validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    #[must_use]
    pub fn with_config(config: ValidationConfig) -> Self {
        let t = thresholds();
        Self {
            config,
            max_trait_methods: t.max_trait_methods,
            max_struct_lines: t.max_struct_lines,
            max_match_arms: t.max_match_arms,
            max_impl_methods: t.max_impl_methods,
        }
    }

    /// Run all SOLID validations
    ///
    /// # Errors
    /// Returns an error if file traversal or pattern compilation fails.
    pub fn validate_all(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_srp()?);
        violations.extend(self.validate_ocp()?);
        violations.extend(self.validate_isp()?);
        violations.extend(self.validate_lsp()?);
        violations.extend(self.validate_string_dispatch()?);
        violations.extend(self.validate_impl_method_count()?);
        Ok(violations)
    }

    /// SRP: Check for structs/impls that are too large
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_srp(&self) -> Result<Vec<SolidViolation>> {
        srp::validate_srp(&self.config, self.max_struct_lines)
    }

    /// OCP: Check for excessive match statements
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_ocp(&self) -> Result<Vec<SolidViolation>> {
        ocp::validate_ocp(&self.config, self.max_match_arms)
    }

    /// OCP: Check for string-based type dispatch
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_string_dispatch(&self) -> Result<Vec<SolidViolation>> {
        ocp::validate_string_dispatch(&self.config)
    }

    /// ISP: Check for traits with too many methods
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_isp(&self) -> Result<Vec<SolidViolation>> {
        isp::validate_isp(&self.config, self.max_trait_methods)
    }

    /// LSP: Check for partial trait implementations (panic!/todo! in trait methods).
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_lsp(&self) -> Result<Vec<SolidViolation>> {
        lsp::validate_lsp(&self.config)
    }

    /// SRP: Check for impl blocks with too many methods
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_impl_method_count(&self) -> Result<Vec<SolidViolation>> {
        srp::validate_impl_method_count(&self.config, self.max_impl_methods)
    }
}

crate::impl_validator!(SolidValidator, "solid", "Validates SOLID principles");
