//! Tests for AST-based unwrap detector.
//!
//! Uses `UNWRAP_METHOD` and `EXPECT_METHOD` constants from shared
//! test infrastructure.

use mcb_validate::ast::UnwrapDetector;
use rstest::rstest;
use rstest::*;

use crate::test_constants::{EXPECT_METHOD, UNWRAP_METHOD};

#[test]
fn test_detector_creation() {
    let detector = UnwrapDetector::new();
    assert!(
        detector.is_ok(),
        "Should create unwrap detector successfully"
    );
}

#[fixture]
fn detector() -> UnwrapDetector {
    UnwrapDetector::new().expect("Should create detector")
}

#[rstest]
#[case("fn main() { let x = Some(1).unwrap(); }", UNWRAP_METHOD)]
#[case("fn main() { let x = Some(1).expect(\"error\"); }", EXPECT_METHOD)]
fn detect_single_unwrap_or_expect(
    mut detector: UnwrapDetector,
    #[case] code: &str,
    #[case] expected_method: &str,
) {
    let detections = detector
        .detect_in_content(code, "test.rs")
        .expect("Should detect unwrap/expect");

    assert_eq!(detections.len(), 1);
    assert_eq!(detections[0].method, expected_method);
    assert!(!detections[0].in_test);
}

#[test]
fn test_detect_multiple() {
    let mut detector = UnwrapDetector::new().expect("Should create detector");
    let code =
        "fn main() {\n    let x = Some(1).unwrap();\n    let y = Some(2).expect(\"error\");\n}";

    let detections = detector
        .detect_in_content(code, "test.rs")
        .expect("Should detect multiple");

    assert_eq!(detections.len(), 2);
    assert_eq!(detections[0].method, UNWRAP_METHOD);
    assert_eq!(detections[1].method, EXPECT_METHOD);
}

#[test]
fn test_ignore_safe_alternatives() {
    let mut detector = UnwrapDetector::new().expect("Should create detector");
    let code = "fn main() {\n    let x = Some(1).unwrap_or(0);\n    let y = Some(2).unwrap_or_default();\n}";

    let detections = detector
        .detect_in_content(code, "test.rs")
        .expect("Should not detect safe alternatives");

    assert_eq!(detections.len(), 0, "Should not detect unwrap_or variants");
}

#[test]
fn test_detect_in_test_module() {
    let mut detector = UnwrapDetector::new().expect("Should create detector");
    let code =
        "#[cfg(test)]\nmod tests {\n    fn test() {\n        let x = Some(1).unwrap();\n    }\n}";

    let detections = detector
        .detect_in_content(code, "test.rs")
        .expect("Should detect in test module");

    assert_eq!(detections.len(), 1);
    assert!(detections[0].in_test, "Should be marked as in test");
}

#[test]
fn test_line_numbers_are_correct() {
    let mut detector = UnwrapDetector::new().expect("Should create detector");
    let code = "fn main() {\n    let x = Some(1).unwrap();\n}\n";

    let detections = detector
        .detect_in_content(code, "test.rs")
        .expect("Should detect");

    assert_eq!(detections.len(), 1);
    assert_eq!(detections[0].line, 2, "Should be on line 2");
}
