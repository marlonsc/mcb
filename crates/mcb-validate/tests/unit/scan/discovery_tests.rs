//! Discovery tests — run every validator against the full fixture workspace.
//!
//! These tests catalog ALL violations each validator finds. Run with:
//! ```
//! cargo test -p mcb-validate --test unit -- discovery --nocapture
//! ```
//!
//! The output is used to set exact assertion counts in the real test files.

use mcb_validate::{
    AsyncPatternValidator, DocumentationValidator, ErrorBoundaryValidator, HygieneValidator,
    ImplementationQualityValidator, KissValidator, OrganizationValidator, PatternValidator,
    PerformanceValidator, QualityValidator, RefactoringValidator, SolidValidator,
};
use rstest::rstest;

use crate::utils::test_constants::*;
use crate::utils::*;

/// Helper: sets up the full fixture workspace with all 4 crates.
fn full_workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE])
}

fn expect_validation_ok<T, E: std::fmt::Debug>(
    validator_name: &str,
    result: Result<Vec<T>, E>,
) -> Vec<T> {
    assert!(
        result.is_ok(),
        "{validator_name} failed to validate fixture workspace"
    );
    result.unwrap_or_default()
}

#[derive(Clone, Copy)]
enum DiscoveryValidatorKind {
    Quality,
    Kiss,
    AsyncPattern,
    Documentation,
    Performance,
    ImplementationQuality,
    Organization,
    Solid,
    Pattern,
    Refactoring,
    Hygiene,
    ErrorBoundary,
}

impl DiscoveryValidatorKind {
    fn name(self) -> &'static str {
        match self {
            DiscoveryValidatorKind::Quality => "QualityValidator",
            DiscoveryValidatorKind::Kiss => "KissValidator",
            DiscoveryValidatorKind::AsyncPattern => "AsyncPatternValidator",
            DiscoveryValidatorKind::Documentation => "DocumentationValidator",
            DiscoveryValidatorKind::Performance => "PerformanceValidator",
            DiscoveryValidatorKind::ImplementationQuality => "ImplementationQualityValidator",
            DiscoveryValidatorKind::Organization => "OrganizationValidator",
            DiscoveryValidatorKind::Solid => "SolidValidator",
            DiscoveryValidatorKind::Pattern => "PatternValidator",
            DiscoveryValidatorKind::Refactoring => "RefactoringValidator",
            DiscoveryValidatorKind::Hygiene => "HygieneValidator",
            DiscoveryValidatorKind::ErrorBoundary => "ErrorBoundaryValidator",
        }
    }

    fn validate(self, root: &std::path::Path) -> Vec<String> {
        match self {
            DiscoveryValidatorKind::Quality => {
                let validator = QualityValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Kiss => {
                let validator = KissValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::AsyncPattern => {
                let validator = AsyncPatternValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Documentation => {
                let validator = DocumentationValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Performance => {
                let validator = PerformanceValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::ImplementationQuality => {
                let validator = ImplementationQualityValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Organization => {
                let validator = OrganizationValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Solid => {
                let validator = SolidValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Pattern => {
                let validator = PatternValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Refactoring => {
                let validator = RefactoringValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Hygiene => {
                let validator = HygieneValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::ErrorBoundary => {
                let validator = ErrorBoundaryValidator::new(root);
                expect_validation_ok(self.name(), validator.validate_all())
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// QualityValidator
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[case(DiscoveryValidatorKind::Quality, true)]
#[case(DiscoveryValidatorKind::Kiss, true)]
#[case(DiscoveryValidatorKind::AsyncPattern, true)]
#[case(DiscoveryValidatorKind::Documentation, false)]
#[case(DiscoveryValidatorKind::Performance, false)]
#[case(DiscoveryValidatorKind::ImplementationQuality, true)]
#[case(DiscoveryValidatorKind::Organization, true)]
#[case(DiscoveryValidatorKind::Solid, true)]
#[case(DiscoveryValidatorKind::Pattern, false)]
#[case(DiscoveryValidatorKind::Refactoring, false)]
#[case(DiscoveryValidatorKind::Hygiene, true)]
#[case(DiscoveryValidatorKind::ErrorBoundary, true)]
fn discover_violations(
    #[case] validator_kind: DiscoveryValidatorKind,
    #[case] should_have_violations: bool,
) {
    let (_temp, root) = full_workspace();
    let violations = validator_kind.validate(&root);
    let validator_name = validator_kind.name();

    eprintln!(
        "\n=== {validator_name}: {} violations ===",
        violations.len()
    );
    for (i, violation) in violations.iter().enumerate() {
        eprintln!("  [{i}] {violation}");
    }

    if should_have_violations {
        assert!(
            !violations.is_empty(),
            "{validator_name} should find violations in fixtures"
        );
    }
}
