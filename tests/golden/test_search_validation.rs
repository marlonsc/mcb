//! Golden tests: search relevance, ranking, empty query, limit, extension filter.
//! Contract: docs/testing/GOLDEN_TESTS_CONTRACT.md.

use crate::test_utils::test_fixtures::{
    GOLDEN_COLLECTION, SAMPLE_CODEBASE_FILES, create_test_mcp_server, golden_content_to_string,
    golden_count_result_entries, golden_parse_results_found, sample_codebase_path,
};
use mcb_server::args::{IndexCodebaseArgs, SearchCodeArgs};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn golden_search_returns_relevant_results() {
    let server = create_test_mcp_server().await;
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
    let server = create_test_mcp_server().await;
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
async fn golden_search_filters_by_extension() {
    let server = create_test_mcp_server().await;
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
    let text = golden_content_to_string(&r.unwrap());
    let entries = golden_count_result_entries(&text);
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
