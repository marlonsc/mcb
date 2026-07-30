//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use super::violation::HygieneViolation;
use crate::ValidationConfigExt;
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

        check_missing_test_subdirs(
            &tests_dir,
            &normalized_tests_dir,
            &rust_entries,
            &mut violations,
        );
        check_misplaced_root_test_files(&normalized_tests_dir, &rust_entries, &mut violations);
    }

    Ok(violations)
}

/// Returns `true` for files allowed to live directly in `tests/` (entry points
/// and shared helpers).
fn is_allowed_root_test_file(file_name: &str) -> bool {
    matches!(
        file_name,
        "lib.rs" | "mod.rs" | "utils.rs" | "unit.rs" | "integration.rs" | "e2e.rs"
    )
}

/// Flag the `tests/` directory when neither `unit/` nor `integration/` exist yet
/// real test files are present in its root.
fn check_missing_test_subdirs(
    tests_dir: &std::path::Path,
    normalized_tests_dir: &std::path::Path,
    rust_entries: &[&crate::run_context::InventoryEntry],
    violations: &mut Vec<HygieneViolation>,
) {
    if tests_dir.join("unit").exists() || tests_dir.join("integration").exists() {
        return;
    }
    let has_test_files = rust_entries.iter().any(|entry| {
        let path = &entry.absolute_path;
        path.parent() == Some(normalized_tests_dir)
            && !matches!(
                path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                "lib.rs" | "mod.rs" | "utils.rs"
            )
    });
    if has_test_files {
        violations.push(HygieneViolation::BadTestFileName {
            file: tests_dir.to_path_buf(),
            suggestion: "Create tests/unit/ or tests/integration/ directory".to_owned(),
            severity: Severity::Warning,
        });
    }
}

/// Flag any `.rs` test file that sits directly in `tests/` instead of a
/// `unit/`, `integration/`, or `e2e/` subdirectory.
fn check_misplaced_root_test_files(
    normalized_tests_dir: &std::path::Path,
    rust_entries: &[&crate::run_context::InventoryEntry],
    violations: &mut Vec<HygieneViolation>,
) {
    for entry in rust_entries
        .iter()
        .filter(|entry| entry.absolute_path.parent() == Some(normalized_tests_dir))
    {
        let path = &entry.absolute_path;
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if is_allowed_root_test_file(file_name) {
            continue;
        }
        violations.push(HygieneViolation::BadTestFileName {
            file: path.clone(),
            suggestion: "Move to tests/unit/, tests/integration/, or tests/e2e/ directory"
                .to_owned(),
            severity: Severity::Warning,
        });
    }
}
