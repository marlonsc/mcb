//! Context auto-injection: agents never pass context manually.
//!
//! These tests verify the zero-config agent experience: the server resolves
//! session, repo, operator, and other context fields from boot-time defaults
//! and per-request overrides, then injects them into tool requests transparently.

use std::collections::HashMap;

use mcb_server::tools::{ExecutionFlow, RuntimeDefaults, ToolExecutionContext};
use rmcp::model::CallToolRequestParams;
use rstest::{fixture, rstest};

/// Boot-time defaults simulating a detected workspace.
#[fixture]
fn defaults() -> RuntimeDefaults {
    RuntimeDefaults {
        workspace_root: Some("/repo".to_owned()),
        repo_path: Some("/repo".to_owned()),
        repo_id: Some("repo-abc".to_owned()),
        operator_id: Some("dev-user".to_owned()),
        machine_id: Some("laptop-01".to_owned()),
        session_id: Some("sess-boot".to_owned()),
        agent_program: Some("claude-code".to_owned()),
        model_id: Some("opus".to_owned()),
        execution_flow: Some(ExecutionFlow::StdioOnly),
        client_session_id: None,
        org_id: None,
        project_id: None,
    }
}

fn empty_defaults() -> RuntimeDefaults {
    RuntimeDefaults {
        workspace_root: None,
        repo_path: None,
        repo_id: None,
        operator_id: None,
        machine_id: None,
        session_id: None,
        agent_program: None,
        model_id: None,
        execution_flow: None,
        client_session_id: None,
        org_id: None,
        project_id: None,
    }
}

fn empty_request() -> CallToolRequestParams {
    CallToolRequestParams::new("search_code")
}

// ─── Resolution: boot defaults + per-request overrides ───────────────

#[rstest]
fn agent_override_wins_over_boot_default(defaults: RuntimeDefaults) {
    let overrides = HashMap::from([("session_id".to_owned(), "agent-session".to_owned())]);

    let ctx = ToolExecutionContext::resolve(&defaults, &overrides);

    assert_eq!(ctx.session_id.as_deref(), Some("agent-session"));
}

#[rstest]
fn boot_default_used_when_agent_sends_nothing(defaults: RuntimeDefaults) {
    let ctx = ToolExecutionContext::resolve(&defaults, &HashMap::new());

    assert_eq!(ctx.session_id.as_deref(), Some("sess-boot"));
    assert_eq!(ctx.repo_id.as_deref(), Some("repo-abc"));
    assert_eq!(ctx.operator_id.as_deref(), Some("dev-user"));
}

#[rstest]
fn multiple_overrides_applied_independently(defaults: RuntimeDefaults) {
    let overrides = HashMap::from([
        ("session_id".to_owned(), "s1".to_owned()),
        ("repo_id".to_owned(), "r1".to_owned()),
        ("operator_id".to_owned(), "o1".to_owned()),
    ]);

    let ctx = ToolExecutionContext::resolve(&defaults, &overrides);

    assert_eq!(ctx.session_id.as_deref(), Some("s1"));
    assert_eq!(ctx.repo_id.as_deref(), Some("r1"));
    assert_eq!(ctx.operator_id.as_deref(), Some("o1"));
}

// ─── Delegation inference ────────────────────────────────────────────

#[rstest]
fn parent_session_implies_delegation() {
    let overrides = HashMap::from([("parent_session_id".to_owned(), "parent-123".to_owned())]);

    let ctx = ToolExecutionContext::resolve(&empty_defaults(), &overrides);

    assert_eq!(ctx.delegated, Some(true));
    assert_eq!(ctx.parent_session_id.as_deref(), Some("parent-123"));
}

#[rstest]
fn explicit_delegated_false_overrides_inference() {
    let overrides = HashMap::from([("delegated".to_owned(), "false".to_owned())]);

    let ctx = ToolExecutionContext::resolve(&empty_defaults(), &overrides);

    assert_eq!(ctx.delegated, Some(false));
}

// ─── Transparent injection into MCP requests ─────────────────────────

#[rstest]
fn missing_context_fields_injected_into_request() {
    let ctx = ToolExecutionContext {
        session_id: Some("s1".to_owned()),
        project_id: Some("p1".to_owned()),
        repo_id: Some("r1".to_owned()),
        ..ToolExecutionContext::resolve(&empty_defaults(), &HashMap::new())
    };

    let mut req = empty_request();
    ctx.apply_to_request_if_missing(&mut req);

    let args = req.arguments.as_ref().expect("args created");
    assert_eq!(args.get("session_id"), Some(&serde_json::json!("s1")));
    assert_eq!(args.get("project_id"), Some(&serde_json::json!("p1")));
    assert_eq!(args.get("repo_id"), Some(&serde_json::json!("r1")));
}

#[rstest]
fn agent_explicit_values_never_overwritten() {
    let ctx = ToolExecutionContext {
        session_id: Some("server-session".to_owned()),
        project_id: Some("server-project".to_owned()),
        ..ToolExecutionContext::resolve(&empty_defaults(), &HashMap::new())
    };

    let mut req = CallToolRequestParams::new("search_code").with_arguments(
        [
            ("session_id", "agent-session"),
            ("project_id", "agent-project"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), serde_json::json!(v)))
        .collect(),
    );

    ctx.apply_to_request_if_missing(&mut req);

    let args = req.arguments.as_ref().unwrap();
    assert_eq!(
        args.get("session_id"),
        Some(&serde_json::json!("agent-session"))
    );
    assert_eq!(
        args.get("project_id"),
        Some(&serde_json::json!("agent-project"))
    );
}

#[rstest]
fn boolean_and_numeric_fields_injected_correctly() {
    let ctx = ToolExecutionContext {
        delegated: Some(true),
        timestamp: Some(1_700_000_000),
        ..ToolExecutionContext::resolve(&empty_defaults(), &HashMap::new())
    };

    let mut req = empty_request();
    ctx.apply_to_request_if_missing(&mut req);

    let args = req.arguments.as_ref().expect("args created");
    assert_eq!(args.get("delegated"), Some(&serde_json::json!(true)));
    assert_eq!(
        args.get("timestamp"),
        Some(&serde_json::json!(1_700_000_000))
    );
}

#[rstest]
fn empty_context_only_injects_auto_derived_fields() {
    let ctx = ToolExecutionContext::resolve(&empty_defaults(), &HashMap::new());

    let mut req = empty_request();
    ctx.apply_to_request_if_missing(&mut req);

    // With empty defaults, only auto-derived fields (timestamp, execution_flow)
    // should be injected — no agent-visible context like session/repo/operator
    if let Some(args) = req.arguments.as_ref() {
        let agent_fields = [
            "session_id",
            "repo_id",
            "repo_path",
            "operator_id",
            "machine_id",
            "agent_program",
            "model_id",
            "project_id",
        ];
        for field in &agent_fields {
            assert!(
                !args.contains_key(*field),
                "'{field}' should not be injected from empty context"
            );
        }
    }
}
