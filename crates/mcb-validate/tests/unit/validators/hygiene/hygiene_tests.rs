//! Tests for Test Hygiene Validation
//!
//! Validates `HygieneValidator` against fixture crates with precise
//! file + line + violation-type assertions.
//!
//! Note: `BadTestFileName` violations have no line field, so line=0
//! is used (skips line check).

use crate::utils::test_constants::*;
use crate::utils::*;
use mcb_domain::utils::tests::assertions::{assert_no_violations, assert_violations_exact};
use rstest::rstest;
use std::fs;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
fn test_hygiene_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let violations = run_named_validator(&root, "hygiene").unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── InlineTestModule ────────────────────────────────────────
            ("my-test/src/lib.rs", 366, "InlineTestModule"),
            // ── BadTestFileName — no line field, use 0 ──────────────────
            ("my-test/tests", 0, "BadTestFileName"),
            ("integration_test.rs", 0, "BadTestFileName"),
            // ── TrivialAssertion ────────────────────────────────────────
            ("integration_test.rs", 4, "TrivialAssertion"),
            ("integration_test.rs", 10, "TrivialAssertion"),
            ("integration_test.rs", 16, "TrivialAssertion"),
        ],
        "HygieneValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
fn test_clean_hygiene_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        "
/// A well-structured module.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
",
    );
    let violations = run_named_validator(&root, "hygiene").unwrap();

    assert_no_violations(
        &violations,
        "Clean test organization should produce no violations",
    );
}

#[rstest]
fn test_support_helpers_are_not_test_functions() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        "
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
",
    );

    let helper_dir = root
        .join("crates")
        .join(TEST_CRATE)
        .join("tests")
        .join("utils");
    fs::create_dir_all(&helper_dir).expect("create tests/utils fixture dir");
    fs::write(
        helper_dir.join("test_fixtures.rs"),
        r#"
fn create_shared_test_context() -> Result<&'static str, &'static str> {
    // individual `#[tokio::test]` runtimes must not make the next helper a test.
    Ok("ctx")
}

pub fn shared_app_context() -> Result<&'static str, &'static str> {
    Ok("ctx")
}

pub fn try_shared_mcb_state() -> Option<&'static str> {
    // existing #[tokio::test] runtime comments must stay ignored.
    Some("state")
}

pub fn shared_mcb_state() -> Result<&'static str, &'static str> {
    try_shared_mcb_state().ok_or("state init failed")
}

#[test]
fn helper_smoke_test() {
    assert_eq!(shared_app_context(), Ok("ctx"));
    assert_eq!(shared_mcb_state(), Ok("state"));
}
"#,
    )
    .expect("write tests/utils helper fixture");

    let violations = run_named_validator(&root, "hygiene").unwrap();
    assert_no_violations(
        &violations,
        "Test support helpers under tests/utils are not executable tests",
    );
}

#[rstest]
fn test_marker_inside_string_literal_is_not_inline_test() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r##"
pub fn marker_text() -> &'static str {
    "#[test]"
}
"##,
    );
    let violations = run_named_validator(&root, "hygiene").unwrap();

    assert_no_violations(
        &violations,
        "String literals that mention test attributes are not inline tests",
    );
}
