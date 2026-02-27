//! Tests for the `index` MCP tool.
//!
//! Actions: start, `git_index`, status, clear

use super::common::{
    TestResult, assert_tool_error, call_tool, cleanup_temp_dbs, create_client, extract_text,
    is_error, shutdown_client,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_index_status() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "index", serde_json::json!({"action": "status"})).await?;
    assert!(!is_error(&result), "index status should not error");
    assert!(
        !extract_text(&result).is_empty(),
        "status should return text"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_index_clear_missing_collection() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "index", serde_json::json!({"action": "clear"})).await;
    assert_tool_error(result, &["collection", "required"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_index_start_missing_path() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "index", serde_json::json!({"action": "start"})).await;
    assert_tool_error(result, &["path", "required"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_index_invalid_action() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "index",
        serde_json::json!({"action": "nonexistent"}),
    )
    .await;
    assert_tool_error(result, &["unknown variant", "expected one of"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
