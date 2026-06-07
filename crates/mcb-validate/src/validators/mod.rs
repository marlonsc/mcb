//! Architecture Validators - Clean Architecture, SOLID, Quality, and hygiene
//!
//! **Documentation**: [`docs/modules/validate.md#validators-validators`](../../../../docs/modules/validate.md#validators-validators)
//!
//! Validation macros (`impl_validator!`, `define_violations!`) live in `crate::macros`.

pub mod async_patterns;
pub mod clean_architecture;
pub mod config_quality;
pub(crate) mod declarative_support;
pub mod declarative_validator;
/// Dependency validation module
pub mod dependency;
pub mod documentation;
pub mod error_boundary;
mod helpers;
/// Hygiene validation module (e.g., TODOs, formatting)
pub mod hygiene;
/// Implementation pattern validation module
pub mod implementation;
/// KISS principle validation (Keep It Simple, Stupid).
pub mod kiss;
pub mod layer_flow;
/// Naming convention validation module
pub mod naming;
/// Organization validation (e.g., directory structure)
pub mod organization;
pub mod pattern_validator;
pub mod performance;
pub mod pmat;
pub(crate) mod pmat_native;
pub mod port_adapter;
/// Code quality validation module (unwrap, panic, metrics)
pub mod quality;
pub mod refactoring;
/// SOLID principles validation module
pub mod solid;
/// Single Source of Truth (SSOT) invariants validator
pub mod ssot;
pub mod test_quality;
pub mod visibility;

pub(crate) use helpers::for_each_non_test_non_comment_line;

pub use self::async_patterns::{AsyncPatternValidator, AsyncViolation};
pub use self::clean_architecture::{CleanArchitectureValidator, CleanArchitectureViolation};
pub use self::config_quality::{ConfigQualityValidator, ConfigQualityViolation};
pub use self::declarative_validator::DeclarativeValidator;
pub use self::dependency::{DependencyValidator, DependencyViolation};
pub use self::documentation::{DocumentationValidator, DocumentationViolation};
pub use self::error_boundary::{ErrorBoundaryValidator, ErrorBoundaryViolation};
pub use self::hygiene::{HygieneValidator, HygieneViolation};
pub use self::implementation::{ImplementationQualityValidator, ImplementationViolation};
pub use self::kiss::{KissValidator, KissViolation};
pub use self::layer_flow::{LayerFlowValidator, LayerFlowViolation};
pub use self::naming::{NamingValidator, NamingViolation};
pub use self::organization::{OrganizationValidator, OrganizationViolation};
pub use self::pattern_validator::{PatternValidator, PatternViolation};
pub use self::performance::{PerformanceValidator, PerformanceViolation};
pub use self::pmat::{PmatValidator, PmatViolation};
pub use self::port_adapter::{PortAdapterValidator, PortAdapterViolation};
pub use self::quality::{QualityValidator, QualityViolation};
pub use self::refactoring::{RefactoringValidator, RefactoringViolation};
pub use self::solid::{SolidValidator, SolidViolation};
pub use self::ssot::{SsotValidator, SsotViolation};
pub use self::test_quality::{TestQualityValidator, TestQualityViolation};
pub use self::visibility::{VisibilityValidator, VisibilityViolation};

use std::sync::Arc;

use crate::run_context::ValidationRunContext;
use crate::{Result, ValidationConfig, ValidationError, Validator, Violation};

/// Run all enabled validators and return violations.
/// # Errors
/// Returns an error if the validation context cannot be built.
pub fn validate_all(config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
    let context = Arc::new(ValidationRunContext::build(config)?);
    ValidationRunContext::with_active(&context, || {
        let Some(active) = ValidationRunContext::active() else {
            return Err(ValidationError::ContextNotActive);
        };
        let validators =
            mcb_domain::registry::validation::build_all_validators(&config.workspace_root)
                .map_err(|e| ValidationError::Config(e.to_string()))?;
        let mut all_violations = Vec::new();
        mcb_domain::info!(
            "validators",
            "Validation run started",
            &format!(
                "trace_id={} validators={} files={}",
                active.trace_id(),
                validators.len(),
                active.file_inventory_count()
            )
        );
        for validator in &validators {
            if !validator.enabled_by_default() {
                continue;
            }
            let langs = validator.supported_languages();
            if !langs.is_empty() && !langs.iter().any(|l| active.has_files_for_language(*l)) {
                continue;
            }
            run_single_validator(
                validator.as_ref(),
                config,
                active.trace_id(),
                &mut all_violations,
            );
        }
        Ok(all_violations)
    })
}

/// Run only the named validators and return violations.
/// # Errors
/// Returns an error if the validation context cannot be built or unknown validators are requested.
pub fn validate_named(
    config: &ValidationConfig,
    names: &[&str],
) -> Result<Vec<Box<dyn Violation>>> {
    let validators = mcb_domain::registry::validation::build_all_validators(&config.workspace_root)
        .map_err(|e| ValidationError::Config(e.to_string()))?;

    let available: std::collections::BTreeSet<&str> = validators
        .iter()
        .map(|validator| validator.name())
        .collect();
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

    let context = Arc::new(ValidationRunContext::build(config)?);
    ValidationRunContext::with_active(&context, || {
        let Some(active) = ValidationRunContext::active() else {
            return Err(ValidationError::ContextNotActive);
        };
        let mut all_violations = Vec::new();
        for validator in &validators {
            if names.contains(&validator.name()) {
                run_single_validator(
                    validator.as_ref(),
                    config,
                    active.trace_id(),
                    &mut all_violations,
                );
            }
        }
        Ok(all_violations)
    })
}

fn run_single_validator(
    validator: &dyn Validator,
    config: &ValidationConfig,
    trace_id: &str,
    all_violations: &mut Vec<Box<dyn Violation>>,
) {
    let started = std::time::Instant::now();
    match validator.validate(config) {
        Ok(violations) => {
            let count = violations.len();
            let elapsed = started.elapsed();
            mcb_domain::info!(
                "validators",
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
                "validators",
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

/// Return the standard validator names from the domain registry.
#[must_use]
pub fn standard_validator_names() -> Vec<String> {
    mcb_domain::registry::validation::list_validator_names()
}
