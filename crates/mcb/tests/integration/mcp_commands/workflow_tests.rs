//! Real MCP workflow tests through the public stdio transport.
//!
//! This validates the command ergonomics an agent exercises through MCP:
//! session, project, memory, indexing, and code search in one flow.

use super::common::{call_tool, cleanup_temp_dbs, create_client, shutdown_client};
use mcb_domain::utils::tests::fixtures::sample_codebase_path;
use mcb_domain::utils::tests::mcp_assertions::{extract_text, is_error};
use mcb_domain::utils::tests::utils::TestResult;
use mcb_utils::constants::testing::SAMPLE_CODEBASE_FILES;
use rstest::rstest;
use tokio::time::{Duration, Instant};

fn workflow_collection() -> String {
    format!(
        "stdio-workflow-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

fn parse_results_found(text: &str) -> usize {
    text.lines()
        .find(|line| line.contains("Results found"))
        .and_then(|line| line.rsplit_once(':').map(|(_, value)| value))
        .map(|value| value.trim().trim_start_matches('*').trim())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

async fn wait_for_search_result(
    client: &rmcp::service::RunningService<rmcp::RoleClient, ()>,
    collection: &str,
) -> TestResult<String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut last_text = String::new();
    while Instant::now() < deadline {
        let result = call_tool(
            client,
            "search_code",
            serde_json::json!({
                "query": "handle MCP index codebase request",
                "collection": collection,
                "limit": 5
            }),
        )
        .await?;
        assert!(
            !is_error(&result),
            "search_code should not return an error: {}",
            extract_text(&result)
        );
        let text = extract_text(&result);
        if parse_results_found(&text) > 0
            && SAMPLE_CODEBASE_FILES.iter().any(|file| text.contains(file))
        {
            return Ok(text);
        }
        last_text = text;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    unreachable!("search_code did not return a sample fixture file after indexing: {last_text}");
}

#[rstest]
#[tokio::test]
async fn test_stdio_mcp_real_workflow_indexes_and_searches_code() -> TestResult {
    let client = create_client().await?;
    let collection = workflow_collection();
    let fixture_path = sample_codebase_path();

    let create_project = call_tool(
        &client,
        "project",
        serde_json::json!({
            "action": "create",
            "resource": "project",
            "project_id": "stdio-workflow-project",
            "data": {
                "name": "Stdio Workflow Project",
                "path": fixture_path.to_string_lossy()
            }
        }),
    )
    .await?;
    assert!(
        !is_error(&create_project),
        "project create should succeed: {}",
        extract_text(&create_project)
    );

    let session = call_tool(
        &client,
        "start_session",
        serde_json::json!({
            "agent_type": "explore",
            "project_id": "stdio-workflow-project",
            "data": {"model": "test-model", "prompt_summary": "validate real MCP workflow"}
        }),
    )
    .await?;
    assert!(
        !is_error(&session),
        "start_session should succeed: {}",
        extract_text(&session)
    );

    let memory = call_tool(
        &client,
        "store_memory",
        serde_json::json!({
            "project_id": "stdio-workflow-project",
            "data": {
                "content": "Stdio workflow memory proves real persistence",
                "observation_type": "context",
                "tags": ["workflow", "stdio"]
            }
        }),
    )
    .await?;
    assert!(
        !is_error(&memory),
        "store_memory should succeed: {}",
        extract_text(&memory)
    );

    let clear = call_tool(
        &client,
        "clear_index",
        serde_json::json!({
            "path": fixture_path.to_string_lossy(),
            "collection": collection
        }),
    )
    .await?;
    assert!(
        !is_error(&clear),
        "clear_index should succeed: {}",
        extract_text(&clear)
    );

    let index = call_tool(
        &client,
        "index_repo",
        serde_json::json!({
            "path": fixture_path.to_string_lossy(),
            "collection": collection,
            "extensions": ["rs"]
        }),
    )
    .await?;
    assert!(
        !is_error(&index),
        "index_repo should succeed: {}",
        extract_text(&index)
    );

    let search_text = wait_for_search_result(&client, &collection).await?;
    assert!(
        search_text.contains("Semantic Code Search Results"),
        "search response should be readable MCP text: {search_text}"
    );

    let memories = call_tool(
        &client,
        "list_memories",
        serde_json::json!({
            "project_id": "stdio-workflow-project",
            "query": "real persistence",
            "limit": 10
        }),
    )
    .await?;
    let memories_text = extract_text(&memories);
    assert!(
        memories_text.contains("Stdio workflow memory"),
        "list_memories should return stored content: {memories_text}"
    );

    shutdown_client(client).await;
    cleanup_temp_dbs();
    Ok(())
}
