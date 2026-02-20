use super::violation::HygieneViolation;
use crate::filters::LanguageId;
use crate::run_context::ValidationRunContext;
use crate::{Result, Severity, ValidationConfig};

/// Validates that tests are properly organized in subdirectories (unit/, integration/, e2e/).
///
/// # Errors
///
/// Returns an error if directory scanning or file reading fails.
pub fn validate_test_directory_structure(
    config: &ValidationConfig,
) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();
    let context = ValidationRunContext::active_or_build(config)?;

    for crate_dir in config.get_source_dirs()? {
        let tests_dir = crate_dir.join("tests");
        if !tests_dir.exists() {
            continue;
        }

        // Check that at least unit/ or integration/ exists (e2e/ is optional)
        let unit_exists = tests_dir.join("unit").exists();
        let integration_exists = tests_dir.join("integration").exists();
        let Ok(normalized_tests_dir) = std::fs::canonicalize(&tests_dir) else {
            continue;
        };
        let rust_entries: Vec<_> = context
            .file_inventory()
            .iter()
            .filter(|entry| {
                entry.absolute_path.starts_with(&normalized_tests_dir)
                    && entry.detected_language == Some(LanguageId::Rust)
            })
            .collect();

        // Only flag if NEITHER unit/ nor integration/ exist and there are test files
        if !unit_exists && !integration_exists {
            let has_test_files = rust_entries.iter().any(|entry| {
                let path = &entry.absolute_path;
                path.parent() == Some(normalized_tests_dir.as_path())
                    && !matches!(
                        path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                        "lib.rs" | "mod.rs" | "utils.rs"
                    )
            });

            if has_test_files {
                violations.push(HygieneViolation::BadTestFileName {
                    file: tests_dir.clone(),
                    suggestion: "Create tests/unit/ or tests/integration/ directory".to_owned(),
                    severity: Severity::Warning,
                });
            }
        }

        // Check for test files directly in tests/ directory (not in subdirs)
        for entry in rust_entries
            .iter()
            .filter(|entry| entry.absolute_path.parent() == Some(normalized_tests_dir.as_path()))
        {
            let path = &entry.absolute_path;

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip allowed files in root tests directory
            // These are: lib.rs, mod.rs, utils.rs, and entry points for test subdirectories
            if matches!(
                file_name,
                "lib.rs" | "mod.rs" | "utils.rs" | "unit.rs" | "integration.rs" | "e2e.rs"
            ) {
                continue;
            }

            // Any other .rs file directly in tests/ is a violation
            violations.push(HygieneViolation::BadTestFileName {
                file: path.clone(),
                suggestion: "Move to tests/unit/, tests/integration/, or tests/e2e/ directory"
                    .to_owned(),
                severity: Severity::Warning,
            });
        }
    }

    Ok(violations)
}
