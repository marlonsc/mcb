//! Tests for public VCS MCP tools.
//!
//! Tools: `list_repos`, `compare_branches`, `analyze_impact`

use super::common::{call_tool, cleanup_temp_dbs, create_client, shutdown_client};
use mcb_domain::utils::tests::mcp_assertions::{assert_tool_error, extract_text, is_error};
use mcb_domain::utils::tests::utils::TestResult;
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_vcs_list_repositories() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "list_repos", serde_json::json!({})).await?;
    assert!(!is_error(&result), "list_repositories should not error");
    assert!(
        !extract_text(&result).is_empty(),
        "list_repositories should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_vcs_analyze_impact() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "analyze_impact", serde_json::json!({"limit": 5})).await?;
    assert!(
        !extract_text(&result).is_empty(),
        "analyze_impact should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_vcs_compare_default_branches() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "compare_branches",
        serde_json::json!({"base_branch": "HEAD", "target_branch": "HEAD"}),
    )
    .await?;
    assert!(
        !is_error(&result),
        "compare_branches with defaults should not error, got: {}",
        extract_text(&result)
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_compare_branches_requires_base_branch() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "compare_branches",
        serde_json::json!({"target_branch": "HEAD"}),
    )
    .await;
    assert_tool_error(result, &["base_branch", "missing field"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
