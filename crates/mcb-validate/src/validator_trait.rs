//! Validator Trait and Registry
//!
//! Provides a unified interface for all validators and a registry
//! for managing and running validators.

use anyhow::Result;

use crate::ValidationConfig;
use crate::violation_trait::Violation;

/// All validators implement this trait
///
/// This enables a plugin-like architecture where validators can be
/// registered and run uniformly.
pub trait Validator: Send + Sync {
    /// Unique name of this validator
    fn name(&self) -> &'static str;

    /// Run validation and return violations
    fn validate(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>>;

    /// Whether this validator is enabled by default
    fn enabled_by_default(&self) -> bool {
        true
    }

    /// Description of what this validator checks
    fn description(&self) -> &'static str {
        ""
    }
}

/// Registry of validators
///
/// Manages a collection of validators and provides methods to run
/// all or selected validators.
pub struct ValidatorRegistry {
    validators: Vec<Box<dyn Validator>>,
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Register a validator
    pub fn register(&mut self, validator: Box<dyn Validator>) {
        self.validators.push(validator);
    }

    /// Register a validator (builder pattern)
    #[must_use]
    pub fn with_validator(mut self, validator: Box<dyn Validator>) -> Self {
        self.register(validator);
        self
    }

    /// Get all registered validators
    pub fn validators(&self) -> &[Box<dyn Validator>] {
        &self.validators
    }

    /// Run all enabled validators
    pub fn validate_all(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
        let mut all_violations = Vec::new();

        for validator in &self.validators {
            if validator.enabled_by_default() {
                match validator.validate(config) {
                    Ok(violations) => all_violations.extend(violations),
                    Err(e) => {
                        eprintln!(
                            "Warning: Validator '{}' failed, skipping: {}",
                            validator.name(),
                            e
                        );
                    }
                }
            }
        }

        Ok(all_violations)
    }

    /// Run specific validators by name
    pub fn validate_named(
        &self,
        config: &ValidationConfig,
        names: &[&str],
    ) -> Result<Vec<Box<dyn Violation>>> {
        let mut all_violations = Vec::new();

        for validator in &self.validators {
            if names.contains(&validator.name()) {
                match validator.validate(config) {
                    Ok(violations) => all_violations.extend(violations),
                    Err(e) => {
                        eprintln!(
                            "Warning: Validator '{}' failed, skipping: {}",
                            validator.name(),
                            e
                        );
                    }
                }
            }
        }

        Ok(all_violations)
    }

    /// Create a registry with standard validators
    ///
    /// This registers all built-in validators with default configuration.
    /// Validators include:
    /// - Architecture: `clean_architecture`, `layer_flow`, `port_adapter`, visibility
    /// - Dependencies: dependency
    /// - Quality: quality, solid, naming, patterns, documentation, `tests_org`
    /// - Performance: performance, `async_patterns`, kiss, pmat
    /// - Organization: organization, implementation, refactoring, `error_boundary`
    pub fn standard() -> Self {
        Self::standard_for(".")
    }

    /// Create a registry with standard validators for a specific workspace
    pub fn standard_for(workspace_root: impl Into<std::path::PathBuf>) -> Self {
        // Architecture validators
        // Performance validators
        use crate::async_patterns::AsyncPatternValidator;
        use crate::clean_architecture::CleanArchitectureValidator;
        // Dependency validators
        use crate::dependency::DependencyValidator;
        // Note: Legacy validator removed - now using linkme-based plugin architecture

        // Quality validators
        use crate::documentation::DocumentationValidator;
        // Organization validators
        use crate::error_boundary::ErrorBoundaryValidator;
        use crate::implementation::ImplementationQualityValidator;
        use crate::kiss::KissValidator;
        use crate::layer_flow::LayerFlowValidator;
        use crate::naming::NamingValidator;
        use crate::organization::OrganizationValidator;
        use crate::pattern_validator::PatternValidator;
        use crate::performance::PerformanceValidator;
        use crate::pmat::PmatValidator;
        use crate::port_adapter::PortAdapterValidator;
        use crate::quality::QualityValidator;
        use crate::refactoring::RefactoringValidator;
        use crate::solid::SolidValidator;
        use crate::tests_org::TestValidator;
        use crate::visibility::VisibilityValidator;

        let root = workspace_root.into();

        Self::new()
            // Architecture
            .with_validator(Box::new(CleanArchitectureValidator::new(&root)))
            .with_validator(Box::new(LayerFlowValidator::new(&root)))
            .with_validator(Box::new(PortAdapterValidator::new(&root)))
            .with_validator(Box::new(VisibilityValidator::new(&root)))
            // Dependencies
            .with_validator(Box::new(DependencyValidator::new(&root)))
            // Note: Legacy validator removed - now using linkme-based plugin architecture
            // Quality
            .with_validator(Box::new(QualityValidator::new(&root)))
            .with_validator(Box::new(SolidValidator::new(&root)))
            .with_validator(Box::new(NamingValidator::new(&root)))
            .with_validator(Box::new(PatternValidator::new(&root)))
            .with_validator(Box::new(DocumentationValidator::new(&root)))
            .with_validator(Box::new(TestValidator::new(&root)))
            // Performance
            .with_validator(Box::new(PerformanceValidator::new(&root)))
            .with_validator(Box::new(AsyncPatternValidator::new(&root)))
            .with_validator(Box::new(KissValidator::new(&root)))
            .with_validator(Box::new(PmatValidator::new(&root)))
            // Organization
            .with_validator(Box::new(OrganizationValidator::new(&root)))
            .with_validator(Box::new(ImplementationQualityValidator::new(&root)))
            .with_validator(Box::new(RefactoringValidator::new(&root)))
            .with_validator(Box::new(ErrorBoundaryValidator::new(&root)))
    }
}

/// Helper struct for wrapping existing validators
///
/// This allows existing validators to be used with the new registry
/// during the migration period.
pub struct LegacyValidatorAdapter<F>
where
    F: Fn(&ValidationConfig) -> Result<Vec<Box<dyn Violation>>> + Send + Sync,
{
    /// Unique name of the validator
    name: &'static str,
    /// Detailed description of what it validates
    description: &'static str,
    /// Function that performs the validation
    validate_fn: F,
}

impl<F> LegacyValidatorAdapter<F>
where
    F: Fn(&ValidationConfig) -> Result<Vec<Box<dyn Violation>>> + Send + Sync,
{
    /// Create a new adapter
    pub fn new(name: &'static str, description: &'static str, validate_fn: F) -> Self {
        Self {
            name,
            description,
            validate_fn,
        }
    }
}

impl<F> Validator for LegacyValidatorAdapter<F>
where
    F: Fn(&ValidationConfig) -> Result<Vec<Box<dyn Violation>>> + Send + Sync,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn validate(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
        (self.validate_fn)(config)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::ValidatorRegistry;

    #[test]
    fn test_canonical_registry_completeness() {
        let registry = ValidatorRegistry::standard_for(".");
        let names: Vec<&'static str> = registry
            .validators()
            .iter()
            .map(|validator| validator.name())
            .collect();

        let actual: BTreeSet<&'static str> = names.iter().copied().collect();
        let expected: BTreeSet<&'static str> = [
            "clean_architecture",
            "layer_flow",
            "port_adapter",
            "visibility",
            "dependency",
            "quality",
            "solid",
            "naming",
            "patterns",
            "documentation",
            "tests_org",
            "performance",
            "async_patterns",
            "kiss",
            "pmat",
            "organization",
            "implementation",
            "refactoring",
            "error_boundary",
        ]
        .into_iter()
        .collect();

        assert_eq!(
            actual, expected,
            "registry validator set must match canonical list"
        );
        assert_eq!(
            names.len(),
            expected.len(),
            "registry must not contain duplicate validators"
        );
    }
}
