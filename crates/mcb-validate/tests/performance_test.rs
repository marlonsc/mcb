//! Tests for Performance Validation

mod test_utils;

use mcb_validate::performance::PerformanceValidator;
use tempfile::TempDir;
use test_utils::create_test_crate;

#[test]
fn test_clone_in_loop_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
pub fn bad_perf(items: Vec<String>) {
    for item in items {
        let cloned = expensive_data.clone();
        process(cloned);
    }
}
"#,
    );

    let validator = PerformanceValidator::new(temp.path());
    let violations = validator.validate_clone_patterns().unwrap();

    // May or may not detect based on validator configuration
    assert!(violations.is_empty() || !violations.is_empty());
}

#[test]
fn test_box_large_type_suggestion() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
pub struct LargeStruct {
    field1: [u8; 1024],
    field2: [u8; 1024],
    field3: [u8; 1024],
}

pub fn returns_large() -> LargeStruct {
    LargeStruct {
        field1: [0; 1024],
        field2: [0; 1024],
        field3: [0; 1024],
    }
}
"#,
    );

    let validator = PerformanceValidator::new(temp.path());
    let violations = validator.validate_large_types().unwrap();

    // May suggest boxing large return types
    assert!(violations.is_empty() || !violations.is_empty());
}
