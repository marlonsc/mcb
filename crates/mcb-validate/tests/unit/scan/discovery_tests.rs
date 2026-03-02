//! Discovery tests — run every validator against the full fixture workspace.
//!
//! These tests catalog ALL violations each validator finds. Run with:
//! ```
//! cargo test -p mcb-validate --test unit -- discovery --nocapture
//! ```
//!
//! The output is used to set exact assertion counts in the real test files.

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
                expect_validation_ok(self.name(), run_named_validator(root, "quality"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Kiss => {
                expect_validation_ok(self.name(), run_named_validator(root, "kiss"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::AsyncPattern => {
                expect_validation_ok(self.name(), run_named_validator(root, "async_patterns"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Documentation => {
                expect_validation_ok(self.name(), run_named_validator(root, "documentation"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Performance => {
                expect_validation_ok(self.name(), run_named_validator(root, "performance"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::ImplementationQuality => {
                expect_validation_ok(self.name(), run_named_validator(root, "implementation"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Organization => {
                expect_validation_ok(self.name(), run_named_validator(root, "organization"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Solid => {
                expect_validation_ok(self.name(), run_named_validator(root, "solid"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Pattern => {
                expect_validation_ok(self.name(), run_named_validator(root, "patterns"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Refactoring => {
                expect_validation_ok(self.name(), run_named_validator(root, "refactoring"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::Hygiene => {
                expect_validation_ok(self.name(), run_named_validator(root, "hygiene"))
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect()
            }
            DiscoveryValidatorKind::ErrorBoundary => {
                expect_validation_ok(self.name(), run_named_validator(root, "error_boundary"))
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
