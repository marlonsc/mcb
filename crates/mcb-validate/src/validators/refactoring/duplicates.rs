use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};

use super::RefactoringValidator;
use super::violation::RefactoringViolation;

/// Check for same type defined in multiple locations
pub fn validate_duplicate_definitions(
    validator: &RefactoringValidator,
) -> Result<Vec<RefactoringViolation>> {
    let mut violations = Vec::new();
    let definition_pattern = compile_regex(
        r"(?:pub\s+)?(?:struct|trait|enum)\s+([A-Z][a-zA-Z0-9_]*)(?:\s*<|\s*\{|\s*;|\s*\(|\s+where)",
    )?;

    // Map: type_name -> Vec<file_path>
    let mut definitions: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for src_dir in validator.config.get_scan_dirs()? {
        if validator.should_skip_crate(&src_dir) {
            continue;
        }

        for_each_scan_file(
            &validator.config,
            Some(LanguageId::Rust),
            false,
            |entry, candidate_src_dir| {
                let path = &entry.absolute_path;
                if candidate_src_dir != src_dir {
                    return Ok(());
                }

                let Some(path_str) = path.to_str() else {
                    return Ok(());
                };

                // Skip test files and archived directories
                if path_str.contains("/tests/")
                    || path_str.contains("_test.rs")
                    || path_str.contains(".archived")
                    || path_str.contains(".bak")
                {
                    return Ok(());
                }

                let content = std::fs::read_to_string(path)?;

                for cap in definition_pattern.captures_iter(&content) {
                    let type_name = cap.get(1).map_or("", |m| m.as_str());

                    // Skip generic names that are expected to appear in multiple places
                    if validator.generic_type_names.contains(type_name) {
                        continue;
                    }

                    definitions
                        .entry(type_name.to_owned())
                        .or_default()
                        .push(path.clone());
                }

                Ok(())
            },
        )?;
    }

    // Find duplicates (same name in different files)
    for (type_name, locations) in definitions {
        if locations.len() > 1 {
            // Check if duplicates are in different crates
            let crates: HashSet<String> = locations
                .iter()
                .filter_map(|p| {
                    p.to_str()?
                        .split("/crates/")
                        .nth(1)
                        .and_then(|s| s.split('/').next())
                        .map(std::string::ToString::to_string)
                })
                .collect();

            if crates.len() > 1 {
                // Cross-crate duplicate - categorize by pattern
                let severity = categorize_duplicate_severity(validator, &type_name, &crates);

                let suggestion = match severity {
                    Severity::Info => format!(
                        "Type '{type_name}' exists in {crates:?}. This is a known migration pattern - consolidate when migration completes."
                    ),
                    Severity::Warning => format!(
                        "Type '{type_name}' is defined in {crates:?}. Consider consolidating to one location."
                    ),
                    Severity::Error => format!(
                        "Type '{type_name}' is unexpectedly defined in multiple crates: {crates:?}. This requires immediate consolidation."
                    ),
                };

                violations.push(RefactoringViolation::DuplicateDefinition {
                    type_name: type_name.clone(),
                    locations: locations.clone(),
                    suggestion,
                    severity,
                });
            } else if locations.len() >= 2 {
                // Same crate but duplicates - check if in different directories
                let dirs: HashSet<String> = locations
                    .iter()
                    .filter_map(|p| {
                        let parent = p.parent()?;
                        let parent_str = parent.to_str()?;
                        Some(parent_str.to_owned())
                    })
                    .collect();

                // Only flag if duplicates are in different directories (not just mod.rs + impl.rs)
                if dirs.len() >= 2 {
                    violations.push(RefactoringViolation::DuplicateDefinition {
                        type_name: type_name.clone(),
                        locations: locations.clone(),
                        suggestion: format!(
                            "Type '{}' is defined {} times in different directories within the same crate. Consolidate to single location.",
                            type_name,
                            locations.len()
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }

    Ok(violations)
}

/// Categorize duplicate severity based on known patterns
fn categorize_duplicate_severity(
    validator: &RefactoringValidator,
    type_name: &str,
    crates: &HashSet<String>,
) -> Severity {
    // Check if this is an intentionally duplicated utility type
    if validator.utility_types.contains(type_name) {
        return Severity::Info;
    }

    // Check if the crates match a known migration pattern
    let crate_vec: Vec<&String> = crates.iter().collect();
    if crate_vec.len() == 2 {
        for (crate_a, crate_b) in &validator.known_migration_pairs {
            if (crate_vec[0].as_str() == *crate_a && crate_vec[1].as_str() == *crate_b)
                || (crate_vec[0].as_str() == *crate_b && crate_vec[1].as_str() == *crate_a)
            {
                // This is a known migration pair - downgrade to Info
                return Severity::Info;
            }
        }
    }

    // Check for patterns that suggest migration in progress
    // Types ending with Provider, Processor, etc. between known pairs
    let migration_type_patterns = [
        "Provider",
        "Processor",
        "Handler",
        "Service",
        "Repository",
        "Adapter",
        "Factory",
        "Publisher",
        "Subscriber",
    ];

    if migration_type_patterns
        .iter()
        .any(|p| type_name.ends_with(p))
    {
        // Check if any known migration pair is involved
        for (crate_a, crate_b) in &validator.known_migration_pairs {
            if crates.contains(crate_a) || crates.contains(crate_b) {
                return Severity::Warning; // Migration-related, but should be tracked
            }
        }
    }

    Severity::Warning
}
