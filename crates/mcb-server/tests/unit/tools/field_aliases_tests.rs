//! Unit tests for field alias resolution and normalization.

use std::collections::HashMap;

use mcb_server::tools::field_aliases::{
    field_aliases, insert_override, normalize_text, resolve_override_bool, resolve_override_value,
    str_value,
};
use rstest::rstest;

#[rstest]
#[test]
fn test_field_aliases_returns_correct_aliases_for_session_id() {
    let aliases = field_aliases("session_id");
    assert!(!aliases.is_empty());
    assert!(aliases.contains(&"session_id"));
    assert!(aliases.contains(&"sessionId"));
    assert!(aliases.contains(&"x-session-id"));
    assert!(aliases.contains(&"x_session_id"));
}

#[rstest]
#[test]
fn test_field_aliases_returns_correct_aliases_for_delegated() {
    let aliases = field_aliases("delegated");
    assert!(!aliases.is_empty());
    assert!(aliases.contains(&"delegated"));
    assert!(aliases.contains(&"is_delegated"));
    assert!(aliases.contains(&"isDelegated"));
    assert!(aliases.contains(&"x-delegated"));
}

#[rstest]
#[test]
fn test_field_aliases_returns_empty_for_unknown_field() {
    let aliases = field_aliases("unknown_field");
    assert!(aliases.is_empty());
}

#[rstest]
#[test]
fn test_field_aliases_returns_aliases_for_all_canonical_fields() {
    let canonical_fields = vec![
        "session_id",
        "parent_session_id",
        "project_id",
        "worktree_id",
        "repo_id",
        "repo_path",
        "workspace_root",
        "operator_id",
        "machine_id",
        "agent_program",
        "model_id",
        "execution_flow",
        "delegated",
    ];

    for field in canonical_fields {
        let aliases = field_aliases(field);
        assert!(!aliases.is_empty(), "Field {field} should have aliases");
        assert!(
            aliases.contains(&field),
            "Field {field} should include itself as an alias",
        );
    }
}

#[rstest]
#[test]
fn test_normalize_text_returns_none_for_empty_string() {
    let result = normalize_text(Some(String::new()));
    assert!(result.is_none());
}

#[rstest]
#[test]
fn test_normalize_text_returns_none_for_whitespace_only() {
    let result = normalize_text(Some("   ".to_owned()));
    assert!(result.is_none());
}

#[rstest]
#[test]
fn test_normalize_text_returns_none_for_none() {
    let result = normalize_text(None);
    assert!(result.is_none());
}

#[rstest]
#[test]
fn test_normalize_text_trims_and_returns_value() {
    let result = normalize_text(Some("  hello world  ".to_owned()));
    assert_eq!(result, Some("hello world".to_owned()));
}

#[rstest]
#[test]
fn test_resolve_override_value_returns_none_when_no_keys_match() {
    let overrides = HashMap::new();
    let keys = vec!["session_id", "sessionId"];
    let result = resolve_override_value(&overrides, &keys);
    assert!(result.is_none());
}

#[rstest]
#[test]
fn test_resolve_override_value_returns_value_for_first_matching_key() {
    let mut overrides = HashMap::new();
    overrides.insert("session_id".to_owned(), "sess-123".to_owned());
    overrides.insert("sessionId".to_owned(), "sess-456".to_owned());

    let keys = vec!["session_id", "sessionId"];
    let result = resolve_override_value(&overrides, &keys);
    assert_eq!(result, Some("sess-123".to_owned()));
}

#[rstest]
#[test]
fn test_resolve_override_value_checks_all_aliases_in_order() {
    let mut overrides = HashMap::new();
    overrides.insert("sessionId".to_owned(), "sess-456".to_owned());

    let keys = vec!["session_id", "sessionId"];
    let result = resolve_override_value(&overrides, &keys);
    assert_eq!(result, Some("sess-456".to_owned()));
}

#[rstest]
#[test]
fn test_resolve_override_value_ignores_empty_values() {
    let mut overrides = HashMap::new();
    overrides.insert("session_id".to_owned(), "   ".to_owned());
    overrides.insert("sessionId".to_owned(), "sess-456".to_owned());

    let keys = vec!["session_id", "sessionId"];
    let result = resolve_override_value(&overrides, &keys);
    assert_eq!(result, Some("sess-456".to_owned()));
}

#[rstest]
#[test]
fn test_resolve_override_bool_returns_none_when_no_keys_match() {
    let overrides = HashMap::new();
    let keys = vec!["delegated", "is_delegated"];
    let result = resolve_override_bool(&overrides, &keys);
    assert!(result.is_none());
}

#[rstest]
#[test]
fn test_resolve_override_bool_parses_true_variants() {
    let test_cases = vec!["true", "True", "TRUE", "1", "yes", "YES"];

    for value in test_cases {
        let mut overrides = HashMap::new();
        overrides.insert("delegated".to_owned(), value.to_owned());

        let keys = vec!["delegated"];
        let result = resolve_override_bool(&overrides, &keys);
        assert_eq!(result, Some(true), "Failed for value: {value}");
    }
}

#[rstest]
#[test]
fn test_resolve_override_bool_parses_false_variants() {
    let test_cases = vec!["false", "False", "FALSE", "0", "no", "NO"];

    for value in test_cases {
        let mut overrides = HashMap::new();
        overrides.insert("delegated".to_owned(), value.to_owned());

        let keys = vec!["delegated"];
        let result = resolve_override_bool(&overrides, &keys);
        assert_eq!(result, Some(false), "Failed for value: {value}");
    }
}

#[rstest]
#[test]
fn test_resolve_override_bool_returns_none_for_invalid_value() {
    let mut overrides = HashMap::new();
    overrides.insert("delegated".to_owned(), "maybe".to_owned());

    let keys = vec!["delegated"];
    let result = resolve_override_bool(&overrides, &keys);
    assert!(result.is_none());
}

#[rstest]
#[test]
fn test_resolve_override_bool_checks_all_aliases_in_order() {
    let mut overrides = HashMap::new();
    overrides.insert("is_delegated".to_owned(), "true".to_owned());

    let keys = vec!["delegated", "is_delegated"];
    let result = resolve_override_bool(&overrides, &keys);
    assert_eq!(result, Some(true));
}

#[rstest]
#[test]
fn test_str_value_creates_json_string() {
    let result = str_value("hello");
    assert_eq!(result, serde_json::json!("hello"));
}

#[rstest]
#[test]
fn test_str_value_handles_special_characters() {
    let result = str_value("hello-world_123");
    assert_eq!(result, serde_json::json!("hello-world_123"));
}

#[rstest]
#[test]
fn test_insert_override_adds_value_when_present() {
    let mut overrides = HashMap::new();
    insert_override(&mut overrides, "session_id", Some("sess-123".to_owned()));

    assert_eq!(overrides.get("session_id"), Some(&"sess-123".to_owned()));
}

#[rstest]
#[test]
fn test_insert_override_does_not_add_when_none() {
    let mut overrides = HashMap::new();
    insert_override(&mut overrides, "session_id", None);

    assert!(!overrides.contains_key("session_id"));
}

#[rstest]
#[test]
fn test_insert_override_does_not_add_empty_string() {
    let mut overrides = HashMap::new();
    insert_override(&mut overrides, "session_id", Some(String::new()));

    assert!(!overrides.contains_key("session_id"));
}

#[rstest]
#[test]
fn test_insert_override_does_not_add_whitespace_only() {
    let mut overrides = HashMap::new();
    insert_override(&mut overrides, "session_id", Some("   ".to_owned()));

    assert!(!overrides.contains_key("session_id"));
}

#[rstest]
#[test]
fn test_insert_override_trims_value_before_inserting() {
    let mut overrides = HashMap::new();
    insert_override(
        &mut overrides,
        "session_id",
        Some("  sess-123  ".to_owned()),
    );

    assert_eq!(overrides.get("session_id"), Some(&"sess-123".to_owned()));
}
