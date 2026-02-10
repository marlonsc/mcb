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
    /// Canonical validator names used by registry-based execution.
    pub const STANDARD_VALIDATOR_NAMES: &'static [&'static str] = &[
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
    ];

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
        let mut available = std::collections::BTreeSet::new();
        for validator in &self.validators {
            available.insert(validator.name());
        }

        let mut unknown: Vec<&str> = names
            .iter()
            .copied()
            .filter(|name| !available.contains(name))
            .collect();
        unknown.sort_unstable();
        unknown.dedup();

        if !unknown.is_empty() {
            let available_list = available.into_iter().collect::<Vec<_>>().join(", ");
            return Err(anyhow::anyhow!(
                "Unknown validator(s): {}. Available validators: {}",
                unknown.join(", "),
                available_list
            ));
        }

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
        let root = workspace_root.into();
        crate::mk_validators!(
            &root;
            crate::clean_architecture::CleanArchitectureValidator,
            crate::layer_flow::LayerFlowValidator,
            crate::port_adapter::PortAdapterValidator,
            crate::visibility::VisibilityValidator,
            crate::dependency::DependencyValidator,
            crate::quality::QualityValidator,
            crate::solid::SolidValidator,
            crate::naming::NamingValidator,
            crate::pattern_validator::PatternValidator,
            crate::documentation::DocumentationValidator,
            crate::tests_org::TestValidator,
            crate::performance::PerformanceValidator,
            crate::async_patterns::AsyncPatternValidator,
            crate::kiss::KissValidator,
            crate::pmat::PmatValidator,
            crate::organization::OrganizationValidator,
            crate::implementation::ImplementationQualityValidator,
            crate::refactoring::RefactoringValidator,
            crate::error_boundary::ErrorBoundaryValidator,
        )
    }

    /// Return canonical validator names for public API consumers.
    pub fn standard_validator_names() -> &'static [&'static str] {
        Self::STANDARD_VALIDATOR_NAMES
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

    use super::{LegacyValidatorAdapter, ValidatorRegistry};
    use crate::ValidationConfig;

    #[test]
    fn test_canonical_registry_completeness() {
        let registry = ValidatorRegistry::standard_for(".");
        let names: Vec<&'static str> = registry
            .validators()
            .iter()
            .map(|validator| validator.name())
            .collect();

        let actual: BTreeSet<&'static str> = names.iter().copied().collect();
        let expected: BTreeSet<&'static str> = ValidatorRegistry::standard_validator_names()
            .iter()
            .copied()
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

    #[test]
    fn test_validate_named_rejects_unknown_validators() {
        let registry = ValidatorRegistry::new().with_validator(Box::new(
            LegacyValidatorAdapter::new("known", "", |_| Ok(Vec::new())),
        ));
        let config = ValidationConfig::new(".");

        let err = registry
            .validate_named(&config, &["known", "unknown", "unknown"])
            .expect_err("expected unknown validator names to fail");

        let msg = err.to_string();
        assert!(msg.contains("Unknown validator(s): unknown"));
        assert!(msg.contains("Available validators: known"));
    }
}
