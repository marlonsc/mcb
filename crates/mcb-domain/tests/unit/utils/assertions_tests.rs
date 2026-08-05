//! Tests for shared assertion helpers.

use mcb_domain::utils::tests::assertions::assert_violations_exact;

struct DemoViolation {
    file: &'static str,
    line: usize,
    kind: &'static str,
}

impl std::fmt::Debug for DemoViolation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DemoViolation")
            .field("file", &self.file)
            .field("line", &self.line)
            .field("kind", &self.kind)
            .finish()
    }
}

#[test]
fn exact_violation_assertion_rejects_duplicate_matching_entries() {
    let violations = [
        DemoViolation {
            file: "tests/foo.rs",
            line: 7,
            kind: "BadThing",
        },
        DemoViolation {
            file: "tests/foo.rs",
            line: 7,
            kind: "BadThing",
        },
    ];

    let result = std::panic::catch_unwind(|| {
        assert_violations_exact(&violations, &[("tests/foo.rs", 7, "BadThing")], "duplicate");
    });

    assert!(
        result.is_err(),
        "exact violation assertion must reject duplicate matching violations"
    );
}
