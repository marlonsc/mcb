use super::violation::HygieneViolation;
use crate::{Result, Severity, ValidationConfig};

/// Validates that tests are properly organized in subdirectories (unit/, integration/, e2e/).
pub fn validate_test_directory_structure(
    config: &ValidationConfig,
) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();

    for crate_dir in config.get_source_dirs()? {
        let tests_dir = crate_dir.join("tests");
        if !tests_dir.exists() {
            continue;
        }

        // Check that at least unit/ or integration/ exists (e2e/ is optional)
        let unit_exists = tests_dir.join("unit").exists();
        let integration_exists = tests_dir.join("integration").exists();

        // Only flag if NEITHER unit/ nor integration/ exist and there are test files
        if !unit_exists && !integration_exists {
            let has_test_files = std::fs::read_dir(&tests_dir)
                .map(|entries| {
                    entries.filter_map(std::result::Result::ok).any(|e| {
                        let path = e.path();
                        path.is_file()
                            && path.extension().and_then(|x| x.to_str()) == Some("rs")
                            && !matches!(
                                path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                                "lib.rs" | "mod.rs" | "test_utils.rs"
                            )
                    })
                })
                .unwrap_or(false);

            if has_test_files {
                violations.push(HygieneViolation::BadTestFileName {
                    file: tests_dir.clone(),
                    suggestion: "Create tests/unit/ or tests/integration/ directory".to_string(),
                    severity: Severity::Warning,
                });
            }
        }

        // Check for test files directly in tests/ directory (not in subdirs)
        for entry in std::fs::read_dir(&tests_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Skip non-Rust files
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip allowed files in root tests directory
            // These are: lib.rs, mod.rs, test_utils.rs, and entry points for test subdirectories
            if matches!(
                file_name,
                "lib.rs" | "mod.rs" | "test_utils.rs" | "unit.rs" | "integration.rs" | "e2e.rs"
            ) {
                continue;
            }

            // Any other .rs file directly in tests/ is a violation
            violations.push(HygieneViolation::BadTestFileName {
                file: path,
                suggestion: "Move to tests/unit/, tests/integration/, or tests/e2e/ directory"
                    .to_string(),
                severity: Severity::Warning,
            });
        }
    }

    Ok(violations)
}
