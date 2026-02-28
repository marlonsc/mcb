//! Validator Trait and Registry
//!
//! Provides a unified interface for all validators and a registry
//! for managing and running validators.

use std::sync::Arc;

use crate::Result;
use crate::ValidationConfig;
use crate::ValidationError;
use crate::filters::LanguageId;
use crate::run_context::ValidationRunContext;
use crate::traits::violation::Violation;

/// Unified interface for all validators, enabling plugin-like registration.
pub trait Validator: Send + Sync {
    /// Unique name of this validator
    fn name(&self) -> &'static str;

    /// Run validation and return violations
    ///
    /// # Errors
    ///
    /// Returns an error if the validation process fails.
    fn validate(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>>;

    /// Whether this validator is enabled by default
    fn enabled_by_default(&self) -> bool {
        true
    }

    /// Description of what this validator checks
    fn description(&self) -> &'static str {
        ""
    }

    /// Languages this validator applies to. Empty = all languages (always run).
    fn supported_languages(&self) -> &[LanguageId] {
        &[LanguageId::Rust]
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
    /// Canonical validator names from linkme registry (domain). Use this for CLI/handlers.
    #[must_use]
    pub fn standard_validator_names() -> Vec<String> {
        mcb_domain::registry::validation::list_validator_names()
    }

    /// Create an empty registry
    #[must_use]
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
    #[must_use]
    pub fn validators(&self) -> &[Box<dyn Validator>] {
        &self.validators
    }

    /// Run all enabled validators
    ///
    /// # Errors
    ///
    /// Returns an error if the validation context cannot be built.
    pub fn validate_all(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
        let context = Arc::new(ValidationRunContext::build(config)?);
        ValidationRunContext::with_active(&context, || {
            let Some(active) = ValidationRunContext::active() else {
                return Err(ValidationError::ContextNotActive);
            };
            let mut all_violations = Vec::new();

            mcb_domain::info!(
                "validator_registry",
                "Validation run context initialized",
                &format!(
                    "trace_id={} file_inventory_source={} file_inventory_count={}",
                    active.trace_id(),
                    active.file_inventory_source().as_str(),
                    active.file_inventory_count()
                )
            );

            for validator in &self.validators {
                if !validator.enabled_by_default() {
                    continue;
                }

                let langs = validator.supported_languages();
                if !langs.is_empty() && !langs.iter().any(|l| active.has_files_for_language(*l)) {
                    mcb_domain::debug!(
                        "validator_registry",
                        "Skipping validator: no files match supported languages",
                        &format!(
                            "validator={} trace_id={}",
                            validator.name(),
                            active.trace_id()
                        )
                    );
                    continue;
                }

                mcb_domain::info!(
                    "validator_registry",
                    "Running validator",
                    &format!("validator={}", validator.name())
                );

                let started = std::time::Instant::now();
                match validator.validate(config) {
                    Ok(violations) => {
                        let count = violations.len();
                        let elapsed = started.elapsed();
                        mcb_domain::info!(
                            "validator_registry",
                            "Validator completed",
                            &format!(
                                "validator={} violations={} elapsed={:.2?}",
                                validator.name(),
                                count,
                                elapsed
                            )
                        );
                        all_violations.extend(violations);
                    }
                    Err(e) => {
                        mcb_domain::warn!(
                            "validator_registry",
                            "Validator failed, skipping",
                            &format!(
                                "validator={} trace_id={} error={}",
                                validator.name(),
                                active.trace_id(),
                                e
                            )
                        );
                    }
                }
            }

            Ok(all_violations)
        })
    }

    /// Run specific validators by name
    ///
    /// # Errors
    ///
    /// Returns an error if the validation context cannot be built.
    pub fn validate_named(
        &self,
        config: &ValidationConfig,
        names: &[&str],
    ) -> Result<Vec<Box<dyn Violation>>> {
        let context = Arc::new(ValidationRunContext::build(config)?);
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
            return Err(ValidationError::UnknownValidator {
                names: unknown.join(", "),
                available: available_list,
            });
        }

        ValidationRunContext::with_active(&context, || {
            let Some(active) = ValidationRunContext::active() else {
                return Err(ValidationError::ContextNotActive);
            };
            let mut all_violations = Vec::new();

            mcb_domain::info!(
                "validator_registry",
                "Named validation run context initialized",
                &format!(
                    "trace_id={} file_inventory_source={} file_inventory_count={}",
                    active.trace_id(),
                    active.file_inventory_source().as_str(),
                    active.file_inventory_count()
                )
            );

            for validator in &self.validators {
                if names.contains(&validator.name()) {
                    mcb_domain::info!(
                        "validator_registry",
                        "Running validator",
                        &format!("validator={}", validator.name())
                    );

                    let started = std::time::Instant::now();
                    match validator.validate(config) {
                        Ok(violations) => {
                            let count = violations.len();
                            let elapsed = started.elapsed();
                            mcb_domain::info!(
                                "validator_registry",
                                "Validator completed",
                                &format!(
                                    "validator={} violations={} elapsed={:.2?}",
                                    validator.name(),
                                    count,
                                    elapsed
                                )
                            );
                            all_violations.extend(violations);
                        }
                        Err(e) => {
                            mcb_domain::warn!(
                                "validator_registry",
                                "Validator failed, skipping",
                                &format!(
                                    "validator={} trace_id={} error={}",
                                    validator.name(),
                                    active.trace_id(),
                                    e
                                )
                            );
                        }
                    }
                }
            }

            Ok(all_violations)
        })
    }

    /// Create a registry with standard validators for a specific workspace
    pub fn standard_for(workspace_root: impl Into<std::path::PathBuf>) -> Self {
        let root = workspace_root.into();
        crate::mk_validators!(
            &root;
            crate::validators::clean_architecture::CleanArchitectureValidator,
            crate::validators::layer_flow::LayerFlowValidator,
            crate::validators::port_adapter::PortAdapterValidator,
            crate::validators::visibility::VisibilityValidator,
            crate::validators::dependency::DependencyValidator,
            crate::validators::quality::QualityValidator,
            crate::validators::solid::SolidValidator,
            crate::validators::naming::NamingValidator,
            crate::validators::pattern_validator::PatternValidator,
            crate::validators::documentation::DocumentationValidator,
            crate::validators::hygiene::HygieneValidator,
            crate::validators::performance::PerformanceValidator,
            crate::validators::async_patterns::AsyncPatternValidator,
            crate::validators::kiss::KissValidator,
            crate::validators::organization::OrganizationValidator,
            crate::validators::implementation::ImplementationQualityValidator,
            crate::validators::refactoring::RefactoringValidator,
            crate::validators::error_boundary::ErrorBoundaryValidator,
            crate::validators::declarative_validator::DeclarativeValidator,
        )
    }
}
