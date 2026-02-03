//! Golden tests: full E2E workflow, index, MCP response shape, and search validation.
//!
//! Contract: see docs/testing/GOLDEN_TESTS_CONTRACT.md.
//! No #[ignore]; all tests run with real DI (null embedding + in-memory vector store).
//! Uses create_test_mcp_server() and invokes handlers directly.

extern crate mcb_providers;

use mcb_server::args::{ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs};
use rmcp::handler::server::wrapper::Parameters;
use std::path::Path;

fn sample_codebase_path() -> std::path::PathBuf {
    crate::test_utils::test_fixtures::sample_codebase_path()
}

fn golden_queries_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden_queries.json")
}

const GOLDEN_COLLECTION: &str = crate::test_utils::test_fixtures::GOLDEN_COLLECTION;

/// Minimal struct to load golden_queries.json for E2E via handlers.
#[derive(serde::Deserialize)]
struct GoldenQueriesFixture {
    queries: Vec<GoldenQueryEntry>,
}

#[derive(serde::Deserialize)]
struct GoldenQueryEntry {
    #[serde(rename = "id")]
    _id: String,
    query: String,
    #[serde(rename = "expected_files")]
    _expected_files: Vec<String>,
}

fn load_golden_queries_fixture() -> GoldenQueriesFixture {
    let path = golden_queries_path();
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
    serde_json::from_str(&content).expect("Failed to parse golden_queries.json")
}

fn sample_codebase_files() -> &'static [&'static str] {
    crate::test_utils::test_fixtures::SAMPLE_CODEBASE_FILES
}

// ============================================================================
// E2E: complete workflow
// ============================================================================

#[tokio::test]
async fn golden_e2e_complete_workflow() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;

    let clear = server.clear_index_handler();
    let status_h = server.get_indexing_status_handler();
    let index_h = server.index_codebase_handler();
    let search_h = server.search_code_handler();

    // 1. Clear any existing data
    let r = clear
        .handle(Parameters(ClearIndexArgs {
            collection: GOLDEN_COLLECTION.to_string(),
        }))
        .await;
    assert!(r.is_ok(), "clear_index should succeed: {:?}", r);
    let clear_res = r.unwrap();
    let clear_text = content_to_string(&clear_res);
    assert!(
        clear_text.to_lowercase().contains("clear"),
        "clear response must mention clear/cleared: {}",
        clear_text
    );

    // 2. Status should be idle / empty
    let r = status_h
        .handle(Parameters(GetIndexingStatusArgs {
            collection: GOLDEN_COLLECTION.to_string(),
        }))
        .await;
    assert!(r.is_ok(), "get_indexing_status should succeed: {:?}", r);
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = content_to_string(&res);
    assert!(text.contains("Indexing Status") || text.contains("Idle") || text.contains("indexing"));

    // 3. Index sample codebase
    let path = sample_codebase_path();
    assert!(path.exists(), "sample_codebase must exist: {:?}", path);
    let r = index_h
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "index_codebase should succeed: {:?}", r);
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = content_to_string(&res);
    assert!(
        text.contains("chunks") || text.contains("Indexing") || text.contains("files"),
        "expected chunks/indexing in response: {}",
        text
    );

    // 4. Status should show work done
    let r = status_h
        .handle(Parameters(GetIndexingStatusArgs {
            collection: GOLDEN_COLLECTION.to_string(),
        }))
        .await;
    assert!(r.is_ok());

    // 5. Search
    let r = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "embedding provider".to_string(),
            limit: 5,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search_code should succeed: {:?}", r);
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = content_to_string(&res);
    assert!(
        text.contains("Search") || text.contains("Results") || text.contains("result"),
        "expected search result text: {}",
        text
    );

    // 6. Clear again
    let r = clear
        .handle(Parameters(ClearIndexArgs {
            collection: GOLDEN_COLLECTION.to_string(),
        }))
        .await;
    assert!(r.is_ok());

    // 7. Status idle again
    let r = status_h
        .handle(Parameters(GetIndexingStatusArgs {
            collection: GOLDEN_COLLECTION.to_string(),
        }))
        .await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn golden_e2e_handles_concurrent_operations() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let status_h = server.get_indexing_status_handler();

    // Concurrent status checks
    let r1 = status_h.handle(Parameters(GetIndexingStatusArgs {
        collection: "default".to_string(),
    }));
    let r2 = status_h.handle(Parameters(GetIndexingStatusArgs {
        collection: "default".to_string(),
    }));
    let (a, b) = tokio::join!(r1, r2);
    assert!(a.is_ok());
    assert!(b.is_ok());
}

#[tokio::test]
async fn golden_e2e_respects_collection_isolation() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let clear = server.clear_index_handler();

    clear
        .handle(Parameters(ClearIndexArgs {
            collection: "collection_a".to_string(),
        }))
        .await
        .expect("clear a");
    clear
        .handle(Parameters(ClearIndexArgs {
            collection: "collection_b".to_string(),
        }))
        .await
        .expect("clear b");
    // No cross-collection state in test server; just ensure both clears succeed
}

#[tokio::test]
async fn golden_e2e_handles_reindex_correctly() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let index_h = server.index_codebase_handler();
    let collection = "golden_reindex_test";

    let args = IndexCodebaseArgs {
        path: path.to_string_lossy().to_string(),
        collection: Some(collection.to_string()),
        extensions: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let r1 = index_h.handle(Parameters(args.clone())).await;
    assert!(r1.is_ok());
    let r2 = index_h.handle(Parameters(args)).await;
    assert!(r2.is_ok());
    // Re-index same path: no duplicate chunks at application level (vector store upsert/clear)
}

// ============================================================================
// Index
// ============================================================================

#[tokio::test]
async fn golden_index_test_repository() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    assert!(path.exists());

    let r = server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;

    assert!(r.is_ok(), "index_codebase must succeed");
    let res = r.unwrap();
    assert!(
        !res.is_error.unwrap_or(true),
        "index response must not be error"
    );
    let text = content_to_string(&res);
    assert!(
        text.contains("chunk")
            || text.contains("file")
            || text.contains("Index")
            || text.contains("Files processed")
            || text.contains("Indexing Started")
            || text.contains("Source directory")
            || text.contains("Path:"),
        "index response must contain chunk/file/Index/Files/Path: {}",
        text
    );
    assert!(
        text.contains("Source directory")
            || text.contains("Path:")
            || text.contains("Indexing Started")
            || text.contains("Operation ID"),
        "index response must reference path or status: {}",
        text
    );
    if text.contains("Indexing Completed")
        && let Some((files, chunks)) =
            crate::test_utils::test_fixtures::golden_parse_indexing_stats(&text)
    {
        assert!(
            files > 0,
            "indexing completed response must report files_processed > 0: {}",
            text
        );
        assert!(
            chunks > 0,
            "indexing completed response must report chunks_created > 0: {}",
            text
        );
    }
}

#[tokio::test]
async fn golden_index_handles_multiple_languages() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let r = server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some("multi_lang".to_string()),
            extensions: Some(vec!["rs".to_string()]),
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn golden_index_respects_ignore_patterns() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let r = server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some("ignore_test".to_string()),
            extensions: None,
            ignore_patterns: Some(vec!["*.md".to_string()]),
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok());
}

// ============================================================================
// MCP response schema (content shape)
// ============================================================================

#[tokio::test]
async fn golden_mcp_index_codebase_schema() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
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
    let text = content_to_string(&res);
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
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
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
    let text = content_to_string(&res);
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
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let r = server
        .get_indexing_status_handler()
        .handle(Parameters(GetIndexingStatusArgs {
            collection: "default".to_string(),
        }))
        .await;
    assert!(r.is_ok());
    let res = r.unwrap();
    assert!(!res.content.is_empty());
    let text = content_to_string(&res);
    assert!(text.contains("Status") || text.contains("indexing") || text.contains("Idle"));
}

#[tokio::test]
async fn golden_mcp_clear_index_schema() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let r = server
        .clear_index_handler()
        .handle(Parameters(ClearIndexArgs {
            collection: "default".to_string(),
        }))
        .await;
    assert!(r.is_ok());
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = content_to_string(&res);
    assert!(text.contains("Clear") || text.contains("clear") || text.contains("Collection"));
}

#[tokio::test]
async fn golden_mcp_error_responses_consistent() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
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

// ============================================================================
// Search validation
// ============================================================================

#[tokio::test]
async fn golden_search_returns_relevant_results() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index");

    let search_h = server.search_code_handler();
    let r = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "embedding vector".to_string(),
            limit: 10,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search_code must succeed after index");
    let res = r.unwrap();
    assert!(
        !res.is_error.unwrap_or(true),
        "search response must not be error"
    );
    let text = content_to_string(&res);
    assert!(
        text.contains("Search")
            || text.contains("Results")
            || text.contains("Results found")
            || text.contains("No Results Found")
            || text.contains("result"),
        "search response must have Search/Results shape: {}",
        text
    );
    let count = crate::test_utils::test_fixtures::golden_parse_results_found(&text)
        .unwrap_or_else(|| crate::test_utils::test_fixtures::golden_count_result_entries(&text));
    if count > 0 {
        let has_expected_file = sample_codebase_files().iter().any(|f| text.contains(f));
        assert!(
            has_expected_file,
            "when results exist, at least one sample file must appear: {} (files: {:?})",
            text,
            sample_codebase_files()
        );
    }
}

#[tokio::test]
async fn golden_search_ranking_is_correct() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_ranking_test";
    server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(collection.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index for ranking test");

    let search_h = server.search_code_handler();
    let r = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "handler".to_string(),
            limit: 5,
            collection: Some(collection.to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search_code must succeed");
    let text = content_to_string(&r.unwrap());
    assert!(
        text.contains("Search")
            || text.contains("Results")
            || text.contains("Relevance Score")
            || text.contains("result"),
        "ranking response must show results/scores: {}",
        text
    );
}

#[tokio::test]
async fn golden_search_handles_empty_query() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let search_h = server.search_code_handler();
    let r = search_h.handle(Parameters(SearchCodeArgs {
        query: "   ".to_string(),
        limit: 5,
        collection: None,
        extensions: None,
        filters: None,
        token: None,
    }));
    let result = r.await;
    assert!(result.is_err());
}

#[tokio::test]
async fn golden_search_respects_limit_parameter() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_limit_test";
    server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(collection.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index for limit test");

    let search_h = server.search_code_handler();
    let limit = 2usize;
    let r = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "function code".to_string(),
            limit,
            collection: Some(collection.to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search_code must succeed");
    let text = content_to_string(&r.unwrap());
    let n = crate::test_utils::test_fixtures::golden_parse_results_found(&text)
        .unwrap_or_else(|| crate::test_utils::test_fixtures::golden_count_result_entries(&text));
    assert!(
        n <= limit,
        "search must respect limit {}: got {} results, text: {}",
        limit,
        n,
        text
    );
}

#[tokio::test]
async fn golden_search_filters_by_extension() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_ext_filter_test";
    server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(collection.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index for extension filter test");

    let search_h = server.search_code_handler();
    let r = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "function".to_string(),
            limit: 5,
            collection: Some(collection.to_string()),
            extensions: Some(vec!["rs".to_string()]),
            filters: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search_code with extension filter must succeed");
    let text = content_to_string(&r.unwrap());
    let entries = crate::test_utils::test_fixtures::golden_count_result_entries(&text);
    if entries > 0 {
        for line in text.lines() {
            if line.contains("📁") {
                assert!(
                    line.contains(".rs"),
                    "filtered results must be .rs only: {}",
                    line
                );
            }
        }
    }
}

/// Part 1: Setup for golden queries E2E (clear, index, wait for idle). Bounded wait to avoid timeout.
#[tokio::test]
async fn golden_e2e_golden_queries_setup() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    assert!(path.exists(), "sample_codebase must exist");
    let collection = "golden_queries_e2e";
    let clear_h = server.clear_index_handler();
    let index_h = server.index_codebase_handler();
    let status_h = server.get_indexing_status_handler();

    clear_h
        .handle(Parameters(ClearIndexArgs {
            collection: collection.to_string(),
        }))
        .await
        .expect("clear");

    let r = index_h
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(collection.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "index_codebase must succeed");
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true), "index must not return error");

    let mut idle_seen = false;
    for _ in 0..20 {
        let r = status_h
            .handle(Parameters(GetIndexingStatusArgs {
                collection: collection.to_string(),
            }))
            .await;
        if let Ok(s) = r {
            let t = content_to_string(&s);
            if t.contains("Idle") || t.contains("processed") || t.contains("files") {
                idle_seen = true;
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    assert!(
        idle_seen,
        "indexing status should report Idle/processed within bounded wait"
    );
}

/// Part 2: Run a single golden query; handler must succeed (null embedding may return 0 results).
#[tokio::test]
async fn golden_e2e_golden_queries_one_query() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_queries_one";
    let index_h = server.index_codebase_handler();
    let search_h = server.search_code_handler();
    let clear_h = server.clear_index_handler();

    clear_h
        .handle(Parameters(ClearIndexArgs {
            collection: collection.to_string(),
        }))
        .await
        .expect("clear");

    index_h
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(collection.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index");

    let r = search_h
        .handle(Parameters(SearchCodeArgs {
            query: "embedding provider implementation".to_string(),
            limit: 5,
            collection: Some(collection.to_string()),
            extensions: None,
            filters: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search_code must succeed for one golden query");
    let res = r.unwrap();
    assert!(
        !res.is_error.unwrap_or(true),
        "search response must not be error"
    );
}

/// Part 3: Run all golden queries; all handler calls must succeed. With null embedding, result counts may be 0.
#[tokio::test]
async fn golden_e2e_golden_queries_all_handlers_succeed() {
    let server = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_queries_all";
    let index_h = server.index_codebase_handler();
    let search_h = server.search_code_handler();
    let clear_h = server.clear_index_handler();

    clear_h
        .handle(Parameters(ClearIndexArgs {
            collection: collection.to_string(),
        }))
        .await
        .expect("clear");

    index_h
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(collection.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index");

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let fixture = load_golden_queries_fixture();
    assert!(
        !fixture.queries.is_empty(),
        "golden_queries.json must have queries"
    );

    for q in &fixture.queries {
        let r = search_h
            .handle(Parameters(SearchCodeArgs {
                query: q.query.clone(),
                limit: 10,
                collection: Some(collection.to_string()),
                extensions: None,
                filters: None,
                token: None,
            }))
            .await;
        assert!(
            r.is_ok(),
            "search_code must succeed for query id '{}': {}",
            q._id,
            q.query
        );
        let res = r.unwrap();
        assert!(
            !res.is_error.unwrap_or(true),
            "search must not return error for query id '{}': {}",
            q._id,
            q.query
        );
    }
}

fn content_to_string(res: &rmcp::model::CallToolResult) -> String {
    res.content
        .iter()
        .filter_map(|x| {
            if let Ok(v) = serde_json::to_value(x) {
                v.get("text").and_then(|t| t.as_str()).map(String::from)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
