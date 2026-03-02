//! Discovery tests — run every validator against the full fixture workspace.
//!
//! These tests catalog ALL violations each validator finds. Run with:
//! ```
//! cargo test -p mcb-validate --test unit -- discovery --nocapture
//! ```
//!
//! The output is used to set exact assertion counts in the real test files.

use rstest::rstest;

use mcb_domain::utils::test_constants::*;
use mcb_domain::utils::*;

fn full_workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE])
}

#[rstest]
#[case("quality", true)]
#[case("kiss", true)]
#[case("async_patterns", true)]
#[case("documentation", false)]
#[case("performance", false)]
#[case("implementation", true)]
#[case("organization", true)]
#[case("solid", true)]
#[case("patterns", false)]
#[case("refactoring", false)]
#[case("hygiene", true)]
#[case("error_boundary", true)]
fn discover_violations(#[case] validator_name: &str, #[case] should_have_violations: bool) {
    let (_temp, root) = full_workspace();
    let violations = run_named_validator(&root, validator_name)
        .unwrap_or_else(|e| panic!("{validator_name} failed to validate fixture workspace: {e}"));

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
