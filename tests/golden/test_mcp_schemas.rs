//! Golden tests: MCP response schema (content shape) for all 4 tools + error consistency.
//! Contract: docs/testing/GOLDEN_TESTS_CONTRACT.md.

use crate::test_utils::test_fixtures::{
    create_test_mcp_server, golden_content_to_string, sample_codebase_path,
};
use mcb_server::args::{ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn golden_mcp_index_codebase_schema() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let r = server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some("schema_test".to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "index_codebase must succeed");
    let res = r.unwrap();
    assert!(!res.content.is_empty(), "response must have content");
    assert!(
        !res.is_error.unwrap_or(true),
        "success response must not be error"
    );
    let text = golden_content_to_string(&res);
    assert!(
        text.contains("Files processed")
            || text.contains("Chunks")
            || text.contains("chunk")
            || text.contains("Indexing Started")
            || text.contains("Path:")
            || text.contains("Source directory"),
        "index schema must include files/chunks/path: {}",
        text
    );
}

#[tokio::test]
async fn golden_mcp_search_code_schema() {
    let server = create_test_mcp_server().await;
    let search_h = server.search_code_handler();
    let r = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "test".to_string(),
            limit: 3,
            collection: Some("default".to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search_code must succeed");
    let res = r.unwrap();
    assert!(!res.content.is_empty(), "search response must have content");
    let text = golden_content_to_string(&res);
    assert!(
        text.contains("Search")
            || text.contains("Results")
            || text.contains("Results found")
            || text.contains("No Results")
            || text.contains("Query"),
        "search schema must include Search/Results/Query: {}",
        text
    );
}

#[tokio::test]
async fn golden_mcp_get_indexing_status_schema() {
    let server = create_test_mcp_server().await;
    let r = server
        .get_indexing_status_handler()
        .handle(Parameters(GetIndexingStatusArgs {
            collection: "default".to_string(),
        }))
        .await;
    assert!(r.is_ok());
    let res = r.unwrap();
    assert!(!res.content.is_empty());
    let text = golden_content_to_string(&res);
    assert!(text.contains("Status") || text.contains("indexing") || text.contains("Idle"));
}

#[tokio::test]
async fn golden_mcp_clear_index_schema() {
    let server = create_test_mcp_server().await;
    let r = server
        .clear_index_handler()
        .handle(Parameters(ClearIndexArgs {
            collection: "default".to_string(),
        }))
        .await;
    assert!(r.is_ok());
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = golden_content_to_string(&res);
    assert!(text.contains("Clear") || text.contains("clear") || text.contains("Collection"));
}

#[tokio::test]
async fn golden_mcp_error_responses_consistent() {
    let server = create_test_mcp_server().await;
    let search_h = server.search_code_handler();
    let r = search_h.handle(Parameters(SearchCodeArgs {
        query: String::new(),
        limit: 10,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    }));
    let result = r.await;
    assert!(result.is_err(), "empty query should yield MCP error");
}
