//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Validation Provider Registry
//!
//! Auto-registration system for validation providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.
//!
//! Individual rule validators (e.g. `clean_architecture`, `quality`) are also
//! registered via [`VALIDATOR_ENTRIES`] and built/resolved through this module.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::Result;
use crate::ports::{RuleValidator, RuleValidatorRequest, ValidationReport};

/// Configuration for validation provider creation
///
/// Contains all configuration options that a validation provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct ValidationProviderConfig {
    /// Provider name (e.g., "mcb-validate")
    pub provider: String,
    /// Workspace root for validation scans
    pub workspace_root: Option<PathBuf>,
    /// Validator names to run (None = all)
    pub validators: Option<Vec<String>>,
    /// Minimum severity to report (error, warning, info)
    pub severity_filter: Option<String>,
    /// Path patterns to exclude from validation
    pub exclude_patterns: Option<Vec<String>>,
    /// Maximum files to validate
    pub max_files: Option<usize>,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(ValidationProviderConfig {
    /// Set workspace root
    workspace_root: with_workspace_root(into PathBuf),
    /// Set validators to run
    validators: with_validators(Vec<String>),
    /// Set severity filter
    severity_filter: with_severity_filter(into String),
    /// Set exclude patterns
    exclude_patterns: with_exclude_patterns(Vec<String>),
    /// Set maximum files to validate
    max_files: with_max_files(usize),
});

crate::impl_registry!(
    provider_trait: crate::ports::ValidationProvider,
    config_type: ValidationProviderConfig,
    entry_type: ValidationProviderEntry,
    slice_name: VALIDATION_PROVIDERS,
    resolve_fn: resolve_validation_provider,
    list_fn: list_validation_providers
);

// ============================================================================
// Rule validator entries (linkme-discovered validators)
// ============================================================================

/// Registry entry for a single rule validator (e.g. `clean_architecture`, `quality`).
///
/// Validators register via `#[linkme::distributed_slice(VALIDATOR_ENTRIES)]`
/// in implementing crates (e.g. mcb-validate).
pub struct ValidatorEntry {
    /// Unique validator name (e.g. `"clean_architecture"`).
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Build a validator instance for the given workspace root
    pub build: fn(PathBuf) -> std::result::Result<Arc<dyn RuleValidator>, String>,
}

#[linkme::distributed_slice]
/// Distributed slice of all registered rule validators.
pub static VALIDATOR_ENTRIES: [ValidatorEntry] = [..];

/// List all registered validator (name, description) pairs.
#[must_use]
pub fn list_validator_entries() -> Vec<(&'static str, &'static str)> {
    VALIDATOR_ENTRIES
        .iter()
        .map(|e| (e.name, e.description))
        .collect()
}

/// List all registered validator names (single source of truth for CLI and handlers).
#[must_use]
pub fn list_validator_names() -> Vec<String> {
    VALIDATOR_ENTRIES
        .iter()
        .map(|e| e.name.to_owned())
        .collect()
}

/// Build all registered validators for the given workspace root.
///
/// # Errors
///
/// Returns an error if any validator's build function fails.
pub fn build_validators(workspace_root: &Path) -> Result<Vec<Arc<dyn RuleValidator>>> {
    let mut out = Vec::with_capacity(VALIDATOR_ENTRIES.len());
    for entry in VALIDATOR_ENTRIES {
        let v = (entry.build)(workspace_root.to_path_buf()).map_err(|e| {
            crate::error::Error::configuration(format!("validator '{}': {}", entry.name, e))
        })?;
        out.push(v);
    }
    Ok(out)
}

/// Build only the requested validators for the given workspace root.
///
/// Unknown validator names are ignored to preserve existing behavior.
///
/// # Errors
///
/// Returns an error if any selected validator's build function fails.
pub fn build_named_validators(
    workspace_root: &Path,
    validator_names: &[String],
) -> Result<Vec<Arc<dyn RuleValidator>>> {
    let requested: std::collections::HashSet<&str> =
        validator_names.iter().map(String::as_str).collect();
    let mut out = Vec::new();

    for entry in VALIDATOR_ENTRIES {
        if !requested.contains(entry.name) {
            continue;
        }

        let validator = (entry.build)(workspace_root.to_path_buf()).map_err(|e| {
            crate::error::Error::configuration(format!("validator '{}': {}", entry.name, e))
        })?;
        out.push(validator);
    }

    Ok(out)
}

/// Run a set of validators with the given request and merge reports.
///
/// If `request.validator_names` is `Some`, only validators whose name is in the list are run.
///
/// # Errors
///
/// Returns an error if any validator's `run` fails.
pub fn run_validators(
    validators: &[Arc<dyn RuleValidator>],
    request: &RuleValidatorRequest,
) -> Result<ValidationReport> {
    let to_run: Vec<&Arc<dyn RuleValidator>> = if let Some(ref names) = request.validator_names {
        let set: std::collections::HashSet<&str> = names.iter().map(String::as_str).collect();
        validators
            .iter()
            .filter(|v| set.contains(v.name()))
            .collect()
    } else {
        validators.iter().collect()
    };

    let mut all_violations = Vec::new();
    for v in to_run {
        let report = v.run(request)?;
        all_violations.extend(report.violations);
    }

    let errors = all_violations
        .iter()
        .filter(|e| e.severity == "ERROR")
        .count();
    let _warnings = all_violations
        .iter()
        .filter(|e| e.severity == "WARNING")
        .count();
    let _infos = all_violations
        .iter()
        .filter(|e| e.severity == "INFO")
        .count();

    let filtered =
        filter_violations_by_severity(&all_violations, request.severity_filter.as_deref());
    let errors_f = filtered.iter().filter(|e| e.severity == "ERROR").count();
    let warnings_f = filtered.iter().filter(|e| e.severity == "WARNING").count();
    let infos_f = filtered.iter().filter(|e| e.severity == "INFO").count();

    Ok(ValidationReport {
        total_violations: filtered.len(),
        errors: errors_f,
        warnings: warnings_f,
        infos: infos_f,
        violations: filtered,
        passed: errors == 0,
    })
}

fn filter_violations_by_severity(
    violations: &[crate::ports::ViolationEntry],
    severity_filter: Option<&str>,
) -> Vec<crate::ports::ViolationEntry> {
    let level = match severity_filter {
        Some("error") => 1,
        Some("warning") => 2,
        Some("info") => 3,
        _ => return violations.to_vec(),
    };
    violations
        .iter()
        .filter(|v| {
            let l = match v.severity.as_str() {
                "ERROR" => 1,
                "WARNING" => 2,
                // INFO and any unknown severity treated as lowest priority
                _ => 3,
            };
            l <= level
        })
        .cloned()
        .collect()
}
