use mcb_server::handlers::helpers::{normalize_identifier, resolve_identifier_precedence};
use rstest::rstest;

#[rstest]
#[case(None, None)]
#[case(Some(""), None)]
#[case(Some("   "), None)]
#[case(Some("  abc  "), Some("abc"))]
#[test]
fn normalize_identifier_treats_blank_as_missing(
    #[case] input: Option<&str>,
    #[case] expected: Option<&str>,
) {
    assert_eq!(
        normalize_identifier(input),
        expected.map(std::string::ToString::to_string)
    );
}

#[test]
fn resolve_identifier_precedence_prefers_args_when_equal() {
    let resolved = resolve_identifier_precedence("project_id", Some("proj-1"), Some("proj-1"))
        .expect("should resolve");
    assert_eq!(resolved, Some("proj-1".to_string()));
}

#[test]
fn resolve_identifier_precedence_prefers_args_when_payload_missing() {
    let resolved = resolve_identifier_precedence("project_id", Some("proj-1"), Some("   "))
        .expect("should resolve");
    assert_eq!(resolved, Some("proj-1".to_string()));
}

#[test]
fn resolve_identifier_precedence_uses_payload_when_args_missing() {
    let resolved = resolve_identifier_precedence("project_id", Some("   "), Some("proj-2"))
        .expect("should resolve");
    assert_eq!(resolved, Some("proj-2".to_string()));
}

#[test]
fn resolve_identifier_precedence_rejects_conflicting_values() {
    let err = resolve_identifier_precedence("project_id", Some("proj-a"), Some("proj-b"))
        .expect_err("should reject conflicting identifiers");
    assert!(err.message.contains("conflicting project_id"));
}
