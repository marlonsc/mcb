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
            // ── BlockingInAsync ────────────────────────────────────────
            // std::fs::read in server handler
            (SERVER_CRATE_HANDLER, 106, "BlockingInAsync"),
            // std::fs::read in async fn
            (TEST_CRATE_LIB, 102, "BlockingInAsync"),
            // thread::sleep in async fn (two patterns match same line)
            (TEST_CRATE_LIB, 105, "BlockingInAsync"),
            (TEST_CRATE_LIB, 105, "BlockingInAsync"),
            // ── BlockOnInAsync ─────────────────────────────────────────
            // async fn def triggers block_on detection
            (TEST_CRATE_LIB, 293, "BlockOnInAsync"),
            // actual block_on call
            (TEST_CRATE_LIB, 295, "BlockOnInAsync"),
            // ── WrongMutexType ─────────────────────────────────────────
            // std::sync::Mutex in OveruseExample struct (also triggers PERF003)
            (TEST_CRATE_LIB, 169, "WrongMutexType"),
        ],
        "AsyncPatternValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

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
    let validator = AsyncPatternValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(&violations, "Clean async code should produce no violations");
}
