//! Validator execution runner.

use rayon::prelude::*;
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
        let all_violations: Vec<Box<dyn Violation>> = validators
            .par_iter()
            .filter(|v| {
                if !v.enabled_by_default() {
                    return false;
                }
                let langs = v.supported_languages();
                langs.is_empty() || langs.iter().any(|l| active.has_files_for_language(*l))
            })
            .flat_map_iter(|validator| {
                ValidationRunContext::with_active(&context, || {
                    run_single_validator(validator.as_ref(), config, active.trace_id())
                })
            })
            .collect();
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
                all_violations.extend(run_single_validator(
                    validator.as_ref(),
                    config,
                    active.trace_id(),
                ));
            }
        }
        Ok(all_violations)
    })
}

fn run_single_validator(
    validator: &dyn Validator,
    config: &ValidationConfig,
    trace_id: &str,
) -> Vec<Box<dyn Violation>> {
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
            violations
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
            Vec::new()
        }
    }
}

/// Return the standard validator names from the domain registry.
#[must_use]
pub fn standard_validator_names() -> Vec<String> {
    mcb_domain::registry::validation::list_validator_names()
}
