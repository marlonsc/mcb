//! Tests for the `session` MCP tool.
//!
//! Actions: create, get, update, list, summarize

use super::common::{call_tool, cleanup_temp_dbs, create_client, shutdown_client};
use mcb_domain::utils::tests::mcp_assertions::{assert_tool_error, extract_text, is_error};
use mcb_domain::utils::tests::utils::TestResult;
use rstest::rstest;
use serial_test::serial;

#[serial]
#[rstest]
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
#[rstest]
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
    .await?;
    assert!(
        !is_error(&result),
        "auto-context should provide project_id and create session should succeed"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
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
#[rstest]
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
#[rstest]
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
