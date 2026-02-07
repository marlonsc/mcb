//! Tests for Async Patterns Validation
//!
//! Discovery found 4 violations in the full workspace:
//! - my-server: std::fs::read in delete_user() (blocking I/O in async)
//! - my-test: std::fs::read_to_string in async_file_processor() (blocking I/O)
//! - my-test: std::thread::sleep in async_file_processor() (blocking sleep ×2 patterns)

use mcb_validate::async_patterns::AsyncPatternValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_async_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = AsyncPatternValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violation_count(&violations, 4, "AsyncPatternValidator full workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-method tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_blocking_in_async_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has async_file_processor with:
    //   - std::fs::read_to_string (blocking I/O)
    //   - std::thread::sleep (blocking sleep)
    let validator = AsyncPatternValidator::new(&root);
    let violations = validator.validate_blocking_in_async().unwrap();

    assert_min_violations(&violations, 1, "blocking calls in async fixture crate");
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_proper_async_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
use tokio::time;

pub async fn good_async() {
    time::sleep(std::time::Duration::from_secs(1)).await;
}
",
    );

    let validator = AsyncPatternValidator::new(&root);
    let violations = validator.validate_blocking_in_async().unwrap();

    assert_no_violations(&violations, "Proper async should pass");
}
