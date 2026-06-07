//! Tests for the `agent` MCP tool.
//!
//! Actions: `log_tool`, `log_delegation`

use super::common::{
    TestResult, call_tool, cleanup_temp_dbs, create_client, extract_text, is_error, shutdown_client,
};
use rstest::rstest;
use serial_test::serial;

#[serial]
#[rstest]
#[tokio::test]
async fn test_agent_log_tool() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "agent",
        serde_json::json!({
            "action": "log_tool",
            "session_id": "00000000-0000-0000-0000-000000000001",
            "data": {"tool_name": "search", "success": true, "duration_ms": 150}
        }),
    )
    .await?;
    assert!(
        !extract_text(&result).is_empty(),
        "log_tool should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
#[tokio::test]
async fn test_agent_log_delegation() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "agent", serde_json::json!({
        "action": "log_delegation",
        "session_id": "00000000-0000-0000-0000-000000000001",
        "data": {"child_session_id": "00000000-0000-0000-0000-000000000002", "prompt": "Find auth", "success": true}
    })).await?;
    assert!(
        !extract_text(&result).is_empty(),
        "log_delegation should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
#[tokio::test]
async fn test_agent_missing_session_id() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "agent",
        serde_json::json!({
            "action": "log_tool", "data": {"tool_name": "search", "success": true}
        }),
    )
    .await?;
    assert!(
        !is_error(&result),
        "auto-context should provide a session_id and log_tool should succeed"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
