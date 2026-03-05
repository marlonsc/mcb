//! Tests for the `index` MCP tool.
//!
//! Actions: start, `git_index`, status, clear

use super::common::{call_tool, cleanup_temp_dbs, create_client, shutdown_client};
use mcb_domain::utils::tests::mcp_assertions::{assert_tool_error, extract_text, is_error};
use mcb_domain::utils::tests::utils::TestResult;
use rstest::rstest;
use serial_test::serial;

#[serial]
#[rstest]
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
#[rstest]
#[tokio::test]
async fn test_index_clear_missing_collection() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "index", serde_json::json!({"action": "clear"})).await?;
    assert!(
        !is_error(&result),
        "auto-context should provide collection and clear should succeed"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
#[tokio::test]
async fn test_index_start_missing_path() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "index", serde_json::json!({"action": "start"})).await?;
    assert!(
        !is_error(&result),
        "auto-context should provide path and start should succeed"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
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
