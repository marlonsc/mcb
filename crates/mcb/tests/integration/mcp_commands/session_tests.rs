//! Tests for the `session` MCP tool.
//!
//! Actions: create, get, update, list, summarize

use super::common::{
    TestResult, assert_tool_error, call_tool, cleanup_temp_dbs, create_client, extract_text,
    is_error, shutdown_client,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_session_list() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "session",
        serde_json::json!({"action": "list", "limit": 10}),
    )
    .await?;
    assert!(!is_error(&result), "session list should not error");
    assert!(
        !extract_text(&result).is_empty(),
        "session list should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_session_create() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "session",
        serde_json::json!({
            "action": "create", "data": {"model": "test-model", "agent_type": "explore"}
        }),
    )
    .await;
    assert_tool_error(result, &["project_id", "required"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_session_get_nonexistent() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "session",
        serde_json::json!({
            "action": "get", "session_id": "00000000-0000-0000-0000-000000000099"
        }),
    )
    .await;
    assert_tool_error(result, &["not found", "error"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_session_summarize_without_id() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "session",
        serde_json::json!({"action": "summarize"}),
    )
    .await;
    assert_tool_error(result, &["session", "summary", "not found"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_session_invalid_action() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "session",
        serde_json::json!({"action": "nonexistent"}),
    )
    .await;
    assert_tool_error(result, &["unknown variant", "expected one of"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
