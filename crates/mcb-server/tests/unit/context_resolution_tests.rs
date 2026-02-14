use mcb_server::handlers::helpers::{normalize_identifier, resolve_identifier_precedence};
use rstest::*;

#[rstest]
#[case(None, None)]
#[case(Some(""), None)]
#[case(Some("   "), None)]
#[case(Some("  abc  "), Some("abc"))]
fn normalize_identifier_treats_blank_as_missing(
    #[case] input: Option<&str>,
    #[case] expected: Option<&str>,
) {
    assert_eq!(
        normalize_identifier(input),
        expected.map(std::string::ToString::to_string)
    );
}

#[rstest]
#[case(Some("proj-1"), Some("proj-1"), Some("proj-1"))]
#[case(Some("proj-1"), Some("   "), Some("proj-1"))]
#[case(Some("   "), Some("proj-2"), Some("proj-2"))]
fn resolve_identifier_precedence_uses_non_conflicting_values(
    #[case] args_value: Option<&str>,
    #[case] payload_value: Option<&str>,
    #[case] expected: Option<&str>,
) {
    let resolved = resolve_identifier_precedence("project_id", args_value, payload_value)
        .expect("should resolve");
    assert_eq!(resolved, expected.map(std::string::ToString::to_string));
}

#[test]
fn resolve_identifier_precedence_rejects_conflicting_values() {
    let err = resolve_identifier_precedence("project_id", Some("proj-a"), Some("proj-b"))
        .expect_err("should reject conflicting identifiers");
    assert!(err.message.contains("conflicting project_id"));
}
