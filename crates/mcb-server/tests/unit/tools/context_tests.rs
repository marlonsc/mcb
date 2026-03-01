//! Unit tests for `ToolExecutionContext` resolution and injection.

use std::collections::HashMap;

use mcb_server::tools::{ExecutionFlow, RuntimeDefaults, ToolExecutionContext};
use rmcp::model::CallToolRequestParams;
use rstest::rstest;

#[rstest]
#[test]
fn test_resolve_uses_override_when_present() {
    let defaults = RuntimeDefaults {
        workspace_root: Some("default-root".to_owned()),
        repo_path: Some("default-repo".to_owned()),
        repo_id: Some("default-repo-id".to_owned()),
        operator_id: Some("default-operator".to_owned()),
        machine_id: Some("default-machine".to_owned()),
        session_id: Some("default-session".to_owned()),
        agent_program: Some("default-agent".to_owned()),
        model_id: Some("default-model".to_owned()),
        execution_flow: Some(ExecutionFlow::StdioOnly),
    };

    let mut overrides = HashMap::new();
    overrides.insert("session_id".to_owned(), "override-session".to_owned());

    let context = ToolExecutionContext::resolve(&defaults, &overrides);

    assert_eq!(context.session_id, Some("override-session".to_owned()));
}

#[rstest]
#[test]
fn test_resolve_falls_back_to_default_when_override_missing() {
    let defaults = RuntimeDefaults {
        workspace_root: Some("default-root".to_owned()),
        repo_path: Some("default-repo".to_owned()),
        repo_id: Some("default-repo-id".to_owned()),
        operator_id: Some("default-operator".to_owned()),
        machine_id: Some("default-machine".to_owned()),
        session_id: Some("default-session".to_owned()),
        agent_program: Some("default-agent".to_owned()),
        model_id: Some("default-model".to_owned()),
        execution_flow: Some(ExecutionFlow::StdioOnly),
    };

    let overrides = HashMap::new();

    let context = ToolExecutionContext::resolve(&defaults, &overrides);

    assert_eq!(context.session_id, Some("default-session".to_owned()));
}

#[rstest]
#[test]
fn test_resolve_prefers_override_over_default() {
    let defaults = RuntimeDefaults {
        workspace_root: Some("default-root".to_owned()),
        repo_path: Some("default-repo".to_owned()),
        repo_id: Some("default-repo-id".to_owned()),
        operator_id: Some("default-operator".to_owned()),
        machine_id: Some("default-machine".to_owned()),
        session_id: Some("default-session".to_owned()),
        agent_program: Some("default-agent".to_owned()),
        model_id: Some("default-model".to_owned()),
        execution_flow: Some(ExecutionFlow::StdioOnly),
    };

    let mut overrides = HashMap::new();
    overrides.insert("repo_id".to_owned(), "override-repo-id".to_owned());

    let context = ToolExecutionContext::resolve(&defaults, &overrides);

    assert_eq!(context.repo_id, Some("override-repo-id".to_owned()));
}

#[rstest]
#[test]
fn test_resolve_handles_multiple_overrides() {
    let defaults = RuntimeDefaults {
        workspace_root: Some("default-root".to_owned()),
        repo_path: Some("default-repo".to_owned()),
        repo_id: Some("default-repo-id".to_owned()),
        operator_id: Some("default-operator".to_owned()),
        machine_id: Some("default-machine".to_owned()),
        session_id: Some("default-session".to_owned()),
        agent_program: Some("default-agent".to_owned()),
        model_id: Some("default-model".to_owned()),
        execution_flow: Some(ExecutionFlow::StdioOnly),
    };

    let mut overrides = HashMap::new();
    overrides.insert("session_id".to_owned(), "override-session".to_owned());
    overrides.insert("repo_id".to_owned(), "override-repo-id".to_owned());
    overrides.insert("operator_id".to_owned(), "override-operator".to_owned());

    let context = ToolExecutionContext::resolve(&defaults, &overrides);

    assert_eq!(context.session_id, Some("override-session".to_owned()));
    assert_eq!(context.repo_id, Some("override-repo-id".to_owned()));
    assert_eq!(context.operator_id, Some("override-operator".to_owned()));
}

#[rstest]
#[test]
fn test_resolve_sets_delegated_true_when_parent_session_id_present() {
    let defaults = RuntimeDefaults {
        workspace_root: None,
        repo_path: None,
        repo_id: None,
        operator_id: None,
        machine_id: None,
        session_id: None,
        agent_program: None,
        model_id: None,
        execution_flow: None,
    };

    let mut overrides = HashMap::new();
    overrides.insert("parent_session_id".to_owned(), "parent-123".to_owned());

    let context = ToolExecutionContext::resolve(&defaults, &overrides);

    assert_eq!(context.parent_session_id, Some("parent-123".to_owned()));
    assert_eq!(context.delegated, Some(true));
}

#[rstest]
#[test]
fn test_resolve_respects_explicit_delegated_override() {
    let defaults = RuntimeDefaults {
        workspace_root: None,
        repo_path: None,
        repo_id: None,
        operator_id: None,
        machine_id: None,
        session_id: None,
        agent_program: None,
        model_id: None,
        execution_flow: None,
    };

    let mut overrides = HashMap::new();
    overrides.insert("delegated".to_owned(), "false".to_owned());

    let context = ToolExecutionContext::resolve(&defaults, &overrides);

    assert_eq!(context.delegated, Some(false));
}

#[rstest]
#[test]
fn test_apply_to_request_if_missing_injects_missing_values() {
    let context = ToolExecutionContext {
        session_id: Some("sess-123".to_owned()),
        parent_session_id: None,
        project_id: Some("proj-456".to_owned()),
        worktree_id: None,
        repo_id: Some("repo-789".to_owned()),
        repo_path: None,
        operator_id: None,
        machine_id: None,
        agent_program: None,
        model_id: None,
        delegated: None,
        timestamp: None,
        execution_flow: None,
    };

    let mut request = CallToolRequestParams {
        name: "test_tool".to_owned().into(),
        arguments: None,
        task: None,
        meta: None,
    };

    context.apply_to_request_if_missing(&mut request);

    let args = request.arguments.as_ref().unwrap();
    assert_eq!(args.get("session_id"), Some(&serde_json::json!("sess-123")));
    assert_eq!(args.get("project_id"), Some(&serde_json::json!("proj-456")));
    assert_eq!(args.get("repo_id"), Some(&serde_json::json!("repo-789")));
}

#[rstest]
#[test]
fn test_apply_to_request_if_missing_does_not_overwrite_existing_values() {
    let context = ToolExecutionContext {
        session_id: Some("context-session".to_owned()),
        parent_session_id: None,
        project_id: Some("context-project".to_owned()),
        worktree_id: None,
        repo_id: None,
        repo_path: None,
        operator_id: None,
        machine_id: None,
        agent_program: None,
        model_id: None,
        delegated: None,
        timestamp: None,
        execution_flow: None,
    };

    let mut request = CallToolRequestParams {
        name: "test_tool".to_owned().into(),
        arguments: Some(
            vec![
                (
                    "session_id".to_owned(),
                    serde_json::json!("request-session"),
                ),
                (
                    "project_id".to_owned(),
                    serde_json::json!("request-project"),
                ),
            ]
            .into_iter()
            .collect(),
        ),
        task: None,
        meta: None,
    };

    context.apply_to_request_if_missing(&mut request);

    let args = request.arguments.as_ref().unwrap();
    assert_eq!(
        args.get("session_id"),
        Some(&serde_json::json!("request-session"))
    );
    assert_eq!(
        args.get("project_id"),
        Some(&serde_json::json!("request-project"))
    );
}

#[rstest]
#[test]
fn test_apply_to_request_if_missing_injects_boolean_values() {
    let context = ToolExecutionContext {
        session_id: None,
        parent_session_id: None,
        project_id: None,
        worktree_id: None,
        repo_id: None,
        repo_path: None,
        operator_id: None,
        machine_id: None,
        agent_program: None,
        model_id: None,
        delegated: Some(true),
        timestamp: None,
        execution_flow: None,
    };

    let mut request = CallToolRequestParams {
        name: "test_tool".to_owned().into(),
        arguments: None,
        task: None,
        meta: None,
    };

    context.apply_to_request_if_missing(&mut request);

    let args = request.arguments.as_ref().unwrap();
    assert_eq!(args.get("delegated"), Some(&serde_json::json!(true)));
}

#[rstest]
#[test]
fn test_apply_to_request_if_missing_injects_timestamp() {
    let context = ToolExecutionContext {
        session_id: None,
        parent_session_id: None,
        project_id: None,
        worktree_id: None,
        repo_id: None,
        repo_path: None,
        operator_id: None,
        machine_id: None,
        agent_program: None,
        model_id: None,
        delegated: None,
        timestamp: Some(1234567890),
        execution_flow: None,
    };

    let mut request = CallToolRequestParams {
        name: "test_tool".to_owned().into(),
        arguments: None,
        task: None,
        meta: None,
    };

    context.apply_to_request_if_missing(&mut request);

    let args = request.arguments.as_ref().unwrap();
    assert_eq!(args.get("timestamp"), Some(&serde_json::json!(1234567890)));
}

#[rstest]
#[test]
fn test_apply_to_request_if_missing_does_not_inject_none_values() {
    let context = ToolExecutionContext {
        session_id: None,
        parent_session_id: None,
        project_id: None,
        worktree_id: None,
        repo_id: None,
        repo_path: None,
        operator_id: None,
        machine_id: None,
        agent_program: None,
        model_id: None,
        delegated: None,
        timestamp: None,
        execution_flow: None,
    };

    let mut request = CallToolRequestParams {
        name: "test_tool".to_owned().into(),
        arguments: None,
        task: None,
        meta: None,
    };

    context.apply_to_request_if_missing(&mut request);

    let args = request.arguments.as_ref();
    assert!(args.is_none() || args.unwrap().is_empty());
}

#[rstest]
#[test]
fn test_apply_to_request_if_missing_creates_arguments_map_if_needed() {
    let context = ToolExecutionContext {
        session_id: Some("sess-123".to_owned()),
        parent_session_id: None,
        project_id: None,
        worktree_id: None,
        repo_id: None,
        repo_path: None,
        operator_id: None,
        machine_id: None,
        agent_program: None,
        model_id: None,
        delegated: None,
        timestamp: None,
        execution_flow: None,
    };

    let mut request = CallToolRequestParams {
        name: "test_tool".to_owned().into(),
        arguments: None,
        task: None,
        meta: None,
    };

    assert!(request.arguments.is_none());

    context.apply_to_request_if_missing(&mut request);

    assert!(request.arguments.is_some());
    let args = request.arguments.as_ref().unwrap();
    assert_eq!(args.get("session_id"), Some(&serde_json::json!("sess-123")));
}
