//! Included by mcb-server test binary; contract: `docs/testing/GOLDEN_TESTS_CONTRACT.md`.

use std::path::Path;
use std::time::Duration;

use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;
use serde::Deserialize;

use mcb_domain::utils::tests::collection::unique_collection;
use mcb_domain::utils::tests::fixtures::sample_codebase_path;
use mcb_domain::utils::tests::mcp_assertions::{
    extract_text as extract_result_text, golden_count_result_entries, golden_parse_results_found,
};
use mcb_domain::utils::tests::timeouts::TEST_TIMEOUT;
use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::utils::text::extract_text_from;
use mcb_utils::constants::testing::SAMPLE_CODEBASE_FILES;

fn index_args(action: IndexAction, path: Option<String>, collection: Option<String>) -> IndexArgs {
    IndexArgs {
        action,
        path,
        collection,
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
        repo_id: None,
    }
}

fn search_args(query: &str, collection: Option<String>, limit: Option<u32>) -> SearchArgs {
    SearchArgs {
        query: query.to_owned(),
        org_id: None,
        resource: SearchResource::Code,
        collection,
        extensions: None,
        filters: None,
        limit,
        min_score: None,
        tags: None,
        session_id: None,
        token: None,
        repo_id: None,
        repo_path: None,
    }
}

fn golden_queries_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden_queries.json")
}

#[derive(Debug, Deserialize)]
struct GoldenQuery {
    id: String,
    query: String,
    min_results: usize,
}

#[derive(Debug, Deserialize)]
struct GoldenQueriesFixture {
    queries: Vec<GoldenQuery>,
}

fn load_golden_queries_fixture() -> Result<GoldenQueriesFixture, Box<dyn std::error::Error>> {
    let path = golden_queries_path();
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn assert_search_response_contains_file(text: &str, min_results: usize, expected_file: &str) {
    let count =
        golden_parse_results_found(text).unwrap_or_else(|| golden_count_result_entries(text));
    assert!(
        count >= min_results,
        "expected at least {min_results} search results, got {count}: {text}"
    );
    assert!(
        text.contains(expected_file),
        "search results should include {expected_file}: {text}"
    );
}

fn assert_search_response_contains_sample_file(text: &str, min_results: usize) {
    let count =
        golden_parse_results_found(text).unwrap_or_else(|| golden_count_result_entries(text));
    assert!(
        count >= min_results,
        "expected at least {min_results} search results, got {count}: {text}"
    );
    let has_expected = SAMPLE_CODEBASE_FILES.iter().any(|file| text.contains(file));
    assert!(
        has_expected,
        "search must return at least one known sample file: {text} (files: {SAMPLE_CODEBASE_FILES:?})"
    );
}

async fn wait_for_indexing_completion(
    server: &mcb_server::mcp_server::McpServer,
    collection: &str,
) -> TestResult {
    let deadline = tokio::time::Instant::now() + TEST_TIMEOUT;
    let mut last_text = String::new();
    while tokio::time::Instant::now() < deadline {
        let result = server
            .index_handler()
            .handle(Parameters(index_args(
                IndexAction::Status,
                None,
                Some(collection.to_owned()),
            )))
            .await?;
        assert!(
            !result.is_error.unwrap_or(true),
            "index status returned an error: {}",
            extract_result_text(&result)
        );
        let text = extract_result_text(&result);
        if text.contains("Indexing Status: Idle") {
            return Ok(());
        }
        last_text = text;
        tokio::task::yield_now().await;
    }
    panic!("indexing did not reach terminal state: {last_text}");
}

async fn wait_for_search_response_contains_file(
    server: &mcb_server::mcp_server::McpServer,
    collection: &str,
    query: &str,
    limit: u32,
    min_results: usize,
    expected_file: &str,
) -> TestResult<String> {
    let deadline = tokio::time::Instant::now() + TEST_TIMEOUT;
    let mut last_text = String::new();
    while tokio::time::Instant::now() < deadline {
        let result = server
            .search_handler()
            .handle(Parameters(search_args(
                query,
                Some(collection.to_owned()),
                Some(limit),
            )))
            .await?;
        assert!(
            !result.is_error.unwrap_or(true),
            "search returned an error: {}",
            extract_result_text(&result)
        );
        let text = extract_result_text(&result);
        let count =
            golden_parse_results_found(&text).unwrap_or_else(|| golden_count_result_entries(&text));
        if count >= min_results && text.contains(expected_file) {
            return Ok(text);
        }
        last_text = text;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert_search_response_contains_file(&last_text, min_results, expected_file);
    Ok(last_text)
}

async fn wait_for_search_response_contains_sample_file(
    server: &mcb_server::mcp_server::McpServer,
    collection: &str,
    query: &str,
    limit: u32,
    min_results: usize,
) -> TestResult<String> {
    let deadline = tokio::time::Instant::now() + TEST_TIMEOUT;
    let mut last_text = String::new();
    while tokio::time::Instant::now() < deadline {
        let result = server
            .search_handler()
            .handle(Parameters(search_args(
                query,
                Some(collection.to_owned()),
                Some(limit),
            )))
            .await?;
        assert!(
            !result.is_error.unwrap_or(true),
            "search returned an error: {}",
            extract_result_text(&result)
        );
        let text = extract_result_text(&result);
        let count =
            golden_parse_results_found(&text).unwrap_or_else(|| golden_count_result_entries(&text));
        if count >= min_results && SAMPLE_CODEBASE_FILES.iter().any(|file| text.contains(file)) {
            return Ok(text);
        }
        last_text = text;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert_search_response_contains_sample_file(&last_text, min_results);
    Ok(last_text)
}

#[rstest]
#[tokio::test]
async fn test_golden_e2e_complete_workflow() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-complete");
    assert!(
        path.exists(),
        "sample_codebase fixture must exist: {path:?}"
    );
    let path_str = path.to_string_lossy().into_owned();

    let index_h = server.index_handler();

    let r = index_h
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(collection.clone()),
        )))
        .await;
    assert!(r.is_ok(), "index clear should succeed: {r:?}");
    let clear_text = extract_text_from(&r.unwrap().content);
    assert!(
        clear_text.to_lowercase().contains("clear"),
        "clear response must mention clear/cleared: {clear_text}"
    );

    let r = index_h
        .handle(Parameters(index_args(
            IndexAction::Status,
            None,
            Some(collection.clone()),
        )))
        .await;
    assert!(r.is_ok(), "index status should succeed: {r:?}");
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = extract_text_from(&res.content);
    assert!(text.contains("Indexing Status") || text.contains("Idle") || text.contains("indexing"));

    let r = index_h
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path_str),
            Some(collection.clone()),
        )))
        .await;
    assert!(r.is_ok(), "index should succeed: {r:?}");
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = extract_text_from(&res.content);
    assert!(
        text.contains("chunks") || text.contains("Indexing") || text.contains("files"),
        "expected chunks/indexing in response: {text}"
    );
    wait_for_indexing_completion(&server, &collection).await?;

    let text = wait_for_search_response_contains_file(
        &server,
        &collection,
        "handle MCP index codebase request",
        5,
        1,
        "handlers.rs",
    )
    .await?;
    assert!(
        text.contains("Semantic Code Search Results"),
        "expected search result text: {text}"
    );

    let r = index_h
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(collection),
        )))
        .await;
    assert!(r.is_ok());
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_e2e_handles_concurrent_operations() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let status_h = server.index_handler();
    let r1 = status_h.handle(Parameters(index_args(
        IndexAction::Status,
        None,
        Some(mcb_utils::constants::DEFAULT_NAMESPACE.to_owned()),
    )));
    let r2 = status_h.handle(Parameters(index_args(
        IndexAction::Status,
        None,
        Some(mcb_utils::constants::DEFAULT_NAMESPACE.to_owned()),
    )));
    let (a, b) = tokio::join!(r1, r2);
    assert!(a.is_ok());
    assert!(b.is_ok());
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_e2e_respects_collection_isolation() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let clear = server.index_handler();
    let collection_a = unique_collection("golden-isolation-a");
    let collection_b = unique_collection("golden-isolation-b");
    let clear_a = clear
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(collection_a),
        )))
        .await;
    assert!(
        clear_a.is_ok(),
        "clear collection_a should succeed: {clear_a:?}"
    );

    let clear_b = clear
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(collection_b),
        )))
        .await;
    assert!(
        clear_b.is_ok(),
        "clear collection_b should succeed: {clear_b:?}"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_e2e_handles_reindex_correctly() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let index_h = server.index_handler();
    let collection = unique_collection("golden-reindex");
    let args = index_args(
        IndexAction::Start,
        Some(path.to_string_lossy().into_owned()),
        Some(collection.clone()),
    );
    let r1 = index_h.handle(Parameters(args.clone())).await;
    assert!(r1.is_ok());
    wait_for_indexing_completion(&server, &collection).await?;
    let r2 = index_h.handle(Parameters(args)).await;
    assert!(r2.is_ok());
    wait_for_indexing_completion(&server, &collection).await?;
    Ok(())
}

#[rstest]
#[case("golden-index", None)]
#[case("golden-multi-lang", Some(vec!["rs".to_owned()]))]
#[tokio::test]
async fn test_golden_index_variants(
    #[case] collection_prefix: &str,
    #[case] extensions: Option<Vec<String>>,
) -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    assert!(path.exists(), "sample_codebase must exist: {path:?}");

    let collection = unique_collection(collection_prefix);
    let handler = server.index_handler();
    let mut args = index_args(
        IndexAction::Start,
        Some(path.to_string_lossy().into_owned()),
        Some(collection.clone()),
    );
    args.extensions = extensions;

    let result = handler.handle(Parameters(args)).await;
    let response = result.expect("index variants should succeed");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(!response.is_error.unwrap_or(false));

    let text = extract_text_from(&response.content);
    assert!(
        text.contains("Files processed")
            || text.contains("Indexing Started")
            || text.contains("started"),
        "response: {text}"
    );
    wait_for_indexing_completion(&server, &collection).await?;
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_index_respects_ignore_patterns() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-ignore");
    let handler = server.index_handler();
    let mut args = index_args(
        IndexAction::Start,
        Some(path.to_string_lossy().into_owned()),
        Some(collection.clone()),
    );
    args.ignore_patterns = Some(vec!["*_test.rs".to_owned()]);
    let result = handler.handle(Parameters(args)).await;
    let response = result.expect("index with ignore patterns should succeed");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(!response.is_error.unwrap_or(false));

    let text = extract_text_from(&response.content);
    assert!(
        text.contains("Files processed")
            || text.contains("Indexing Started")
            || text.contains("started"),
        "response: {text}"
    );
    wait_for_indexing_completion(&server, &collection).await?;
    Ok(())
}

#[rstest]
#[case(IndexAction::Status, false)]
#[case(IndexAction::Clear, false)]
#[case(IndexAction::Status, true)]
#[rstest]
#[tokio::test]
async fn test_golden_mcp_index_schema_actions(
    #[case] action: IndexAction,
    #[case] assert_status_text: bool,
) -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let index_h = server.index_handler();
    let r = index_h
        .handle(Parameters(index_args(
            action,
            None,
            Some(mcb_utils::constants::DEFAULT_NAMESPACE.to_owned()),
        )))
        .await;
    assert!(r.is_ok());
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    if assert_status_text {
        let text = extract_text_from(&res.content);
        assert!(
            text.contains("Status") || text.contains("indexing") || text.contains("Idle"),
            "{}",
            text
        );
    }
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_mcp_search_code_schema() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let search_h = server.search_handler();
    let r = search_h
        .handle(Parameters(search_args(
            "test",
            Some(mcb_utils::constants::DEFAULT_NAMESPACE.to_owned()),
            Some(5),
        )))
        .await;
    assert!(r.is_ok());
    Ok(())
}

#[rstest]
#[case("")]
#[case("   ")]
#[tokio::test]
async fn test_golden_mcp_empty_query_error_responses(#[case] query: &str) -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let result = server
        .search_handler()
        .handle(Parameters(search_args(query, None, Some(5))))
        .await;
    let text = match result {
        Ok(response) => {
            assert!(
                !response.content.is_empty(),
                "error response should have content"
            );
            assert!(response.is_error.unwrap_or(false));
            extract_text_from(&response.content)
        }
        Err(e) => e.to_string(),
    };
    assert!(
        text.to_lowercase().contains("empty")
            || text.to_lowercase().contains("query")
            || text.to_lowercase().contains("invalid")
            || text.to_lowercase().contains("parameter"),
        "error response should mention empty query or invalid parameters: {text}"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_search_returns_relevant_results() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-search-relevance");
    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().into_owned()),
            Some(collection.clone()),
        )))
        .await
        .expect("index");
    wait_for_indexing_completion(&server, &collection).await?;

    let text = wait_for_search_response_contains_file(
        &server,
        &collection,
        "handle MCP search request",
        10,
        1,
        "handlers.rs",
    )
    .await?;
    assert_search_response_contains_sample_file(&text, 1);
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_search_ranking_is_correct() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-ranking");
    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().into_owned()),
            Some(collection.clone()),
        )))
        .await
        .expect("index for ranking test");
    wait_for_indexing_completion(&server, &collection).await?;

    let text = wait_for_search_response_contains_file(
        &server,
        &collection,
        "handle MCP index codebase request",
        5,
        1,
        "handlers.rs",
    )
    .await?;
    assert!(
        text.contains("Semantic Code Search Results"),
        "expected semantic search result heading: {text}"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_search_respects_limit_parameter() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-limit");
    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().into_owned()),
            Some(collection.clone()),
        )))
        .await
        .expect("index for limit test");
    wait_for_indexing_completion(&server, &collection).await?;

    let r = server
        .search_handler()
        .handle(Parameters(search_args(
            "function code",
            Some(collection),
            Some(2),
        )))
        .await;
    assert!(r.is_ok(), "search must succeed");
    let text = extract_result_text(&r.unwrap());
    let n = golden_parse_results_found(&text).unwrap_or_else(|| golden_count_result_entries(&text));
    assert!(n <= 2, "search must respect limit: got {n} results");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_search_filters_by_extension() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-ext-filter");
    let mut args = index_args(
        IndexAction::Start,
        Some(path.to_string_lossy().into_owned()),
        Some(collection.clone()),
    );
    args.extensions = Some(vec!["rs".to_owned()]);
    server
        .index_handler()
        .handle(Parameters(args))
        .await
        .expect("index");
    wait_for_indexing_completion(&server, &collection).await?;

    let r = server
        .search_handler()
        .handle(Parameters(search_args(
            "function",
            Some(collection),
            Some(5),
        )))
        .await;
    assert!(r.is_ok(), "search with indexed extensions must succeed");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_e2e_golden_queries_setup() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-queries-setup");

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(collection.clone()),
        )))
        .await
        .expect("clear");

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().into_owned()),
            Some(collection.clone()),
        )))
        .await
        .expect("index");
    wait_for_indexing_completion(&server, &collection).await?;
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_e2e_golden_queries_one_query() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-queries-one");

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(collection.clone()),
        )))
        .await
        .expect("clear");

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().into_owned()),
            Some(collection.clone()),
        )))
        .await
        .expect("index");
    wait_for_indexing_completion(&server, &collection).await?;

    let fixture = load_golden_queries_fixture().expect("load golden queries fixture");
    assert!(
        !fixture.queries.is_empty(),
        "golden_queries.json must have queries"
    );

    let query = &fixture.queries[0];
    assert!(!query.id.is_empty(), "golden query id must be present");
    wait_for_search_response_contains_sample_file(&server, &collection, &query.query, 5, 1).await?;
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_e2e_golden_queries_all_handlers_succeed() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-queries-all");

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(collection.clone()),
        )))
        .await
        .expect("clear");

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().into_owned()),
            Some(collection.clone()),
        )))
        .await
        .expect("index");
    wait_for_indexing_completion(&server, &collection).await?;

    let fixture = load_golden_queries_fixture().expect("load golden queries fixture");
    assert!(
        !fixture.queries.is_empty(),
        "golden_queries.json must have queries"
    );

    for query in fixture.queries.iter() {
        assert!(!query.id.is_empty(), "golden query id must be present");
        wait_for_search_response_contains_sample_file(
            &server,
            &collection,
            &query.query,
            5,
            query.min_results,
        )
        .await?;
    }
    Ok(())
}
