//! Tests for Async Pattern Validation
//!
//! Validates `AsyncPatternValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use crate::utils::test_constants::*;
use crate::utils::*;
use rstest::rstest;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_async_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let violations = run_named_validator(&root, "async_patterns").unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── BlockingInAsync ────────────────────────────────────────
            // std::fs::read in server handler
            (
                "my-server/src/handlers/user_handler.rs",
                106,
                "BlockingInAsync",
            ),
            // std::fs::read in async fn
            ("my-test/src/lib.rs", 102, "BlockingInAsync"),
            // thread::sleep in async fn (two patterns match same line)
            ("my-test/src/lib.rs", 105, "BlockingInAsync"),
            ("my-test/src/lib.rs", 105, "BlockingInAsync"),
            // ── BlockOnInAsync ─────────────────────────────────────────
            // async fn def triggers block_on detection
            ("my-test/src/lib.rs", 293, "BlockOnInAsync"),
            // actual block_on call
            ("my-test/src/lib.rs", 295, "BlockOnInAsync"),
            // ── WrongMutexType ─────────────────────────────────────────
            // std::sync::Mutex in OveruseExample struct (also triggers PERF003)
            ("my-test/src/lib.rs", 169, "WrongMutexType"),
        ],
        "AsyncPatternValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_clean_async_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r#"
//! Clean async code.
/// A clean async function using proper async APIs.
pub async fn fetch_data(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(format!("fetched from {}", url))
}
"#,
    );
    let violations = run_named_validator(&root, "async_patterns").unwrap();

    assert_no_violations(&violations, "Clean async code should produce no violations");
}
