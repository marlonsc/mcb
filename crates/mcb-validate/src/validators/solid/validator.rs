//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! Solid validator implementation
//!
//! This module provides the `SolidValidator` which enforces SOLID design principles:
//! - Single Responsibility Principle (SRP)
//! - Open/Closed Principle (OCP)
//! - Liskov Substitution Principle (LSP)
//! - Interface Segregation Principle (ISP)
//! - Dependency Inversion Principle (DIP)

use super::violation::SolidViolation;
use super::{isp, lsp, ocp, srp};
use crate::thresholds::thresholds;
use crate::{Result, ValidationConfig};

crate::create_validator!(
    SolidValidator,
    "solid",
    "Validates SOLID principles",
    SolidViolation,
    [
        Self::validate_srp,
        Self::validate_ocp,
        Self::validate_isp,
        Self::validate_lsp,
        Self::validate_string_dispatch,
        Self::validate_impl_method_count,
    ]
);

impl SolidValidator {
    /// SRP: Check for structs/impls that are too large
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_srp(config: &ValidationConfig) -> Result<Vec<SolidViolation>> {
        srp::validate_srp(config, thresholds().max_struct_lines)
    }

    /// OCP: Check for excessive match statements
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_ocp(config: &ValidationConfig) -> Result<Vec<SolidViolation>> {
        ocp::validate_ocp(config, thresholds().max_match_arms)
    }

    /// OCP: Check for string-based type dispatch
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_string_dispatch(config: &ValidationConfig) -> Result<Vec<SolidViolation>> {
        ocp::validate_string_dispatch(config)
    }

    /// ISP: Check for traits with too many methods
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_isp(config: &ValidationConfig) -> Result<Vec<SolidViolation>> {
        isp::validate_isp(config, thresholds().max_trait_methods)
    }

    /// LSP: Check for partial trait implementations (panic!/todo! in trait methods).
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_lsp(config: &ValidationConfig) -> Result<Vec<SolidViolation>> {
        lsp::validate_lsp(config)
    }

    /// SRP: Check for impl blocks with too many methods
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_impl_method_count(config: &ValidationConfig) -> Result<Vec<SolidViolation>> {
        srp::validate_impl_method_count(config, thresholds().max_impl_methods)
    }
}
