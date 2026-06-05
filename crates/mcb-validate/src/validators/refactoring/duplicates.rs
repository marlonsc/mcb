//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#refactoring)
//!
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};
use mcb_utils::utils::regex::compile_regex;

use super::RefactoringValidator;
use super::violation::RefactoringViolation;
use mcb_utils::constants::validate::{
    CRATE_PATH_DELIMITER, MIGRATION_TYPE_SUFFIXES, REFACTORING_SKIP_PATTERNS, TYPE_DEFINITION_REGEX,
};

/// Extract the set of crate names that own the given type definition locations.
fn crates_from_locations(locations: &[PathBuf]) -> HashSet<String> {
    locations
        .iter()
        .filter_map(|p| {
            p.to_str()?
                .split(CRATE_PATH_DELIMITER)
                .nth(1)
                .and_then(|s| s.split('/').next())
                .map(str::to_owned)
        })
        .collect()
}

/// Count the distinct parent directories among the given locations.
fn duplicate_dirs_count(locations: &[PathBuf]) -> usize {
    locations
        .iter()
        .filter_map(|p| p.parent()?.to_str().map(str::to_owned))
        .collect::<HashSet<_>>()
        .len()
}

/// Build the cross-crate consolidation suggestion message for a given severity.
fn duplicate_suggestion(type_name: &str, crates: &HashSet<String>, severity: Severity) -> String {
    match severity {
        Severity::Info => format!(
            "Type '{type_name}' exists in {crates:?}. This is a known migration pattern - consolidate when migration completes."
        ),
        Severity::Warning => format!(
            "Type '{type_name}' is defined in {crates:?}. Consider consolidating to one location."
        ),
        Severity::Error => format!(
            "Type '{type_name}' is unexpectedly defined in multiple crates: {crates:?}. This requires immediate consolidation."
        ),
    }
}

/// Record every type definition in `content`, keyed by type name, skipping
/// generic names expected to recur.
fn record_definitions(
    validator: &RefactoringValidator,
    path: &Path,
    content: &str,
    definition_pattern: &regex::Regex,
    definitions: &mut HashMap<String, Vec<PathBuf>>,
) {
    for cap in definition_pattern.captures_iter(content) {
        let type_name = cap.get(1).map_or("", |m| m.as_str());
        if validator.generic_type_names.contains(type_name) {
            continue;
        }
        definitions
            .entry(type_name.to_owned())
            .or_default()
            .push(path.to_path_buf());
    }
}

/// Returns a `DuplicateDefinition` violation if `type_name` is defined in
/// multiple crates, or multiple directories of the same crate, else `None`.
fn duplicate_violation(
    validator: &RefactoringValidator,
    type_name: &str,
    locations: &[PathBuf],
) -> Option<RefactoringViolation> {
    if locations.len() <= 1 {
        return None;
    }

    let crates = crates_from_locations(locations);
    if crates.len() > 1 {
        let severity = categorize_duplicate_severity(validator, type_name, &crates);
        return Some(RefactoringViolation::DuplicateDefinition {
            type_name: type_name.to_owned(),
            locations: locations.to_vec(),
            suggestion: duplicate_suggestion(type_name, &crates, severity),
            severity,
        });
    }

    (duplicate_dirs_count(locations) >= 2).then(|| RefactoringViolation::DuplicateDefinition {
        type_name: type_name.to_owned(),
        locations: locations.to_vec(),
        suggestion: format!(
            "Type '{}' is defined {} times in different directories within the same crate. Consolidate to single location.",
            type_name,
            locations.len()
        ),
        severity: Severity::Warning,
    })
}

/// Check for same type defined in multiple locations
///
/// # Errors
///
/// Returns an error if regex compilation, file scanning, or reading fails.
pub fn validate_duplicate_definitions(
    validator: &RefactoringValidator,
) -> Result<Vec<RefactoringViolation>> {
    let definition_pattern = compile_regex(TYPE_DEFINITION_REGEX)?;

    // Map: type_name -> Vec<file_path>
    let mut definitions: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for_each_scan_file(
        &validator.config,
        Some(LanguageId::Rust),
        false,
        |entry, candidate_src_dir| {
            if validator.should_skip_crate(candidate_src_dir) {
                return Ok(());
            }

            let path = &entry.absolute_path;
            let Some(path_str) = path.to_str() else {
                return Ok(());
            };
            if REFACTORING_SKIP_PATTERNS
                .iter()
                .any(|p| path_str.contains(p))
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            record_definitions(
                validator,
                path,
                &content,
                &definition_pattern,
                &mut definitions,
            );
            Ok(())
        },
    )?;

    Ok(definitions
        .into_iter()
        .filter_map(|(type_name, locations)| duplicate_violation(validator, &type_name, &locations))
        .collect())
}

/// Whether `crates` is exactly a known migration pair (in either order).
fn is_known_migration_pair(validator: &RefactoringValidator, crates: &HashSet<String>) -> bool {
    let crate_vec: Vec<&String> = crates.iter().collect();
    if crate_vec.len() != 2 {
        return false;
    }
    validator
        .known_migration_pairs
        .iter()
        .any(|(crate_a, crate_b)| {
            (crate_vec[0] == crate_a && crate_vec[1] == crate_b)
                || (crate_vec[0] == crate_b && crate_vec[1] == crate_a)
        })
}

/// Whether `crates` overlaps either side of any known migration pair.
fn touches_migration_pair(validator: &RefactoringValidator, crates: &HashSet<String>) -> bool {
    validator
        .known_migration_pairs
        .iter()
        .any(|(crate_a, crate_b)| crates.contains(crate_a) || crates.contains(crate_b))
}

/// Categorize duplicate severity based on known patterns
fn categorize_duplicate_severity(
    validator: &RefactoringValidator,
    type_name: &str,
    crates: &HashSet<String>,
) -> Severity {
    // Intentionally duplicated utility types are informational.
    if validator.utility_types.contains(type_name) {
        return Severity::Info;
    }

    // Known migration pairs are an expected, temporary duplication.
    if is_known_migration_pair(validator, crates) {
        return Severity::Info;
    }

    // A migration-suffixed type touching a known pair is migration-related.
    if MIGRATION_TYPE_SUFFIXES
        .iter()
        .any(|p| type_name.ends_with(p))
        && touches_migration_pair(validator, crates)
    {
        return Severity::Warning;
    }

    Severity::Warning
}
