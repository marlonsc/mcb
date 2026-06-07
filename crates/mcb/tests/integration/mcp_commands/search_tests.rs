//! Tests for public search MCP tools.
//!
//! Tools: `search_code`, `search_memory`

use super::common::{call_tool, cleanup_temp_dbs, create_client, shutdown_client};
use mcb_domain::utils::tests::mcp_assertions::{assert_tool_error, extract_text, is_error};
use mcb_domain::utils::tests::utils::TestResult;
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_search_memory() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "search_memory",
        serde_json::json!({"query": "test", "limit": 5}),
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

#[rstest]
#[tokio::test]
async fn test_search_code_missing_collection() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "search_code",
        serde_json::json!({"query": "test", "limit": 5}),
    )
    .await?;
    assert!(
        !is_error(&result),
        "auto-context should provide collection and search code should succeed"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_search_missing_query() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "search_code", serde_json::json!({})).await;
    assert_tool_error(result, &["query", "missing field"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_search_rejects_empty_query() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "search_memory", serde_json::json!({"query": ""})).await;
    assert_tool_error(result, &["query", "validation", "length"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
