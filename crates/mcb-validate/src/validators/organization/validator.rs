//! Organization validator implementation

use std::path::PathBuf;

use super::violation::OrganizationViolation;
use super::{
    domain_purity::validate_domain_traits_only, duplicate_strings::validate_duplicate_strings,
    file_placement::validate_file_placement, layer_violations::validate_layer_violations,
    magic_numbers::validate_magic_numbers, strict_directory::validate_strict_directory,
    trait_placement::validate_trait_placement,
};
use crate::{Result, ValidationConfig};

/// Validates the structural organization and architectural compliance of the codebase.
pub struct OrganizationValidator {
    config: ValidationConfig,
}

impl OrganizationValidator {
    /// Initializes a new organization validator for the specified workspace root.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Initializes a new organization validator with a custom configuration.
    #[must_use]
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Executes all organization validation checks and returns the aggregated violations.
    ///
    /// # Errors
    ///
    /// Returns an error if any sub-validation encounters a file system or parsing error.
    pub fn validate_all(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_magic_numbers()?);
        violations.extend(self.validate_duplicate_strings()?);
        violations.extend(self.validate_file_placement()?);
        violations.extend(self.validate_trait_placement()?);
        // validate_declaration_collisions() removed - RefactoringValidator handles
        // duplicate definitions with better categorization (known migration pairs, severity)
        violations.extend(self.validate_layer_violations()?);
        // Strict CA directory and layer compliance
        violations.extend(self.validate_strict_directory()?);
        violations.extend(self.validate_domain_traits_only()?);
        Ok(violations)
    }

    /// Scans for numeric literals that should be extracted as named constants.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_magic_numbers(&self) -> Result<Vec<OrganizationViolation>> {
        validate_magic_numbers(&self.config)
    }

    /// Scans for string literals duplicated across multiple files that should be centralized.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_duplicate_strings(&self) -> Result<Vec<OrganizationViolation>> {
        validate_duplicate_strings(&self.config)
    }

    /// Verifies that files are located in the correct directories based on their architectural role.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_file_placement(&self) -> Result<Vec<OrganizationViolation>> {
        validate_file_placement(&self.config)
    }

    /// Verifies that trait definitions are located in the appropriate ports directory.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_trait_placement(&self) -> Result<Vec<OrganizationViolation>> {
        validate_trait_placement(&self.config)
    }

    /// Checks for violations of Clean Architecture layer boundaries.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_layer_violations(&self) -> Result<Vec<OrganizationViolation>> {
        validate_layer_violations(&self.config)
    }

    /// Enforces strict directory placement rules for specific component types.
    ///
    /// # Errors
    ///
    /// Returns an error if directory scanning fails.
    pub fn validate_strict_directory(&self) -> Result<Vec<OrganizationViolation>> {
        validate_strict_directory(&self.config)
    }

    /// Verifies that the domain layer contains only trait definitions and data structures.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_domain_traits_only(&self) -> Result<Vec<OrganizationViolation>> {
        validate_domain_traits_only(&self.config)
    }
}

crate::impl_validator!(
    OrganizationValidator,
    "organization",
    "Validates code organization patterns"
);
