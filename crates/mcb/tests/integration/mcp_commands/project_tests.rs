//! Tests for the `project` MCP tool.
//!
//! Actions: create, get, update, list, delete
//! Resources: project, phase, issue, dependency, decision

use super::common::{
    TestResult, assert_tool_error, call_tool, cleanup_temp_dbs, create_client, extract_text,
    shutdown_client,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_project_list() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "project",
        serde_json::json!({"action": "list", "resource": "project", "project_id": "test-proj"}),
    )
    .await?;
    assert!(
        !extract_text(&result).is_empty(),
        "project list should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_project_list_issues() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "project",
        serde_json::json!({"action": "list", "resource": "issue", "project_id": "test-proj"}),
    )
    .await;
    assert_tool_error(result, &["unsupported", "list", "issue"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_project_get_nonexistent() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "project", serde_json::json!({"action": "get", "resource": "project", "project_id": "nonexistent-project"})).await;
    assert_tool_error(result, &["not found", "error"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_project_invalid_resource() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "project", serde_json::json!({"action": "list", "resource": "nonexistent_resource", "project_id": "test"})).await;
    assert_tool_error(result, &["unknown variant", "expected one of"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
