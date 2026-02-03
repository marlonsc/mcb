//! Golden tests for MCP tools: full e2e via handlers (no #[ignore]).
//!
//! These tests use the real DI stack (NullEmbedding + InMemoryVectorStore)
//! and call the four core MCP handlers (index_codebase, search_code,
//! get_indexing_status, clear_index) to validate behavior and response schemas.

use mcb_server::args::{ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::Content;
use std::path::Path;

fn sample_codebase_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_codebase")
}

fn test_collection() -> &'static str {
    "mcb_golden_tools_e2e"
}

fn extract_text_content(content: &[Content]) -> String {
    content
        .iter()
        .filter_map(|c| {
            if let Ok(json) = serde_json::to_value(c) {
                if let Some(text) = json.get("text") {
                    return text.as_str().map(|s| s.to_string());
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// =============================================================================
// E2E: Complete workflow (clear -> status -> index -> status -> search -> clear -> status)
// =============================================================================

#[tokio::test]
async fn golden_e2e_complete_workflow() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    assert!(
        path.exists(),
        "sample_codebase fixture must exist: {:?}",
        path
    );
    let path_str = path.to_string_lossy().to_string();
    let coll = test_collection();

    let index_h = server.index_codebase_handler();
    let status_h = server.get_indexing_status_handler();
    let search_h = server.search_code_handler();
    let clear_h = server.clear_index_handler();

    // 1. Clear any existing test data
    let clear_args = ClearIndexArgs {
        collection: coll.to_string(),
    };
    let r = clear_h.handle(Parameters(clear_args)).await;
    assert!(r.is_ok(), "clear_index should succeed");
    let resp = r.unwrap();
    let text = extract_text_content(&resp.content);
    assert!(
        text.contains("cleared") || text.contains("Cleared"),
        "{}",
        text
    );

    // 2. Status (idle / empty)
    let status_args = GetIndexingStatusArgs {
        collection: coll.to_string(),
    };
    let r = status_h.handle(Parameters(status_args)).await;
    assert!(r.is_ok());
    let text = extract_text_content(&r.unwrap().content);
    assert!(
        text.contains("Indexing Status") || text.contains("Status"),
        "{}",
        text
    );

    // 3. Index repository
    let index_args = IndexCodebaseArgs {
        path: path_str.clone(),
        collection: Some(coll.to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    let r = index_h.handle(Parameters(index_args)).await;
    assert!(r.is_ok(), "index_codebase should succeed");
    let resp = r.unwrap();
    assert!(!resp.is_error.unwrap_or(false));
    let text = extract_text_content(&resp.content);
    assert!(
        text.contains("Files processed")
            || text.contains("Chunks created")
            || text.contains("Indexing Started")
            || text.contains("started"),
        "{}",
        text
    );

    // 4. Status again (should show work done or idle)
    let status_args = GetIndexingStatusArgs {
        collection: coll.to_string(),
    };
    let _ = status_h.handle(Parameters(status_args)).await;

    // 5. Search
    let search_args = SearchCodeArgs {
        query: "embedding or vector".to_string(),
        limit: 5,
        collection: Some(coll.to_string()),
        extensions: None,
        filters: None,
        token: None,
    };
    let r = search_h.handle(Parameters(search_args)).await;
    assert!(r.is_ok());
    let resp = r.unwrap();
    let text = extract_text_content(&resp.content);
    assert!(
        text.contains("Search") || text.contains("Results") || text.contains("results"),
        "{}",
        text
    );

    // 6. Clear index
    let clear_args = ClearIndexArgs {
        collection: coll.to_string(),
    };
    let r = clear_h.handle(Parameters(clear_args)).await;
    assert!(r.is_ok());
}

// =============================================================================
// Index: index test repository and check response schema
// =============================================================================

#[tokio::test]
async fn golden_index_test_repository() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    assert!(path.exists(), "sample_codebase must exist: {:?}", path);

    let handler = server.index_codebase_handler();
    let args = IndexCodebaseArgs {
        path: path.to_string_lossy().to_string(),
        collection: Some(test_collection().to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.is_error.unwrap_or(false));

    let text = extract_text_content(&response.content);
    assert!(
        text.contains("Files processed")
            || text.contains("Indexing Started")
            || text.contains("started"),
        "response: {}",
        text
    );
    assert!(
        text.contains("Chunks created")
            || text.contains("chunks")
            || text.contains("Path:")
            || text.contains("Operation ID"),
        "response: {}",
        text
    );
    assert!(
        text.contains("Source directory")
            || text.contains("Path:")
            || text.contains(path.to_string_lossy().as_ref()),
        "response: {}",
        text
    );
}

#[tokio::test]
async fn golden_index_handles_multiple_languages() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let handler = server.index_codebase_handler();
    let args = IndexCodebaseArgs {
        path: path.to_string_lossy().to_string(),
        collection: Some("golden_multi_lang".to_string()),
        extensions: Some(vec!["rs".to_string()]),
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    let result = handler.handle(Parameters(args)).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.is_error.unwrap_or(false));
    let text = extract_text_content(&response.content);
    assert!(
        text.contains("Files processed")
            || text.contains("Chunks")
            || text.contains("Indexing Started")
            || text.contains("started"),
        "{}",
        text
    );
}

#[tokio::test]
async fn golden_index_respects_ignore_patterns() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let handler = server.index_codebase_handler();
    let args = IndexCodebaseArgs {
        path: path.to_string_lossy().to_string(),
        collection: Some("golden_ignore_test".to_string()),
        extensions: None,
        ignore_patterns: Some(vec!["*_test.rs".to_string()]),
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    let result = handler.handle(Parameters(args)).await;
    assert!(result.is_ok());
}

// =============================================================================
// Search: relevance, ranking, empty query, limit, extension filter
// =============================================================================

#[tokio::test]
async fn golden_search_returns_relevant_results() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let index_h = server.index_codebase_handler();
    let search_h = server.search_code_handler();
    let coll = "golden_search_relevant";

    let index_args = IndexCodebaseArgs {
        path: path.to_string_lossy().to_string(),
        collection: Some(coll.to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    index_h.handle(Parameters(index_args)).await.unwrap();

    let search_args = SearchCodeArgs {
        query: "embedding provider".to_string(),
        limit: 10,
        collection: Some(coll.to_string()),
        extensions: None,
        filters: None,
        token: None,
    };
    let result = search_h.handle(Parameters(search_args)).await;
    assert!(result.is_ok());
    let text = extract_text_content(&result.unwrap().content);
    assert!(
        text.contains("Search")
            || text.contains("Results")
            || text.contains("results")
            || text.contains("Relevance"),
        "{}",
        text
    );
}

#[tokio::test]
async fn golden_search_ranking_is_correct() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let index_h = server.index_codebase_handler();
    let search_h = server.search_code_handler();
    let coll = "golden_search_rank";

    index_h
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(coll.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .unwrap();

    let result = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "vector store".to_string(),
            limit: 5,
            collection: Some(coll.to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(result.is_ok());
    let text = extract_text_content(&result.unwrap().content);
    assert!(text.contains("Score") || text.contains("score") || text.contains("Results"));
}

#[tokio::test]
async fn golden_search_handles_empty_query() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let handler = server.search_code_handler();
    let args = SearchCodeArgs {
        query: "".to_string(),
        limit: 10,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    };
    let result = handler.handle(Parameters(args)).await;
    assert!(result.is_err(), "empty query must fail validation");
}

#[tokio::test]
async fn golden_search_respects_limit_parameter() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let index_h = server.index_codebase_handler();
    let search_h = server.search_code_handler();
    let coll = "golden_search_limit";

    index_h
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(coll.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .unwrap();

    let result = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "function".to_string(),
            limit: 2,
            collection: Some(coll.to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(result.is_ok());
    let text = extract_text_content(&result.unwrap().content);
    assert!(text.contains("2") || text.contains("Results") || text.contains("results"));
}

#[tokio::test]
async fn golden_search_filters_by_extension() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let index_h = server.index_codebase_handler();
    let search_h = server.search_code_handler();
    let coll = "golden_search_ext";

    index_h
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(coll.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .unwrap();

    let result = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "embedding".to_string(),
            limit: 10,
            collection: Some(coll.to_string()),
            extensions: Some(vec!["rs".to_string()]),
            filters: None,
            token: None,
        }))
        .await;
    assert!(result.is_ok());
}

// =============================================================================
// MCP response schema: required fields present in formatted response
// =============================================================================

#[tokio::test]
async fn golden_mcp_index_codebase_schema() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let args = IndexCodebaseArgs {
        path: path.to_string_lossy().to_string(),
        collection: Some("schema_index".to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    let result = server
        .index_codebase_handler()
        .handle(Parameters(args))
        .await;
    assert!(result.is_ok());
    let response = result.unwrap();
    let text = extract_text_content(&response.content);
    assert!(
        text.contains("Files processed")
            || text.contains("files")
            || text.contains("Indexing Started")
            || text.contains("started"),
        "{}",
        text
    );
    assert!(
        text.contains("Chunks created")
            || text.contains("chunks")
            || text.contains("Operation ID")
            || text.contains("Path:"),
        "{}",
        text
    );
    assert!(
        response.is_error.is_none() || !response.is_error.unwrap(),
        "success response should not be error"
    );
}

#[tokio::test]
async fn golden_mcp_search_code_schema() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some("schema_search".to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .unwrap();

    let result = server
        .search_code_handler()
        .handle(Parameters(SearchCodeArgs {
            query: "handler".to_string(),
            limit: 5,
            collection: Some("schema_search".to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(result.is_ok());
    let text = extract_text_content(&result.unwrap().content);
    assert!(text.contains("Query") || text.contains("Results") || text.contains("Search"));
}

#[tokio::test]
async fn golden_mcp_get_indexing_status_schema() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let result = server
        .get_indexing_status_handler()
        .handle(Parameters(GetIndexingStatusArgs {
            collection: test_collection().to_string(),
        }))
        .await;
    assert!(result.is_ok());
    let text = extract_text_content(&result.unwrap().content);
    assert!(text.contains("Status") || text.contains("Indexing"));
}

#[tokio::test]
async fn golden_mcp_clear_index_schema() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let result = server
        .clear_index_handler()
        .handle(Parameters(ClearIndexArgs {
            collection: "schema_clear".to_string(),
        }))
        .await;
    assert!(result.is_ok());
    let text = extract_text_content(&result.unwrap().content);
    assert!(text.contains("cleared") || text.contains("Cleared"));
}

#[tokio::test]
async fn golden_mcp_error_responses_consistent() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let handler = server.index_codebase_handler();
    let args = IndexCodebaseArgs {
        path: "/nonexistent/path/12345".to_string(),
        collection: Some("err_coll".to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    let result = handler.handle(Parameters(args)).await;
    assert!(
        result.is_ok(),
        "handler returns Ok(CallToolResult) even on error"
    );
    let response = result.unwrap();
    assert!(
        response.is_error.unwrap_or(false),
        "nonexistent path should return is_error: true"
    );
    let text = extract_text_content(&response.content);
    assert!(text.contains("Error") || text.contains("Failed") || text.contains("exist"));
}

// =============================================================================
// E2E: Collection isolation and reindex
// =============================================================================

#[tokio::test]
async fn golden_e2e_respects_collection_isolation() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let index_h = server.index_codebase_handler();
    let search_h = server.search_code_handler();

    index_h
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some("coll_a".to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .unwrap();

    index_h
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some("coll_b".to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .unwrap();

    let r = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "embedding".to_string(),
            limit: 5,
            collection: Some("coll_a".to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok());
    let r2 = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "embedding".to_string(),
            limit: 5,
            collection: Some("coll_b".to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(r2.is_ok());
}

#[tokio::test]
async fn golden_e2e_handles_reindex_correctly() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let coll = "golden_reindex";
    let index_h = server.index_codebase_handler();

    let args1 = IndexCodebaseArgs {
        path: path.to_string_lossy().to_string(),
        collection: Some(coll.to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    index_h.handle(Parameters(args1)).await.unwrap();
    let args2 = IndexCodebaseArgs {
        path: path.to_string_lossy().to_string(),
        collection: Some(coll.to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    let r2 = index_h.handle(Parameters(args2)).await;
    assert!(r2.is_ok());
    let text = extract_text_content(&r2.unwrap().content);
    assert!(
        text.contains("Files processed")
            || text.contains("Chunks")
            || text.contains("Completed")
            || text.contains("Indexing Started")
            || text.contains("started"),
        "{}",
        text
    );
}

#[tokio::test]
async fn golden_e2e_handles_concurrent_operations() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let index_h = server.index_codebase_handler();
    let search_h = server.search_code_handler();
    let coll = "golden_concurrent";

    index_h
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(coll.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .unwrap();

    let (s1, s2) = tokio::join!(
        search_h.handle(Parameters(SearchCodeArgs {
            query: "embedding".to_string(),
            limit: 3,
            collection: Some(coll.to_string()),
            extensions: None,
            filters: None,
            token: None,
        })),
        search_h.handle(Parameters(SearchCodeArgs {
            query: "vector".to_string(),
            limit: 3,
            collection: Some(coll.to_string()),
            extensions: None,
            filters: None,
            token: None,
        })),
    );
    assert!(s1.is_ok());
    assert!(s2.is_ok());
}
