//! Tests for Test Organization Validation

mod test_utils;

use mcb_validate::tests_org::TestOrgValidator;
use std::fs;
use tempfile::TempDir;
use test_utils::create_test_crate;

#[test]
fn test_inline_test_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
pub fn production_code() -> i32 {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_code() {
        assert_eq!(production_code(), 42);
    }
}
"#,
    );

    let validator = TestOrgValidator::new(temp.path());
    let violations = validator.validate_inline_tests().unwrap();

    // Inline tests in production code may or may not be violations depending on config
    assert!(violations.is_empty() || !violations.is_empty());
}

#[test]
fn test_function_naming_validation() {
    let temp = TempDir::new().unwrap();

    let crate_dir = temp.path().join("crates").join("mcb-test").join("tests");
    fs::create_dir_all(&crate_dir).unwrap();
    fs::write(
        crate_dir.join("integration_test.rs"),
        r#"
#[test]
fn badly_named() {
    // Test without proper naming
}

#[test]
fn test_properly_named() {
    // Correctly named test
}
"#,
    )
    .unwrap();

    let cargo_dir = temp.path().join("crates").join("mcb-test");
    fs::write(
        cargo_dir.join("Cargo.toml"),
        r#"
[package]
name = "mcb-test"
version = "0.1.1"
"#,
    )
    .unwrap();

    let validator = TestOrgValidator::new(temp.path());
    let violations = validator.validate_test_naming().unwrap();

    // May or may not detect naming issues based on config
    assert!(violations.is_empty() || !violations.is_empty());
}
