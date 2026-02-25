//! Golden tests: Session lifecycle operations.
//!
//! Verifies create, get, list, end (update), error handling, and summarize
//! through the `SessionHandler` MCP tool interface.

use crate::utils::test_fixtures::{create_test_mcp_server, golden_content_to_string};
use mcb_server::args::{SessionAction, SessionArgs};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

/// Helper: build a `SessionArgs` with defaults for all optional fields.
fn base_args(action: SessionAction) -> SessionArgs {
    SessionArgs {
        action,
        org_id: None,
        session_id: None,
        data: None,
        project_id: None,
        worktree_id: None,
        parent_session_id: None,
        agent_type: None,
        status: None,
        limit: None,
    }
}

/// Parse handler response text as JSON value.
fn result_json(res: &rmcp::model::CallToolResult) -> serde_json::Value {
    let text = golden_content_to_string(res);
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("response should be valid JSON: {text}; error: {e}"))
}

/// Create a session and return the parsed JSON response.
async fn create_session(server: &mcb_server::mcp_server::McpServer) -> serde_json::Value {
    let mut args = base_args(SessionAction::Create);
    args.agent_type = Some("sisyphus".to_owned());
    args.project_id = Some("test-project-session".to_owned());
    args.data = Some(json!({
        "model": "claude-3-opus",
        "agent_type": "sisyphus",
        "project_id": "test-project-session",
        "prompt_summary": "Golden test session"
    }));

    let result = server.session_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "session create should succeed: {result:?}");
    let res = result.expect("session create result");
    assert!(
        !res.is_error.unwrap_or(true),
        "session create response should not be error"
    );
    result_json(&res)
}

// ---------------------------------------------------------------------------
// Test 1: Create session → Get by id → verify fields
// ---------------------------------------------------------------------------
#[tokio::test]
async fn golden_session_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;

    let created = create_session(&server).await;
    let session_id = created["session_id"]
        .as_str()
        .expect("created response must contain session_id");
    assert_eq!(created["status"].as_str(), Some("active"));
    assert_eq!(created["agent_type"].as_str(), Some("sisyphus"));

    // Get the session by id
    let mut get_args = base_args(SessionAction::Get);
    get_args.session_id = Some(mcb_domain::value_objects::ids::SessionId::from_string(
        session_id,
    ));

    let get_result = server.session_handler().handle(Parameters(get_args)).await;
    assert!(
        get_result.is_ok(),
        "session get should succeed: {get_result:?}"
    );
    let get_res = get_result.expect("session get result");
    assert!(
        !get_res.is_error.unwrap_or(true),
        "session get response should not be error"
    );

    let fetched = result_json(&get_res);
    assert_eq!(fetched["id"].as_str(), Some(session_id));
    assert_eq!(fetched["agent_type"].as_str(), Some("sisyphus"));
    assert_eq!(fetched["model"].as_str(), Some("claude-3-opus"));
    assert_eq!(fetched["status"].as_str(), Some("active"));
    assert!(
        fetched["started_at"].as_i64().is_some(),
        "started_at must be set"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Create sessions → List → verify count
// ---------------------------------------------------------------------------
#[tokio::test]
async fn golden_session_list() {
    let (server, _td) = create_test_mcp_server().await;

    // Create two sessions
    let _ = create_session(&server).await;
    let _ = create_session(&server).await;

    let mut list_args = base_args(SessionAction::List);
    list_args.limit = Some(10);

    let list_result = server.session_handler().handle(Parameters(list_args)).await;
    assert!(
        list_result.is_ok(),
        "session list should succeed: {list_result:?}"
    );
    let list_res = list_result.expect("session list result");
    assert!(
        !list_res.is_error.unwrap_or(true),
        "session list response should not be error"
    );

    let body = result_json(&list_res);
    let count = body["count"].as_u64().unwrap_or(0);
    assert!(
        count >= 2,
        "list should return at least 2 sessions, got {count}"
    );

    let sessions = body["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert!(
        sessions.len() >= 2,
        "sessions array should have at least 2 entries, got {}",
        sessions.len()
    );
}

// ---------------------------------------------------------------------------
// Test 3: Create → End session (update status) → Get → verify ended_at / status
// ---------------------------------------------------------------------------
#[tokio::test]
async fn golden_session_end() {
    let (server, _td) = create_test_mcp_server().await;

    let created = create_session(&server).await;
    let session_id = created["session_id"]
        .as_str()
        .expect("created response must contain session_id");

    // Update session to completed status
    let mut update_args = base_args(SessionAction::Update);
    update_args.session_id = Some(mcb_domain::value_objects::ids::SessionId::from_string(
        session_id,
    ));
    update_args.status = Some("completed".to_owned());
    update_args.data = Some(json!({
        "result_summary": "Session completed successfully"
    }));

    let update_result = server
        .session_handler()
        .handle(Parameters(update_args))
        .await;
    assert!(
        update_result.is_ok(),
        "session update should succeed: {update_result:?}"
    );
    let update_res = update_result.expect("session update result");
    assert!(
        !update_res.is_error.unwrap_or(true),
        "session update response should not be error"
    );

    let updated = result_json(&update_res);
    assert_eq!(updated["status"].as_str(), Some("completed"));

    // Get the session and verify status persisted
    let mut get_args = base_args(SessionAction::Get);
    get_args.session_id = Some(mcb_domain::value_objects::ids::SessionId::from_string(
        session_id,
    ));

    let get_result = server.session_handler().handle(Parameters(get_args)).await;
    assert!(
        get_result.is_ok(),
        "session get after end should succeed: {get_result:?}"
    );
    let get_res = get_result.expect("session get result");

    let fetched = result_json(&get_res);
    assert_eq!(fetched["status"].as_str(), Some("completed"));
}

// ---------------------------------------------------------------------------
// Test 4: Create with no data → verify error
// ---------------------------------------------------------------------------
#[tokio::test]
async fn golden_session_create_missing_data() {
    let (server, _td) = create_test_mcp_server().await;

    // Create with no data payload at all
    let args = base_args(SessionAction::Create);

    let result = server.session_handler().handle(Parameters(args)).await;
    // The handler may return Ok with is_error=true, or Err — both are acceptable
    match result {
        Ok(res) => {
            assert!(
                res.is_error.unwrap_or(false),
                "create with no data should return an error response"
            );
        }
        Err(_) => {
            // Validation error at the MCP layer is also acceptable
        }
    }
}

// ---------------------------------------------------------------------------
// Test 5: Get with fake id → verify error
// ---------------------------------------------------------------------------
#[tokio::test]
async fn golden_session_get_nonexistent() {
    let (server, _td) = create_test_mcp_server().await;

    let mut args = base_args(SessionAction::Get);
    args.session_id = Some(mcb_domain::value_objects::ids::SessionId::from_string(
        "00000000-0000-0000-0000-000000000000",
    ));

    let result = server.session_handler().handle(Parameters(args)).await;
    assert!(
        result.is_ok(),
        "get nonexistent should not panic: {result:?}"
    );
    let res = result.expect("get nonexistent result");

    // Should indicate not found (is_error=true or error text)
    let text = golden_content_to_string(&res);
    let is_error = res.is_error.unwrap_or(false);
    let mentions_not_found = text.to_lowercase().contains("not found");
    assert!(
        is_error || mentions_not_found,
        "get nonexistent should return error or 'not found', got: {text}"
    );
}

// ---------------------------------------------------------------------------
// Test 6: Create → Summarize → verify response
// ---------------------------------------------------------------------------
#[tokio::test]
async fn golden_session_summary() {
    let (server, _td) = create_test_mcp_server().await;

    let created = create_session(&server).await;
    let session_id = created["session_id"]
        .as_str()
        .expect("created response must contain session_id");

    let mut summarize_args = base_args(SessionAction::Summarize);
    summarize_args.session_id = Some(mcb_domain::value_objects::ids::SessionId::from_string(
        session_id,
    ));
    summarize_args.project_id = Some("test-project-session".to_owned());
    summarize_args.data = Some(json!({
        "topics": ["architecture", "testing"],
        "decisions": ["Use golden tests for session lifecycle"],
        "next_steps": ["Add more coverage"],
        "key_files": ["tests/golden/test_session_lifecycle.rs"],
        "project_id": "test-project-session"
    }));

    let summarize_result = server
        .session_handler()
        .handle(Parameters(summarize_args))
        .await;
    assert!(
        summarize_result.is_ok(),
        "session summarize should succeed: {summarize_result:?}"
    );
    let summarize_res = summarize_result.expect("session summarize result");
    assert!(
        !summarize_res.is_error.unwrap_or(true),
        "session summarize response should not be error"
    );

    let body = result_json(&summarize_res);
    assert!(
        body["summary_id"].as_str().is_some() || body["session_id"].as_str().is_some(),
        "summarize response should contain summary_id or session_id: {body}"
    );
}
