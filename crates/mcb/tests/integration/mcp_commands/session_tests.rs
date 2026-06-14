//! Tests for public session MCP tools.
//!
//! Tools: `start_session`, `get_session`, `list_sessions`, `summarize_session`

use super::common::{call_tool, cleanup_temp_dbs, create_client, shutdown_client};
use mcb_domain::utils::tests::mcp_assertions::{assert_tool_error, extract_text, is_error};
use mcb_domain::utils::tests::utils::TestResult;
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_session_list() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "list_sessions", serde_json::json!({"limit": 10})).await?;
    assert!(!is_error(&result), "session list should not error");
    assert!(
        !extract_text(&result).is_empty(),
        "session list should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_session_create() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "start_session",
        serde_json::json!({
            "agent_type": "explore", "data": {"model": "test-model"}
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

#[rstest]
#[tokio::test]
async fn test_session_get_nonexistent() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "get_session",
        serde_json::json!({
            "session_id": "00000000-0000-0000-0000-000000000099"
        }),
    )
    .await;
    assert_tool_error(result, &["not found", "error"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_session_summarize_without_id() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "summarize_session", serde_json::json!({})).await;
    assert_tool_error(result, &["session", "summary", "not found"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_start_session_requires_data() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "start_session",
        serde_json::json!({"agent_type": "explore"}),
    )
    .await;
    assert_tool_error(result, &["data", "payload"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
