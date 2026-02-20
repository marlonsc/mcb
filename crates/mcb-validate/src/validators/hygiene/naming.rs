//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::filters::LanguageId;
use crate::scan::for_each_file_under_root;
use crate::{Result, Severity, ValidationConfig};

use super::violation::HygieneViolation;

/// Checks test file naming conventions and directory structure compliance.
///
/// # Errors
///
/// Returns an error if directory enumeration or file scanning fails.
pub fn validate_test_naming(config: &ValidationConfig) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();

    for crate_dir in config.get_source_dirs()? {
        let tests_dir = crate_dir.join("tests");
        if !tests_dir.exists() {
            continue;
        }

        for_each_file_under_root(config, &tests_dir, Some(LanguageId::Rust), |entry| {
            let path = &entry.absolute_path;
            let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

            if ["lib", "mod"].contains(&file_name) {
                return Ok(());
            }

            // Skip test utility files (mocks, fixtures, helpers)
            let Some(path_str) = path.to_str() else {
                return Ok(());
            };
            if path_str.contains("utils")
                || ["mock", "fixture", "helper"]
                    .iter()
                    .any(|kw| file_name.contains(kw))
            {
                return Ok(());
            }

            let parent_dir = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");

            match parent_dir {
                "unit" => {
                    // Unit tests must follow [module]_tests.rs pattern
                    if !file_name.ends_with("_tests") {
                        violations.push(HygieneViolation::BadTestFileName {
                            file: path.clone(),
                            suggestion: format!(
                                "{file_name}_tests.rs (unit tests must end with _tests)"
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
                "integration" => {
                    let is_valid_integration = ["integration", "workflow"]
                        .iter()
                        .any(|kw| file_name.contains(kw) || file_name.ends_with(&format!("_{kw}")));

                    if !is_valid_integration {
                        violations.push(HygieneViolation::BadTestFileName {
                            file: path.clone(),
                            suggestion: format!("{file_name}_integration.rs or {file_name}_workflow.rs (integration tests should indicate their purpose)"),
                            severity: Severity::Info,
                        });
                    }
                }
                "e2e" => {
                    let is_valid_e2e = ["e2e", "end_to_end"]
                        .iter()
                        .any(|kw| file_name.contains(kw))
                        || file_name.starts_with("test_");

                    if !is_valid_e2e {
                        violations.push(HygieneViolation::BadTestFileName {
                            file: path.clone(),
                            suggestion: format!("{file_name}_e2e.rs or test_{file_name}.rs (e2e tests should indicate they're end-to-end)"),
                            severity: Severity::Info,
                        });
                    }
                }
                "tests" => {
                    let file_full = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !matches!(
                        file_full,
                        "lib.rs" | "mod.rs" | "utils.rs" | "unit.rs" | "integration.rs" | "e2e.rs"
                    ) {
                        violations.push(HygieneViolation::BadTestFileName {
                            file: path.clone(),
                            suggestion: "Move to a subdirectory (e.g., tests/unit/)".to_owned(),
                            severity: Severity::Warning,
                        });
                    }
                }
                _ => {}
            }
            Ok(())
        })?;
    }

    Ok(violations)
}
