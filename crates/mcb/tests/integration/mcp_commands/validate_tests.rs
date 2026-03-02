//! Tests for the `validate` MCP tool.
//!
//! Actions: run, `list_rules`, analyze

use super::common::{
    TestResult, assert_tool_error, call_tool, cleanup_temp_dbs, create_client, extract_text,
    is_error, shutdown_client,
};
use rstest::rstest;
use serial_test::serial;

#[serial]
#[rstest]
#[tokio::test]
async fn test_validate_list_rules() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "validate",
        serde_json::json!({"action": "list_rules"}),
    )
    .await?;
    assert!(!is_error(&result), "list_rules should not error");
    let text = extract_text(&result);
    assert!(!text.is_empty(), "list_rules should return rules");
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
#[tokio::test]
async fn test_validate_list_rules_with_category() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "validate",
        serde_json::json!({"action": "list_rules", "category": "architecture"}),
    )
    .await?;
    assert!(
        !is_error(&result),
        "list_rules with category should not error"
    );
    assert!(!extract_text(&result).is_empty(), "should return rules");
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
#[tokio::test]
async fn test_validate_run_missing_path() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(&client, "validate", serde_json::json!({"action": "run"})).await;
    assert_tool_error(result, &["path", "required"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}

#[serial]
#[rstest]
#[tokio::test]
async fn test_validate_invalid_action() -> TestResult {
    let client = create_client().await?;
    let result = call_tool(
        &client,
        "validate",
        serde_json::json!({"action": "nonexistent"}),
    )
    .await;
    assert_tool_error(result, &["unknown variant", "expected one of"]);
    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
