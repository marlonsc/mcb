//! Tests for the `entity` MCP tool (unified CRUD).
//!
//! Actions: create, get, update, list, delete, release
//! Resources: repository, branch, worktree, assignment, plan, version, review,
//!            issue, comment, label, `label_assignment`, org, user, team,
//!            `team_member`, `api_key`

use super::common::{
    TestResult, assert_tool_error, call_tool, cleanup_temp_dbs, create_client, extract_text,
    is_error, shutdown_client,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_entity_list_orgs() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "entity",
        serde_json::json!({"action": "list", "resource": "org"}),
    )
    .await?;
    assert!(!is_error(&result), "list orgs should not error");
    assert!(
        !extract_text(&result).is_empty(),
        "list orgs should return a response"
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_entity_list_repositories() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "entity",
        serde_json::json!({"action": "list", "resource": "repository", "project_id": "test-project"}),
    )
    .await?;
    assert!(
        !is_error(&result),
        "list repositories should not error, got: {}",
        extract_text(&result)
    );
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_entity_get_nonexistent() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "entity",
        serde_json::json!({"action": "get", "resource": "org", "id": "nonexistent-org-id"}),
    )
    .await;
    assert_tool_error(result, &["not found", "error"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_entity_list_plans_requires_project() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "entity",
        serde_json::json!({"action": "list", "resource": "plan"}),
    )
    .await;
    assert_tool_error(result, &["project_id", "required"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[tokio::test]
async fn test_entity_invalid_resource() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "entity",
        serde_json::json!({"action": "list", "resource": "nonexistent_resource"}),
    )
    .await;
    assert_tool_error(result, &["unknown variant", "expected one of"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
