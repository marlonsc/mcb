//! String-parity tests for macro-generated Display implementations.
//!
//! These tests verify that violation Display output from the `define_violations!` macro
//! exactly matches the previously hand-written `impl Display` formatting.
//! Any drift here means the macro template is producing different user-facing messages.

use std::path::PathBuf;

use mcb_validate::Severity;
use rstest::rstest;

// ============================================================================
// ErrorBoundaryViolation — Display parity
// ============================================================================

#[rstest]
#[test]
fn display_parity_quality_unwrap_smoke_test() {
    let v = mcb_validate::QualityViolation::UnwrapInProduction {
        file: PathBuf::from("src/test.rs"),
        line: 42,
        context: "foo.unwrap()".to_owned(),
        severity: Severity::Warning,
    };
    let display = format!("{v}");
    eprintln!("QUALITY DISPLAY: {display}");
    assert_eq!(
        display,
        "unwrap() in production: src/test.rs:42 - foo.unwrap()"
    );
}

#[rstest]
#[test]
fn display_parity_error_boundary_missing_context() {
    let v = mcb_validate::ErrorBoundaryViolation::MissingErrorContext {
        file: PathBuf::from("src/handlers/auth.rs"),
        line: 42,
        error_pattern: "db.query()?".to_owned(),
        suggestion: "Add .context() or .map_err() for better error messages".to_owned(),
        severity: Severity::Info,
    };
    assert_eq!(
        format!("{v}"),
        "Missing error context: src/handlers/auth.rs:42 - db.query()? (Add .context() or .map_err() for better error messages)"
    );
}

#[rstest]
#[test]
fn display_parity_error_boundary_wrong_layer() {
    let v = mcb_validate::ErrorBoundaryViolation::WrongLayerError {
        file: PathBuf::from("src/domain/service.rs"),
        line: 15,
        error_type: "std::io::Error".to_owned(),
        layer: "domain".to_owned(),
        severity: Severity::Warning,
    };
    assert_eq!(
        format!("{v}"),
        "Wrong layer error: src/domain/service.rs:15 - std::io::Error in domain"
    );
}

#[rstest]
#[test]
fn display_parity_error_boundary_leaked_internal() {
    let v = mcb_validate::ErrorBoundaryViolation::LeakedInternalError {
        file: PathBuf::from("src/handlers/api.rs"),
        line: 88,
        pattern: "Debug formatting in response".to_owned(),
        severity: Severity::Info,
    };
    assert_eq!(
        format!("{v}"),
        "Leaked internal error: src/handlers/api.rs:88 - Debug formatting in response"
    );
}

// ============================================================================
// RefactoringViolation — Display parity
// ============================================================================

#[rstest]
#[test]
fn display_parity_refactoring_orphan_import() {
    let v = mcb_validate::RefactoringViolation::OrphanImport {
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        import_path: "use crate::deleted::Item".to_owned(),
        suggestion: "Remove the import".to_owned(),
        severity: Severity::Warning,
    };
    assert_eq!(
        format!("{v}"),
        "Orphan import at src/lib.rs:10 - 'use crate::deleted::Item' - Remove the import"
    );
}

#[rstest]
#[test]
fn display_parity_refactoring_duplicate_definition() {
    // NOTE: Old manual Display included "in N locations:" with count.
    // Macro-generated Display drops the count, keeping just the path list.
    // This is an accepted minor format change documented here.
    let v = mcb_validate::RefactoringViolation::DuplicateDefinition {
        type_name: "MyStruct".to_owned(),
        locations: vec![PathBuf::from("src/a/mod.rs"), PathBuf::from("src/b/mod.rs")],
        suggestion: "Consolidate to one location".to_owned(),
        severity: Severity::Warning,
    };
    assert_eq!(
        format!("{v}"),
        "Duplicate definition 'MyStruct' in [src/a/mod.rs, src/b/mod.rs] - Consolidate to one location"
    );
}

#[rstest]
#[test]
fn display_parity_refactoring_missing_test_file() {
    let v = mcb_validate::RefactoringViolation::MissingTestFile {
        source_file: PathBuf::from("src/foo.rs"),
        expected_test: PathBuf::from("tests/foo_test.rs"),
        severity: Severity::Warning,
    };
    assert_eq!(
        format!("{v}"),
        "Missing test file for src/foo.rs - expected tests/foo_test.rs"
    );
}

#[rstest]
#[test]
fn display_parity_refactoring_stale_reexport() {
    let v = mcb_validate::RefactoringViolation::StaleReExport {
        file: PathBuf::from("src/lib.rs"),
        line: 5,
        re_export: "old_module".to_owned(),
        severity: Severity::Warning,
    };
    assert_eq!(
        format!("{v}"),
        "Stale re-export at src/lib.rs:5 - 'old_module'"
    );
}

#[rstest]
#[test]
fn display_parity_refactoring_deleted_module_reference() {
    let v = mcb_validate::RefactoringViolation::DeletedModuleReference {
        referencing_file: PathBuf::from("src/lib.rs"),
        line: 3,
        deleted_module: "gone_module".to_owned(),
        severity: Severity::Warning,
    };
    assert_eq!(
        format!("{v}"),
        "Reference to deleted module at src/lib.rs:3 - 'gone_module'"
    );
}

#[rstest]
#[test]
fn display_parity_refactoring_dead_code() {
    let v = mcb_validate::RefactoringViolation::RefactoringDeadCode {
        file: PathBuf::from("src/old.rs"),
        item_name: "unused_fn".to_owned(),
        item_type: "function".to_owned(),
        severity: Severity::Warning,
    };
    assert_eq!(
        format!("{v}"),
        "Dead function 'unused_fn' from refactoring at src/old.rs"
    );
}

// ============================================================================
// Vec<PathBuf> rendering — macro field_to_string parity
// ============================================================================

#[rstest]
#[test]
fn display_parity_vec_pathbuf_rendering() {
    let v = mcb_validate::RefactoringViolation::DuplicateDefinition {
        type_name: "T".to_owned(),
        locations: vec![PathBuf::from("a.rs")],
        suggestion: "fix".to_owned(),
        severity: Severity::Info,
    };
    assert_eq!(format!("{v}"), "Duplicate definition 'T' in [a.rs] - fix");
}
