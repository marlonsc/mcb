//! Tests for public index MCP tools.
//!
//! Tools: `index_repo`, `index_status`, `clear_index`

use super::common::{call_tool, cleanup_temp_dbs, create_client, shutdown_client};
use mcb_domain::utils::tests::mcp_assertions::{assert_tool_error, extract_text, is_error};
use mcb_domain::utils::tests::utils::TestResult;
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_index_status() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "index_status", serde_json::json!({})).await?;
    assert!(!is_error(&result), "index status should not error");
    assert!(
        !extract_text(&result).is_empty(),
        "status should return text"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_index_clear_missing_collection() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "clear_index", serde_json::json!({})).await?;
    assert!(
        !is_error(&result),
        "auto-context should provide collection and clear should succeed"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_index_start_missing_path() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "index_repo", serde_json::json!({})).await?;
    assert!(
        !is_error(&result),
        "auto-context should provide path and start should succeed"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_index_repo_rejects_invalid_max_file_size() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "index_repo",
        serde_json::json!({"max_file_size": "not-a-number"}),
    )
    .await;
    assert_tool_error(result, &["max_file_size", "invalid type", "parse"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
