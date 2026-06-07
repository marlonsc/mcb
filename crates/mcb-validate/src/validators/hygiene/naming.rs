//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::filters::LanguageId;
use crate::scan::for_each_file_under_root;
use crate::{Result, Severity, ValidationConfig};

use super::violation::HygieneViolation;
use crate::ValidationConfigExt;

fn expected_naming_for_parent(
    parent_dir: &str,
    file_name: &str,
    file_full: &str,
) -> Option<(String, Severity)> {
    match parent_dir {
        "unit" => (!file_name.ends_with("_tests")).then(|| {
            (
                format!("{file_name}_tests.rs (unit tests must end with _tests)"),
                Severity::Warning,
            )
        }),
        "integration" => {
            let ok = ["integration", "workflow"]
                .iter()
                .any(|kw| file_name.contains(kw) || file_name.ends_with(&format!("_{kw}")));
            (!ok).then(|| {
                (
                    format!(
                        "{file_name}_integration.rs or {file_name}_workflow.rs (integration tests should indicate their purpose)"
                    ),
                    Severity::Info,
                )
            })
        }
        "e2e" => {
            let ok = ["e2e", "end_to_end"]
                .iter()
                .any(|kw| file_name.contains(kw))
                || file_name.starts_with("test_");
            (!ok).then(|| {
                (
                    format!(
                        "{file_name}_e2e.rs or test_{file_name}.rs (e2e tests should indicate they're end-to-end)"
                    ),
                    Severity::Info,
                )
            })
        }
        "tests" => (!matches!(
            file_full,
            "lib.rs" | "mod.rs" | "utils.rs" | "unit.rs" | "integration.rs" | "e2e.rs"
        ))
        .then(|| {
            (
                "Move to a subdirectory (e.g., tests/unit/)".to_owned(),
                Severity::Warning,
            )
        }),
        _ => None,
    }
}

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

            let file_full = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if let Some((suggestion, severity)) =
                expected_naming_for_parent(parent_dir, file_name, file_full)
            {
                violations.push(HygieneViolation::BadTestFileName {
                    file: path.clone(),
                    suggestion,
                    severity,
                });
            }

            Ok(())
        })?;
    }

    Ok(violations)
}
