//! Schema regression tests for MCP tool argument types.

use mcb_server::args::{
    EntityAction, EntityArgs, EntityResource, IndexAction, IndexArgs, MemoryAction, MemoryArgs,
    MemoryResource, SearchArgs, SearchResource, SessionAction, SessionArgs, ValidateAction,
    ValidateArgs, VcsAction, VcsArgs,
};
use rstest::rstest;
use schemars::{JsonSchema, schema_for};
use serde_json::Value;

fn schema_json<T: JsonSchema>() -> Value {
    let serialized = serde_json::to_value(schema_for!(T));
    assert!(serialized.is_ok(), "serialize schema");
    serialized.unwrap_or_else(|_| Value::Null)
}

fn required_names(schema: &Value) -> Vec<String> {
    schema
        .get("required")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn assert_property_exists(schema: &Value, field: &str) {
    assert!(
        schema
            .get("properties")
            .and_then(Value::as_object)
            .is_some_and(|props| props.contains_key(field)),
        "expected property '{field}' in schema: {schema}"
    );
}

fn enum_values(schema: &Value) -> Vec<String> {
    fn find_enum(value: &Value) -> Option<Vec<String>> {
        if let Some(values) = value.get("enum").and_then(Value::as_array) {
            return Some(
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect(),
            );
        }

        if let Some(object) = value.as_object() {
            for nested in object.values() {
                if let Some(values) = find_enum(nested) {
                    return Some(values);
                }
            }
        }

        if let Some(items) = value.as_array() {
            for item in items {
                if let Some(values) = find_enum(item) {
                    return Some(values);
                }
            }
        }

        None
    }

    fn collect_consts(value: &Value, out: &mut Vec<String>) {
        if let Some(text) = value.get("const").and_then(Value::as_str) {
            out.push(text.to_owned());
        }

        if let Some(object) = value.as_object() {
            for nested in object.values() {
                collect_consts(nested, out);
            }
        }

        if let Some(items) = value.as_array() {
            for item in items {
                collect_consts(item, out);
            }
        }
    }

    if let Some(values) = find_enum(schema) {
        return values;
    }

    let mut const_values = Vec::new();
    collect_consts(schema, &mut const_values);
    if !const_values.is_empty() {
        return const_values;
    }

    assert!(!const_values.is_empty(), "enum array");
    const_values
}

#[rstest]
#[case("memory", required_names(&schema_json::<MemoryArgs>()), vec!["action", "resource"])]
#[case("session", required_names(&schema_json::<SessionArgs>()), vec!["action"])]
#[case("index", required_names(&schema_json::<IndexArgs>()), vec!["action"])]
#[case("vcs", required_names(&schema_json::<VcsArgs>()), vec!["action"])]
#[case("search", required_names(&schema_json::<SearchArgs>()), vec!["query", "resource"])]
#[case("entity", required_names(&schema_json::<EntityArgs>()), vec!["action", "resource"])]
#[case("validate", required_names(&schema_json::<ValidateArgs>()), vec!["action"])]
fn schema_required_fields_include_contract(
    #[case] schema_name: &str,
    #[case] required: Vec<String>,
    #[case] expected: Vec<&str>,
) {
    for field in expected {
        assert!(
            required.iter().any(|name| name == field),
            "schema '{schema_name}' missing required field '{field}', got: {required:?}"
        );
    }
}

#[rstest]
#[case("memory", schema_json::<MemoryArgs>(), vec!["action", "resource", "data", "session_id"])]
#[case("session", schema_json::<SessionArgs>(), vec!["action", "session_id", "data"])]
#[case("index", schema_json::<IndexArgs>(), vec!["action", "path", "collection"])]
#[case("vcs", schema_json::<VcsArgs>(), vec!["action", "repo_id", "repo_path", "query"])]
#[case("search", schema_json::<SearchArgs>(), vec!["resource", "session_id", "query"])]
#[case("entity", schema_json::<EntityArgs>(), vec!["action", "resource", "data", "org_id"])]
#[case("validate", schema_json::<ValidateArgs>(), vec!["action", "scope", "path"])]
fn schema_has_expected_property_names(
    #[case] schema_name: &str,
    #[case] schema: Value,
    #[case] expected_fields: Vec<&str>,
) {
    assert!(!schema_name.is_empty());
    for field in expected_fields {
        assert_property_exists(&schema, field);
    }
}

#[rstest]
#[case("memory_action", enum_values(&schema_json::<MemoryAction>()), vec!["store", "get", "list", "timeline", "inject"])]
#[case("memory_resource", enum_values(&schema_json::<MemoryResource>()), vec!["observation", "execution", "quality_gate", "error_pattern", "session"])]
#[case("session_action", enum_values(&schema_json::<SessionAction>()), vec!["create", "get", "update", "list", "summarize"])]
#[case("index_action", enum_values(&schema_json::<IndexAction>()), vec!["start", "git_index", "status", "clear"])]
#[case("vcs_action", enum_values(&schema_json::<VcsAction>()), vec!["list_repositories", "index_repository", "compare_branches", "search_branch", "analyze_impact"])]
#[case("search_resource", enum_values(&schema_json::<SearchResource>()), vec!["code", "memory", "context"])]
#[case("entity_action", enum_values(&schema_json::<EntityAction>()), vec!["create", "get", "update", "list", "delete", "release"])]
#[case("entity_resource", enum_values(&schema_json::<EntityResource>()), vec!["repository", "branch", "worktree", "assignment", "plan", "version", "review", "issue", "comment", "label", "label_assignment", "org", "user", "team", "team_member", "api_key"])]
#[case("validate_action", enum_values(&schema_json::<ValidateAction>()), vec!["run", "list_rules", "analyze"])]
fn enum_schema_values_match_snake_case(
    #[case] enum_name: &str,
    #[case] actual: Vec<String>,
    #[case] expected: Vec<&str>,
) {
    let expected: Vec<String> = expected.into_iter().map(str::to_string).collect();
    assert_eq!(actual, expected, "enum '{enum_name}' values changed");
}
