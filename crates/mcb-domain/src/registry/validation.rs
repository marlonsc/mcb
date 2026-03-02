//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Validator Registry
//!
//! Auto-registration system for validators using linkme distributed slices.
//! Validators register themselves via `#[linkme::distributed_slice(VALIDATOR_ENTRIES)]`
//! in implementing crates (e.g. mcb-validate) and are discovered at runtime.

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::ports::validation::Validator;
use crate::ports::{ValidationReport, ViolationEntry};

// ============================================================================
// Validator entries (linkme-discovered validators)
// ============================================================================

/// Registry entry for a single validator (e.g. `clean_architecture`, `quality`).
///
/// Validators register via `#[linkme::distributed_slice(VALIDATOR_ENTRIES)]`
/// in implementing crates (e.g. mcb-validate).
pub struct ValidatorEntry {
    /// Unique validator name (e.g. `"clean_architecture"`).
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Build a validator instance for the given workspace root
    pub build: fn(PathBuf) -> std::result::Result<Box<dyn Validator>, String>,
}

#[linkme::distributed_slice]
/// Distributed slice of all registered validators.
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

/// Return the number of registered validators.
#[must_use]
pub fn validator_count() -> usize {
    VALIDATOR_ENTRIES.len()
}

/// Build all registered validators for the given workspace root.
///
/// # Errors
///
/// Returns an error if any validator's build function fails.
pub fn build_all_validators(workspace_root: &Path) -> Result<Vec<Box<dyn Validator>>> {
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
) -> Result<Vec<Box<dyn Validator>>> {
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

/// Convert a `dyn Violation` to a serializable `ViolationEntry`.
///
/// This is the canonical conversion used by both mcb-validate and mcb-infrastructure
/// to turn trait objects into string-based entries for reports.
pub fn violation_to_entry(v: &dyn crate::ports::validation::Violation) -> ViolationEntry {
    ViolationEntry {
        id: v.id().to_owned(),
        category: v.category().to_string(),
        severity: v.severity().to_string(),
        file: v.file().map(|p| p.display().to_string()),
        line: v.line(),
        message: v.message(),
        suggestion: v.suggestion(),
    }
}

// ============================================================================
// Report building
// ============================================================================

/// Build a `ValidationReport` from a list of trait-object violations.
///
/// Handles counting by severity and applying an optional severity filter.
#[must_use]
pub fn build_report(
    violations: &[Box<dyn crate::ports::validation::Violation>],
    severity_filter: Option<&str>,
) -> ValidationReport {
    let entries: Vec<ViolationEntry> = violations
        .iter()
        .map(|v| violation_to_entry(v.as_ref()))
        .collect();

    let filtered = filter_violations_by_severity(&entries, severity_filter);

    let errors = filtered.iter().filter(|e| e.severity == "ERROR").count();
    let warnings = filtered.iter().filter(|e| e.severity == "WARNING").count();
    let infos = filtered.iter().filter(|e| e.severity == "INFO").count();

    // `passed` is based on unfiltered error count
    let total_errors = entries.iter().filter(|e| e.severity == "ERROR").count();

    ValidationReport {
        total_violations: filtered.len(),
        errors,
        warnings,
        infos,
        violations: filtered,
        passed: total_errors == 0,
    }
}

fn filter_violations_by_severity(
    violations: &[ViolationEntry],
    severity_filter: Option<&str>,
) -> Vec<ViolationEntry> {
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
                _ => 3,
            };
            l <= level
        })
        .cloned()
        .collect()
}
