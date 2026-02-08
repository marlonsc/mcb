//! Tests for Async Pattern Validation
//!
//! Validates `AsyncPatternValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_validate::AsyncPatternValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_async_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = AsyncPatternValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violations_exact(
        &violations,
        &[
            // std::fs::read in async handler
            (
                "my-server/src/handlers/user_handler.rs",
                106,
                "BlockingInAsync",
            ),
            // std::fs::read in async fn
            ("my-test/src/lib.rs", 102, "BlockingInAsync"),
            // std::thread::sleep in async fn
            ("my-test/src/lib.rs", 105, "BlockingInAsync"),
            // thread::sleep in async fn (same line, different pattern)
            ("my-test/src/lib.rs", 105, "BlockingInAsync"),
            // std::sync::Mutex in OveruseExample struct (PERF003 also triggers ASYNC003)
            ("my-test/src/lib.rs", 169, "WrongMutexType"),
        ],
        "AsyncPatternValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_clean_async_code_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r#"
/// An async function using correct non-blocking I/O.
pub async fn fetch_data() -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::from("hello"))
}
"#,
    );
    let validator = AsyncPatternValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(&violations, "Clean async code should produce no violations");
}
