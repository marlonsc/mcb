//! Tests for the `vcs` MCP tool.
//!
//! Actions: `list_repositories`, `index_repository`, `compare_branches`, `search_branch`, `analyze_impact`

use super::common::{
    TestResult, assert_tool_error, call_tool, cleanup_temp_dbs, create_client, extract_text,
    is_error,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_vcs_list_repositories() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "vcs",
        serde_json::json!({"action": "list_repositories"}),
    )
    .await?;
    assert!(!is_error(&result), "list_repositories should not error");
    assert!(
        !extract_text(&result).is_empty(),
        "list_repositories should return a response"
    );
    let _ = client.cancel().await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_vcs_search_branch() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "vcs",
        serde_json::json!({"action": "search_branch", "query": "main", "limit": 5}),
    )
    .await?;
    assert!(
        !extract_text(&result).is_empty(),
        "search_branch should return a response"
    );
    let _ = client.cancel().await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_vcs_compare_missing_branches() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "vcs",
        serde_json::json!({"action": "compare_branches"}),
    )
    .await;
    assert_tool_error(result, &["required", "branch", "path", "error"]);
    let _ = client.cancel().await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_vcs_invalid_action() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "vcs", serde_json::json!({"action": "nonexistent"})).await;
    assert_tool_error(result, &["unknown variant", "expected one of"]);
    let _ = client.cancel().await;
    cleanup_temp_dbs();
    Ok(())
}
