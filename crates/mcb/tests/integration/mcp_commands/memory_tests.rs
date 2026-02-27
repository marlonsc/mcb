//! Tests for the `memory` MCP tool.
//!
//! Actions: store, get, list, timeline, inject
//! Resources: observation, execution, `quality_gate`, `error_pattern`, session

use super::common::{
    TestResult, assert_tool_error, call_tool, cleanup_temp_dbs, create_client, extract_text,
    is_error, shutdown_client,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_memory_list_observations() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "memory",
        serde_json::json!({"action": "list", "resource": "observation", "limit": 10}),
    )
    .await?;
    assert!(
        !extract_text(&result).is_empty(),
        "memory list should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_memory_store_and_list() -> TestResult {
    let client = create_client().await?;
    let store_result = call_tool(
        &client,
        "memory",
        serde_json::json!({
            "action": "store", "resource": "observation", "project_id": "test-proj",
            "data": {"content": "Test observation from TDD", "observation_type": "context", "tags": ["test"]}
        }),
    )
    .await?;
    assert!(
        !is_error(&store_result),
        "store should succeed, got: {}",
        extract_text(&store_result)
    );

    let list_result = call_tool(
        &client,
        "memory",
        serde_json::json!({"action": "list", "resource": "observation", "limit": 50}),
    )
    .await?;
    assert!(
        !extract_text(&list_result).is_empty(),
        "list should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_memory_get_missing_ids() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "memory",
        serde_json::json!({"action": "get", "resource": "observation"}),
    )
    .await;
    assert_tool_error(result, &["id", "required", "error"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_memory_timeline() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "memory",
        serde_json::json!({"action": "timeline", "resource": "observation"}),
    )
    .await?;
    assert!(
        !extract_text(&result).is_empty(),
        "timeline should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_memory_invalid_action() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "memory",
        serde_json::json!({"action": "nonexistent", "resource": "observation"}),
    )
    .await;
    assert_tool_error(result, &["unknown variant", "expected one of"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
