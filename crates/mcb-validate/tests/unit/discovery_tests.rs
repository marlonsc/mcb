//! Discovery tests — run every validator against the full fixture workspace.
//!
//! These tests catalog ALL violations each validator finds. Run with:
//! ```
//! cargo test -p mcb-validate --test unit -- discovery --nocapture
//! ```
//!
//! The output is used to set exact assertion counts in the real test files.

use mcb_validate::{
    AsyncPatternValidator, DocumentationValidator, ErrorBoundaryValidator,
    ImplementationQualityValidator, KissValidator, OrganizationValidator, PatternValidator,
    PerformanceValidator, QualityValidator, RefactoringValidator, SolidValidator, TestValidator,
};

use crate::test_constants::*;
use crate::test_utils::*;

/// Helper: sets up the full fixture workspace with all 4 crates.
fn full_workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE])
}

// ─────────────────────────────────────────────────────────────────────────────
// QualityValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_quality_violations() {
    let (_temp, root) = full_workspace();
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!(
        "\n=== QualityValidator: {} violations ===",
        violations.len()
    );
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    assert!(
        !violations.is_empty(),
        "QualityValidator should find violations in fixtures"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// KissValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_kiss_violations() {
    let (_temp, root) = full_workspace();
    let validator = KissValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!("\n=== KissValidator: {} violations ===", violations.len());
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    assert!(
        !violations.is_empty(),
        "KissValidator should find violations in fixtures"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AsyncPatternValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_async_violations() {
    let (_temp, root) = full_workspace();
    let validator = AsyncPatternValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!(
        "\n=== AsyncPatternValidator: {} violations ===",
        violations.len()
    );
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    assert!(
        !violations.is_empty(),
        "AsyncPatternValidator should find violations in fixtures"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DocumentationValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_documentation_violations() {
    let (_temp, root) = full_workspace();
    let validator = DocumentationValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!(
        "\n=== DocumentationValidator: {} violations ===",
        violations.len()
    );
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    // Documentation may or may not find violations depending on patterns
}

// ─────────────────────────────────────────────────────────────────────────────
// PerformanceValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_performance_violations() {
    let (_temp, root) = full_workspace();
    let validator = PerformanceValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!(
        "\n=== PerformanceValidator: {} violations ===",
        violations.len()
    );
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    // May return 0 — patterns might not match fixture code
}

// ─────────────────────────────────────────────────────────────────────────────
// ImplementationQualityValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_implementation_violations() {
    let (_temp, root) = full_workspace();
    let validator = ImplementationQualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!(
        "\n=== ImplementationQualityValidator: {} violations ===",
        violations.len()
    );
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    assert!(
        !violations.is_empty(),
        "ImplementationQualityValidator should find empty methods / todo macros"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// OrganizationValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_organization_violations() {
    let (_temp, root) = full_workspace();
    let validator = OrganizationValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!(
        "\n=== OrganizationValidator: {} violations ===",
        violations.len()
    );
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    assert!(
        !violations.is_empty(),
        "OrganizationValidator should find magic number violations"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SolidValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_solid_violations() {
    let (_temp, root) = full_workspace();
    let validator = SolidValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!("\n=== SolidValidator: {} violations ===", violations.len());
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    assert!(
        !violations.is_empty(),
        "SolidValidator should find ISP / LSP violations in my-domain"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// PatternValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_pattern_violations() {
    let (_temp, root) = full_workspace();
    let validator = PatternValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!(
        "\n=== PatternValidator: {} violations ===",
        violations.len()
    );
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    // May return 0 — generic patterns may not match fixture code
}

// ─────────────────────────────────────────────────────────────────────────────
// RefactoringValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_refactoring_violations() {
    let (_temp, root) = full_workspace();
    let validator = RefactoringValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!(
        "\n=== RefactoringValidator: {} violations ===",
        violations.len()
    );
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    // May or may not find duplicate types depending on heuristics
}

// ─────────────────────────────────────────────────────────────────────────────
// TestValidator (tests_org)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_test_org_violations() {
    let (_temp, root) = full_workspace();
    let validator = TestValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!("\n=== TestValidator: {} violations ===", violations.len());
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    assert!(
        !violations.is_empty(),
        "TestValidator should find inline test modules / naming violations"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ErrorBoundaryValidator
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_error_boundary_violations() {
    let (_temp, root) = full_workspace();
    let validator = ErrorBoundaryValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    eprintln!(
        "\n=== ErrorBoundaryValidator: {} violations ===",
        violations.len()
    );
    for (i, v) in violations.iter().enumerate() {
        eprintln!("  [{i}] {v}");
    }
    assert!(
        !violations.is_empty(),
        "ErrorBoundaryValidator should find infra errors in domain"
    );
}
