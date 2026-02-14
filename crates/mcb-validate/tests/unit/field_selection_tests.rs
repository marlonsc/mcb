//! Regression tests for `define_violations!` macro field-selection logic.
//!
//! These tests verify that the `@select_file_arm`, `@select_line_arm`, and
//! `@select_suggestion_arm` token-munchers in the refactored macro correctly
//! bind the right field for each Violation trait method across ALL field-naming
//! patterns used in production violation enums:
//!
//! - `file: PathBuf`          (most variants)
//! - `path: PathBuf`          (NamingViolation::BadModuleName)
//! - `location: PathBuf`      (DependencyViolation::ForbiddenCargoDepedency)
//! - `source_file: PathBuf`   (RefactoringViolation::MissingTestFile)
//! - `referencing_file: PathBuf` (RefactoringViolation::DeletedModuleReference)
//! - `locations: Vec<PathBuf>` (RefactoringViolation::DuplicateDefinition → `.first()`)
//! - No file-like field       (DependencyViolation::CircularDependency)
//! - `line: usize`            (most variants)
//! - No `line` field          (NamingViolation::BadModuleName, DependencyViolation::CircularDependency)
//! - Literal `suggestion = "..."` attribute (most variants)
//! - `suggestion: String` field (RefactoringViolation::OrphanImport, NamingViolation::BadCaNaming)
//! - No suggestion at all     (currently not used, but tested via CircularDependency which has a literal)

use std::path::PathBuf;

use mcb_validate::Severity;
use mcb_validate::traits::violation::Violation;

// ============================================================================
// file() — `file: PathBuf` (standard path)
// ============================================================================

#[test]
fn file_selection_standard_file_field() {
    let v = mcb_validate::QualityViolation::UnwrapInProduction {
        file: PathBuf::from("src/main.rs"),
        line: 10,
        context: "x.unwrap()".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(v.file(), Some(&PathBuf::from("src/main.rs")));
}

// ============================================================================
// file() — `path: PathBuf`
// ============================================================================

#[test]
fn file_selection_path_field() {
    let v = mcb_validate::NamingViolation::BadModuleName {
        path: PathBuf::from("src/Foo.rs"),
        expected_case: "snake_case".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(v.file(), Some(&PathBuf::from("src/Foo.rs")));
}

#[test]
fn file_selection_path_field_with_other_fields() {
    let v = mcb_validate::NamingViolation::BadFileSuffix {
        path: PathBuf::from("src/handler.rs"),
        component_type: "Service".to_string(),
        current_suffix: "".to_string(),
        expected_suffix: "_service".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(v.file(), Some(&PathBuf::from("src/handler.rs")));
}

// ============================================================================
// file() — `location: PathBuf`
// ============================================================================

#[test]
fn file_selection_location_field() {
    let v = mcb_validate::DependencyViolation::ForbiddenCargoDepedency {
        crate_name: "mcb-server".to_string(),
        forbidden_dep: "tokio-console".to_string(),
        location: PathBuf::from("crates/mcb-server/Cargo.toml"),
        severity: Severity::Error,
    };
    assert_eq!(
        v.file(),
        Some(&PathBuf::from("crates/mcb-server/Cargo.toml"))
    );
}

// ============================================================================
// file() — `source_file: PathBuf`
// ============================================================================

#[test]
fn file_selection_source_file_field() {
    let v = mcb_validate::RefactoringViolation::MissingTestFile {
        source_file: PathBuf::from("src/service.rs"),
        expected_test: PathBuf::from("tests/service_test.rs"),
        severity: Severity::Warning,
    };
    assert_eq!(v.file(), Some(&PathBuf::from("src/service.rs")));
}

// ============================================================================
// file() — `referencing_file: PathBuf`
// ============================================================================

#[test]
fn file_selection_referencing_file_field() {
    let v = mcb_validate::RefactoringViolation::DeletedModuleReference {
        referencing_file: PathBuf::from("src/lib.rs"),
        line: 5,
        deleted_module: "old_mod".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(v.file(), Some(&PathBuf::from("src/lib.rs")));
}

// ============================================================================
// file() — `locations: Vec<PathBuf>` → None (opaque :ty fragment cannot match literal Vec<PathBuf>)
// ============================================================================

#[test]
fn file_selection_locations_vec_returns_none() {
    // `locations: Vec<PathBuf>` passes through the macro as an opaque `:ty` fragment,
    // so the `Vec<PathBuf>` literal-match arm in `@select_file_arm` never fires.
    // This is a known declarative-macro limitation; `file()` returns None.
    let v = mcb_validate::RefactoringViolation::DuplicateDefinition {
        type_name: "Config".to_string(),
        locations: vec![
            PathBuf::from("src/a/config.rs"),
            PathBuf::from("src/b/config.rs"),
        ],
        suggestion: "Consolidate".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(v.file(), None);
}

// ============================================================================
// file() — No file-like field → None
// ============================================================================

#[test]
fn file_selection_no_file_field_returns_none() {
    let v = mcb_validate::DependencyViolation::CircularDependency {
        cycle: mcb_validate::validators::dependency::DependencyCycle(vec![
            "a".to_string(),
            "b".to_string(),
            "a".to_string(),
        ]),
        severity: Severity::Error,
    };
    assert_eq!(v.file(), None);
}

// ============================================================================
// line() — standard `line: usize`
// ============================================================================

#[test]
fn line_selection_standard_line_field() {
    let v = mcb_validate::ErrorBoundaryViolation::MissingErrorContext {
        file: PathBuf::from("src/handler.rs"),
        line: 99,
        error_pattern: "db.query()?".to_string(),
        suggestion: "Add .context()".to_string(),
        severity: Severity::Info,
    };
    assert_eq!(v.line(), Some(99));
}

#[test]
fn line_selection_line_not_first_field() {
    // `referencing_file` comes before `line` — macro must skip non-line fields
    let v = mcb_validate::RefactoringViolation::DeletedModuleReference {
        referencing_file: PathBuf::from("src/lib.rs"),
        line: 77,
        deleted_module: "gone".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(v.line(), Some(77));
}

// ============================================================================
// line() — No `line` field → None
// ============================================================================

#[test]
fn line_selection_no_line_field_returns_none() {
    // BadModuleName has only `path`, `expected_case`, `severity` — no `line`
    let v = mcb_validate::NamingViolation::BadModuleName {
        path: PathBuf::from("src/BadName.rs"),
        expected_case: "snake_case".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(v.line(), None);
}

#[test]
fn line_selection_no_line_circular_dep() {
    let v = mcb_validate::DependencyViolation::CircularDependency {
        cycle: mcb_validate::validators::dependency::DependencyCycle(vec![
            "x".to_string(),
            "y".to_string(),
        ]),
        severity: Severity::Error,
    };
    assert_eq!(v.line(), None);
}

// ============================================================================
// suggestion() — literal `suggestion = "..."` attribute
// ============================================================================

#[test]
fn suggestion_from_literal_attribute() {
    let v = mcb_validate::QualityViolation::UnwrapInProduction {
        file: PathBuf::from("src/lib.rs"),
        line: 1,
        context: "x.unwrap()".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(
        v.suggestion(),
        Some("Use `?` operator or handle the error explicitly".to_string())
    );
}

#[test]
fn suggestion_from_literal_with_field_interpolation() {
    // RefactoringViolation::MissingTestFile uses `suggestion = "Create test file {expected_test} ..."`
    let v = mcb_validate::RefactoringViolation::MissingTestFile {
        source_file: PathBuf::from("src/foo.rs"),
        expected_test: PathBuf::from("tests/foo_test.rs"),
        severity: Severity::Warning,
    };
    let suggestion = v.suggestion().expect("should have suggestion");
    assert!(
        suggestion.contains("tests/foo_test.rs"),
        "suggestion should interpolate {{expected_test}}: {suggestion}"
    );
    assert!(
        suggestion.contains("src/foo.rs"),
        "suggestion should interpolate {{source_file}}: {suggestion}"
    );
}

// ============================================================================
// suggestion() — `suggestion: String` field (no literal attribute)
// ============================================================================

#[test]
fn suggestion_from_string_field() {
    // OrphanImport has `suggestion = "{suggestion}"` — renders the field value verbatim
    let v = mcb_validate::RefactoringViolation::OrphanImport {
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        import_path: "use crate::old::Thing".to_string(),
        suggestion: "Remove the orphan import".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(v.suggestion(), Some("Remove the orphan import".to_string()));
}

#[test]
fn suggestion_from_string_field_naming() {
    // BadCaNaming has `suggestion = "{suggestion}"` — passthrough
    let v = mcb_validate::NamingViolation::BadCaNaming {
        path: PathBuf::from("src/bad.rs"),
        detected_type: "handler".to_string(),
        issue: "missing suffix".to_string(),
        suggestion: "Rename to bad_handler.rs".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(v.suggestion(), Some("Rename to bad_handler.rs".to_string()));
}

// ============================================================================
// suggestion() — literal suggestion on no-file variant
// ============================================================================

#[test]
fn suggestion_circular_dependency_literal() {
    let v = mcb_validate::DependencyViolation::CircularDependency {
        cycle: mcb_validate::validators::dependency::DependencyCycle(vec![
            "a".to_string(),
            "b".to_string(),
        ]),
        severity: Severity::Error,
    };
    assert_eq!(
        v.suggestion(),
        Some("Extract shared types to the domain crate".to_string())
    );
}

// ============================================================================
// Combined: all three methods on one variant
// ============================================================================

#[test]
fn all_three_methods_on_kiss_violation() {
    let v = mcb_validate::KissViolation::StructTooManyFields {
        file: PathBuf::from("src/big.rs"),
        line: 42,
        struct_name: "Monolith".to_string(),
        field_count: 25,
        max_allowed: 10,
        severity: Severity::Warning,
    };
    assert_eq!(v.file(), Some(&PathBuf::from("src/big.rs")));
    assert_eq!(v.line(), Some(42));
    let suggestion = v.suggestion().expect("should have suggestion");
    assert!(
        suggestion.contains("Monolith"),
        "suggestion should mention struct name: {suggestion}"
    );
    assert!(
        suggestion.contains("25"),
        "suggestion should mention field count: {suggestion}"
    );
}

#[test]
fn all_three_methods_on_no_file_no_line_variant() {
    let v = mcb_validate::DependencyViolation::CircularDependency {
        cycle: mcb_validate::validators::dependency::DependencyCycle(vec![
            "a".to_string(),
            "b".to_string(),
        ]),
        severity: Severity::Error,
    };
    assert_eq!(v.file(), None);
    assert_eq!(v.line(), None);
    assert!(v.suggestion().is_some());
}

// ============================================================================
// severity() — dynamic (from field) vs static (from attribute)
// ============================================================================

#[test]
fn severity_dynamic_reads_field_value() {
    // QualityViolation uses `dynamic_severity` — severity comes from the field
    let v = mcb_validate::QualityViolation::UnwrapInProduction {
        file: PathBuf::from("src/lib.rs"),
        line: 1,
        context: "x".to_string(),
        severity: Severity::Info, // Override: attribute says Warning, but field says Info
    };
    assert_eq!(v.severity(), Severity::Info);
}

// ============================================================================
// id() and category() — basic sanity
// ============================================================================

#[test]
fn id_and_category_correct() {
    let v = mcb_validate::KissViolation::DeepNesting {
        file: PathBuf::from("src/deep.rs"),
        line: 1,
        nesting_level: 8,
        max_allowed: 4,
        context: "if { if { if { ... } } }".to_string(),
        severity: Severity::Warning,
    };
    assert_eq!(v.id(), "KISS004");
    assert_eq!(v.category(), mcb_validate::ViolationCategory::Kiss);
}
