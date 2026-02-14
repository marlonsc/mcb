use crate::scan::for_each_rs_under_root;
use crate::{Result, Severity, ValidationConfig};

use super::violation::HygieneViolation;

/// Checks test file naming conventions and directory structure compliance.
pub fn validate_test_naming(config: &ValidationConfig) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();

    for crate_dir in config.get_source_dirs()? {
        let tests_dir = crate_dir.join("tests");
        if !tests_dir.exists() {
            continue;
        }

        for_each_rs_under_root(config, &tests_dir, |path| {
            let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

            // Skip lib.rs and mod.rs
            if file_name == "lib" || file_name == "mod" {
                return Ok(());
            }

            // Skip test utility files (mocks, fixtures, helpers)
            let Some(path_str) = path.to_str() else {
                return Ok(());
            };
            if path_str.contains("test_utils")
                || file_name.contains("mock")
                || file_name.contains("fixture")
                || file_name.contains("helper")
            {
                return Ok(());
            }

            // Check directory-based naming conventions
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
                            file: path.to_path_buf(),
                            suggestion: format!(
                                "{file_name}_tests.rs (unit tests must end with _tests)"
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
                "integration" => {
                    // Integration tests can be more flexible but should indicate their purpose
                    let is_valid_integration = file_name.contains("integration")
                        || file_name.contains("workflow")
                        || file_name.ends_with("_integration")
                        || file_name.ends_with("_workflow");

                    if !is_valid_integration {
                        violations.push(HygieneViolation::BadTestFileName {
                            file: path.to_path_buf(),
                            suggestion: format!("{file_name}_integration.rs or {file_name}_workflow.rs (integration tests should indicate their purpose)"),
                            severity: Severity::Info,
                        });
                    }
                }
                "e2e" => {
                    // E2E tests should clearly indicate they're end-to-end
                    let is_valid_e2e = file_name.contains("e2e")
                        || file_name.contains("end_to_end")
                        || file_name.starts_with("test_");

                    if !is_valid_e2e {
                        violations.push(HygieneViolation::BadTestFileName {
                            file: path.to_path_buf(),
                            suggestion: format!("{file_name}_e2e.rs or test_{file_name}.rs (e2e tests should indicate they're end-to-end)"),
                            severity: Severity::Info,
                        });
                    }
                }
                "tests" => {
                    // Files directly in tests/ directory (not in any subdirectory)
                    // are violations UNLESS they are entry points
                    let file_full = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !matches!(
                        file_full,
                        "lib.rs"
                            | "mod.rs"
                            | "test_utils.rs"
                            | "unit.rs"
                            | "integration.rs"
                            | "e2e.rs"
                    ) {
                        violations.push(HygieneViolation::BadTestFileName {
                            file: path.to_path_buf(),
                            suggestion: "Move to a subdirectory (e.g., tests/unit/)".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
                _ => {
                    // Files in subdirectories are allowed (module structure)
                    // No violation
                }
            }
            Ok(())
        })?;
    }

    Ok(violations)
}
