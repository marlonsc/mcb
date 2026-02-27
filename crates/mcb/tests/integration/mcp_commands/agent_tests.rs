//! Tests for the `agent` MCP tool.
//!
//! Actions: `log_tool`, `log_delegation`

use super::common::{
    TestResult, assert_tool_error, call_tool, cleanup_temp_dbs, create_client, extract_text,
    shutdown_client,
};
use serial_test::serial;

#[serial]
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
    .await;
    assert_tool_error(result, &["session", "not found"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
