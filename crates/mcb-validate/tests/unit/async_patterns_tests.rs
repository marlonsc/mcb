//! Tests for Async Patterns Validation
//!
//! Uses fixture crate `my-test` which contains `async_file_processor` with
//! blocking I/O and `async_proper_handler` with correct tokio usage.

use mcb_validate::async_patterns::AsyncPatternValidator;

use crate::test_constants::TEST_CRATE;
use crate::test_utils::*;

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
