//! Tests for the `search` MCP tool.
//!
//! Resources: code, memory, context

use super::common::{
    TestResult, assert_tool_error, call_tool, cleanup_temp_dbs, create_client, extract_text,
    shutdown_client,
};
use rstest::rstest;
use serial_test::serial;

#[serial]
#[rstest]
#[tokio::test]
async fn test_search_memory() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "search",
        serde_json::json!({"query": "test", "resource": "memory", "limit": 5}),
    )
    .await?;
    assert!(
        !extract_text(&result).is_empty(),
        "memory search should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
#[tokio::test]
async fn test_search_code_missing_collection() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "search",
        serde_json::json!({"query": "test", "resource": "code", "limit": 5}),
    )
    .await;
    assert_tool_error(result, &["collection", "required"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
#[tokio::test]
async fn test_search_missing_query() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "search", serde_json::json!({"resource": "code"})).await;
    assert_tool_error(result, &["query", "missing field"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
#[tokio::test]
async fn test_search_invalid_resource() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "search",
        serde_json::json!({"query": "test", "resource": "nonexistent"}),
    )
    .await;
    assert_tool_error(result, &["unknown variant", "expected one of"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
