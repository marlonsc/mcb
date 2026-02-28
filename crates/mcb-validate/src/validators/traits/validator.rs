//! Validator Trait and Registry
//!
//! Provides a unified interface for all validators and a registry
//! for managing and running validators.
//!
//! ## Sub-Check Pattern
//!
//! Validators expose their work as a list of [`NamedCheck`]s via [`Validator::checks`].
//! The default [`Validator::validate`] implementation runs them through
//! [`run_checks`], which adds automatic timing and debug logging.
//!
//! Validators with custom execution logic (e.g. `DeclarativeValidator`) can
//! override `validate()` directly and still use `run_checks()` internally.

use std::sync::Arc;

use crate::Result;
use crate::ValidationConfig;
use crate::ValidationError;
use crate::filters::LanguageId;
use crate::run_context::ValidationRunContext;
use crate::traits::violation::Violation;

// ============================================================================
// NamedCheck — a single executable sub-check
// ============================================================================

/// A named, executable sub-check within a validator.
///
/// Each validator can expose multiple `NamedCheck`s via [`Validator::checks`].
/// The runner ([`run_checks`]) executes them sequentially with automatic
/// timing and debug-level logging.
pub struct NamedCheck<'a> {
    /// Human-readable check name (e.g. "struct_fields", "function_params")
    pub name: &'static str,
    /// Closure that runs the check and returns violations
    pub run: Box<dyn FnOnce() -> Result<Vec<Box<dyn Violation>>> + 'a>,
}

impl<'a> NamedCheck<'a> {
    /// Create a new named check.
    pub fn new(
        name: &'static str,
        run: impl FnOnce() -> Result<Vec<Box<dyn Violation>>> + 'a,
    ) -> Self {
        Self {
            name,
            run: Box::new(run),
        }
    }
}

// ============================================================================
// run_checks — centralized check runner with timing + logging
// ============================================================================

/// Execute named checks with automatic timing and debug logging.
///
/// This is the single place where the timing/logging pattern lives.
/// All validators delegate to this (either via the default `validate()`
/// implementation or by calling it directly).
pub fn run_checks(
    validator_name: &str,
    checks: Vec<NamedCheck<'_>>,
) -> Result<Vec<Box<dyn Violation>>> {
    let mut violations = Vec::new();
    for check in checks {
        let t = std::time::Instant::now();
        let v = (check.run)()?;
        mcb_domain::debug!(
            validator_name,
            &format!("{} done", check.name),
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);
    }
    Ok(violations)
}

// ============================================================================
// Validator trait
// ============================================================================

/// Unified interface for all validators, enabling plugin-like registration.
///
/// ## Sub-check pattern
///
/// Implement [`checks`](Validator::checks) to expose named sub-checks.
/// The default [`validate`](Validator::validate) will run them through
/// [`run_checks`] with automatic timing and logging.
///
/// Override `validate()` only when custom execution logic is needed
/// (e.g. `DeclarativeValidator` which manages its own slices).
pub trait Validator: Send + Sync {
    /// Unique name of this validator
    fn name(&self) -> &'static str;

    /// Return the list of named sub-checks this validator runs.
    ///
    /// Default returns an empty list. Override this to expose sub-checks.
    /// The default `validate()` implementation uses this.
    fn checks<'a>(&'a self, _config: &'a ValidationConfig) -> Result<Vec<NamedCheck<'a>>> {
        Ok(Vec::new())
    }

    /// Run validation and return violations.
    ///
    /// Default implementation calls `checks()` and runs them via `run_checks()`.
    /// Override for custom execution logic.
    ///
    /// # Errors
    ///
    /// Returns an error if the validation process fails.
    fn validate(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
        run_checks(self.name(), self.checks(config)?)
    }

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

    /// Run a single validator with timing, logging, and error handling
    fn run_single_validator(
        &self,
        validator: &Box<dyn Validator>,
        config: &ValidationConfig,
        trace_id: &str,
        all_violations: &mut Vec<Box<dyn Violation>>,
    ) {
        let started = std::time::Instant::now();
        match validator.as_ref().validate(config) {
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
                        trace_id,
                        e
                    )
                );
            }
        }
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

                self.run_single_validator(
                    validator,
                    config,
                    active.trace_id(),
                    &mut all_violations,
                );
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

                    self.run_single_validator(
                        validator,
                        config,
                        active.trace_id(),
                        &mut all_violations,
                    );
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
