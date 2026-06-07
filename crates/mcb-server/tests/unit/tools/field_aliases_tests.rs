//! Field alias resolution: agents can use `camelCase`, `snake_case`, or `x-header` style.
//!
//! Tests verify that context fields are resolved from any naming convention
//! the agent might use (`session_id`, `sessionId`, `x-session-id` all work).

use std::collections::HashMap;

use mcb_server::tools::field_aliases::{
    field_aliases, insert_override, normalize_text, resolve_override_bool, resolve_override_value,
    str_value,
};
use rstest::rstest;

// ─── Alias registry ──────────────────────────────────────────────────

#[rstest]
#[case("session_id", &["session_id", "sessionId", "x-session-id", "x_session_id"])]
#[case("delegated", &["delegated", "is_delegated", "isDelegated", "x-delegated"])]
fn known_fields_have_all_naming_variants(#[case] field: &str, #[case] expected: &[&str]) {
    let aliases = field_aliases(field);
    for alias in expected {
        assert!(
            aliases.contains(alias),
            "missing alias '{alias}' for '{field}'"
        );
    }
}

#[rstest]
fn every_canonical_field_includes_itself() {
    let fields = [
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
    for f in fields {
        assert!(
            field_aliases(f).contains(&f),
            "'{f}' should alias to itself"
        );
    }
}

#[rstest]
fn unknown_field_returns_empty_aliases() {
    assert!(field_aliases("nonexistent").is_empty());
}

// ─── Text normalization ──────────────────────────────────────────────

#[rstest]
#[case(None, None)]
#[case(Some(String::new()), None)]
#[case(Some("   ".to_owned()), None)]
#[case(Some("  hello  ".to_owned()), Some("hello".to_owned()))]
fn normalize_trims_and_rejects_empty(
    #[case] input: Option<String>,
    #[case] expected: Option<String>,
) {
    assert_eq!(normalize_text(input), expected);
}

// ─── String override resolution ──────────────────────────────────────

#[rstest]
fn first_matching_alias_wins() {
    let overrides = HashMap::from([
        ("session_id".to_owned(), "s1".to_owned()),
        ("sessionId".to_owned(), "s2".to_owned()),
    ]);
    assert_eq!(
        resolve_override_value(&overrides, &["session_id", "sessionId"]),
        Some("s1".to_owned())
    );
}

#[rstest]
fn fallback_alias_used_when_primary_missing() {
    let overrides = HashMap::from([("sessionId".to_owned(), "s2".to_owned())]);
    assert_eq!(
        resolve_override_value(&overrides, &["session_id", "sessionId"]),
        Some("s2".to_owned())
    );
}

#[rstest]
fn whitespace_only_values_skipped_in_resolution() {
    let overrides = HashMap::from([
        ("session_id".to_owned(), "   ".to_owned()),
        ("sessionId".to_owned(), "s2".to_owned()),
    ]);
    assert_eq!(
        resolve_override_value(&overrides, &["session_id", "sessionId"]),
        Some("s2".to_owned())
    );
}

#[rstest]
fn no_matching_key_returns_none() {
    assert!(resolve_override_value(&HashMap::new(), &["session_id"]).is_none());
}

// ─── Boolean override resolution ─────────────────────────────────────

#[rstest]
#[case("true", Some(true))]
#[case("True", Some(true))]
#[case("1", Some(true))]
#[case("yes", Some(true))]
#[case("false", Some(false))]
#[case("FALSE", Some(false))]
#[case("0", Some(false))]
#[case("no", Some(false))]
#[case("maybe", None)]
fn boolean_parsing_accepts_common_variants(#[case] input: &str, #[case] expected: Option<bool>) {
    let overrides = HashMap::from([("delegated".to_owned(), input.to_owned())]);
    assert_eq!(resolve_override_bool(&overrides, &["delegated"]), expected);
}

// ─── Override insertion ──────────────────────────────────────────────

#[rstest]
#[case(Some("sess-1".to_owned()), true)]
#[case(Some("  sess-1  ".to_owned()), true)]
#[case(None, false)]
#[case(Some(String::new()), false)]
#[case(Some("   ".to_owned()), false)]
fn insert_override_filters_empty_and_trims(#[case] value: Option<String>, #[case] inserted: bool) {
    let mut map = HashMap::new();
    insert_override(&mut map, "session_id", value);
    assert_eq!(map.contains_key("session_id"), inserted);
    if inserted {
        assert_eq!(map["session_id"], "sess-1");
    }
}

// ─── JSON value helper ───────────────────────────────────────────────

#[rstest]
fn str_value_produces_json_string() {
    assert_eq!(str_value("hello"), serde_json::json!("hello"));
}
