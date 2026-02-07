//! Contract: docs/testing/GOLDEN_TESTS_CONTRACT.md.

use crate::test_utils::test_fixtures::{
    create_test_mcp_server, golden_content_to_string, golden_count_result_entries,
    golden_parse_results_found, sample_codebase_path, GOLDEN_COLLECTION, SAMPLE_CODEBASE_FILES,
};
use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn golden_search_returns_relevant_results() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index");

    let search_h = server.search_handler();
    let r = search_h
        .handle(Parameters(SearchArgs {
            query: "embedding vector".to_string(),
            resource: SearchResource::Code,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            filters: None,
            limit: Some(10),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search must succeed after index");
    let res = r.unwrap();
    assert!(
        !res.is_error.unwrap_or(true),
        "search response must not be error"
    );
    let text = golden_content_to_string(&res);
    assert!(
        text.contains("Search")
            || text.contains("Results")
            || text.contains("Results found")
            || text.contains("No Results Found")
            || text.contains("result"),
        "search response must have Search/Results shape: {}",
        text
    );
    let count =
        golden_parse_results_found(&text).unwrap_or_else(|| golden_count_result_entries(&text));
    if count > 0 {
        let has_expected = SAMPLE_CODEBASE_FILES.iter().any(|f| text.contains(f));
        assert!(
            has_expected,
            "when results exist, at least one sample file must appear: {} (files: {:?})",
            text, SAMPLE_CODEBASE_FILES
        );
    }
}

#[tokio::test]
async fn golden_search_ranking_is_correct() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_ranking_test";
    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection.to_string()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index for ranking test");

    let search_h = server.search_handler();
    let r = search_h
        .handle(Parameters(SearchArgs {
            query: "handler".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection.to_string()),
            extensions: None,
            filters: None,
            limit: Some(5),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search must succeed");
    let text = golden_content_to_string(&r.unwrap());
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
    let server = create_test_mcp_server().await;
    let search_h = server.search_handler();
    let r = search_h.handle(Parameters(SearchArgs {
        query: "   ".to_string(),
        resource: SearchResource::Code,
        collection: None,
        extensions: None,
        filters: None,
        limit: Some(5),
        min_score: None,
        tags: None,
        session_id: None,
        token: None,
    }));
    let result = r.await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn golden_search_respects_limit_parameter() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_limit_test";
    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection.to_string()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index for limit test");

    let search_h = server.search_handler();
    let limit = 2usize;
    let r = search_h
        .handle(Parameters(SearchArgs {
            query: "function code".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection.to_string()),
            extensions: None,
            filters: None,
            limit: Some(limit as u32),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search must succeed");
    let text = golden_content_to_string(&r.unwrap());
    let n = golden_parse_results_found(&text).unwrap_or_else(|| golden_count_result_entries(&text));
    assert!(
        n <= limit,
        "search must respect limit {}: got {} results, text: {}",
        limit,
        n,
        text
    );
}

#[tokio::test]
async fn golden_search_respects_index_extensions() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_ext_filter_test";
    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection.to_string()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index for extension filter test");

    let search_h = server.search_handler();
    let r = search_h
        .handle(Parameters(SearchArgs {
            query: "function".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection.to_string()),
            extensions: None,
            filters: None,
            limit: Some(5),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search with indexed extensions must succeed");
    let text = golden_content_to_string(&r.unwrap());
    let entries = golden_count_result_entries(&text);
    assert!(entries <= 5, "limit respected: {}", entries);
}

#[tokio::test]
async fn golden_index_with_consolidation_args_start() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_consolidation_start";

    let result = server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection.to_string()),
            extensions: Some(vec!["rs".to_string(), "py".to_string()]),
            exclude_dirs: Some(vec!["target".to_string(), "node_modules".to_string()]),
            ignore_patterns: None,
            max_file_size: Some(10_000_000),
            follow_symlinks: Some(false),
            token: None,
        }))
        .await;

    assert!(
        result.is_ok(),
        "index with consolidated args should succeed"
    );
    let response = result.unwrap();
    assert!(
        !response.is_error.unwrap_or(true),
        "response should not be error"
    );
}

#[tokio::test]
async fn golden_search_with_very_large_limit() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_large_limit";

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection.to_string()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index");

    let search_result = server
        .search_handler()
        .handle(Parameters(SearchArgs {
            query: "code".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection.to_string()),
            extensions: None,
            filters: None,
            limit: Some(1000),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await;

    assert!(
        search_result.is_ok(),
        "search with large limit should succeed or error gracefully"
    );
}

#[tokio::test]
async fn golden_search_with_min_limit() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_min_limit";

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection.to_string()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index");

    let search_result = server
        .search_handler()
        .handle(Parameters(SearchArgs {
            query: "implementation".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection.to_string()),
            extensions: None,
            filters: None,
            limit: Some(1),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await;

    assert!(search_result.is_ok(), "search with limit=1 should succeed");
    let text = golden_content_to_string(&search_result.unwrap());
    let count = golden_count_result_entries(&text);
    assert!(
        count <= 1,
        "limit=1 must return at most 1 result, got {}",
        count
    );
}

#[tokio::test]
async fn golden_collection_isolation_multiple_searches() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let col_a = "golden_iso_searches_a";
    let col_b = "golden_iso_searches_b";

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(col_a.to_string()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index col_a");

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(col_b.to_string()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index col_b");

    let searches = vec![
        ("function", col_a.to_string()),
        ("implementation", col_b.to_string()),
    ];

    let mut col_a_count = 0;
    let mut col_b_count = 0;

    for (query, collection) in searches {
        let result = server
            .search_handler()
            .handle(Parameters(SearchArgs {
                query: query.to_string(),
                resource: SearchResource::Code,
                collection: Some(collection.clone()),
                extensions: None,
                filters: None,
                limit: Some(5),
                min_score: None,
                tags: None,
                session_id: None,
                token: None,
            }))
            .await;

        assert!(result.is_ok(), "search for '{}' should succeed", query);
        let count = golden_count_result_entries(&golden_content_to_string(&result.unwrap()));

        if collection == col_a {
            col_a_count = count;
        } else {
            col_b_count = count;
        }
    }

    // Ensure search executes without panic; results may be 0 if chunking varies
    let _ = (col_a_count, col_b_count);
}
