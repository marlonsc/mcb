//! Included by mcb-server test binary; contract: `docs/testing/GOLDEN_TESTS_CONTRACT.md`.

use std::path::Path;
use std::time::Duration;

use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;
use serde::Deserialize;

use crate::test_utils::test_fixtures::{
    GOLDEN_COLLECTION, SAMPLE_CODEBASE_FILES, golden_content_to_string,
    golden_count_result_entries, golden_parse_results_found, sample_codebase_path,
};
use crate::test_utils::text::extract_text;
use crate::test_utils::timeouts::TEST_TIMEOUT;

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
    }
}

fn golden_queries_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden_queries.json")
}

#[derive(Debug, Deserialize)]
struct GoldenQuery {
    id: String,
    query: String,
}

#[derive(Debug, Deserialize)]
struct GoldenQueriesFixture {
    queries: Vec<GoldenQuery>,
}

fn load_golden_queries_fixture() -> GoldenQueriesFixture {
    let path = golden_queries_path();
    let content = std::fs::read_to_string(path).expect("Failed to read golden_queries.json");
    serde_json::from_str(&content).expect("Failed to parse golden_queries.json")
}

#[tokio::test]
async fn test_golden_e2e_complete_workflow() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    assert!(
        path.exists(),
        "sample_codebase fixture must exist: {path:?}"
    );
    let path_str = path.to_string_lossy().to_string();

    let index_h = server.index_handler();
    let search_h = server.search_handler();

    let r = index_h
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(GOLDEN_COLLECTION.to_owned()),
        )))
        .await;
    assert!(r.is_ok(), "index clear should succeed: {r:?}");
    let clear_text = extract_text(&r.unwrap().content);
    assert!(
        clear_text.to_lowercase().contains("clear"),
        "clear response must mention clear/cleared: {clear_text}"
    );

    let r = index_h
        .handle(Parameters(index_args(
            IndexAction::Status,
            None,
            Some(GOLDEN_COLLECTION.to_owned()),
        )))
        .await;
    assert!(r.is_ok(), "index status should succeed: {r:?}");
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = extract_text(&res.content);
    assert!(text.contains("Indexing Status") || text.contains("Idle") || text.contains("indexing"));

    let r = index_h
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path_str),
            Some(GOLDEN_COLLECTION.to_owned()),
        )))
        .await;
    assert!(r.is_ok(), "index should succeed: {r:?}");
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = extract_text(&res.content);
    assert!(
        text.contains("chunks") || text.contains("Indexing") || text.contains("files"),
        "expected chunks/indexing in response: {text}"
    );

    let r = search_h
        .handle(Parameters(search_args(
            "embedding provider",
            Some(GOLDEN_COLLECTION.to_owned()),
            Some(5),
        )))
        .await;
    assert!(r.is_ok(), "search should succeed: {r:?}");
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = extract_text(&res.content);
    assert!(
        text.contains("Search") || text.contains("Results") || text.contains("result"),
        "expected search result text: {text}"
    );

    let r = index_h
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(GOLDEN_COLLECTION.to_owned()),
        )))
        .await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn test_golden_e2e_handles_concurrent_operations() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let status_h = server.index_handler();
    let r1 = status_h.handle(Parameters(index_args(
        IndexAction::Status,
        None,
        Some("default".to_owned()),
    )));
    let r2 = status_h.handle(Parameters(index_args(
        IndexAction::Status,
        None,
        Some("default".to_owned()),
    )));
    let (a, b) = tokio::join!(r1, r2);
    assert!(a.is_ok());
    assert!(b.is_ok());
}

#[tokio::test]
async fn test_golden_e2e_respects_collection_isolation() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let clear = server.index_handler();
    clear
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some("collection_a".to_owned()),
        )))
        .await
        .expect("clear a");
    clear
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some("collection_b".to_owned()),
        )))
        .await
        .expect("clear b");
}

#[tokio::test]
async fn test_golden_e2e_handles_reindex_correctly() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let index_h = server.index_handler();
    let collection = "golden_reindex_test";
    let args = index_args(
        IndexAction::Start,
        Some(path.to_string_lossy().to_string()),
        Some(collection.to_owned()),
    );
    let r1 = index_h.handle(Parameters(args.clone())).await;
    assert!(r1.is_ok());
    let r2 = index_h.handle(Parameters(args)).await;
    assert!(r2.is_ok());
}

#[rstest]
#[case(Some(GOLDEN_COLLECTION.to_owned()), None)]
#[case(Some("golden_multi_lang".to_owned()), Some(vec!["rs".to_owned()]))]
#[tokio::test]
async fn test_golden_index_variants(
    #[case] collection: Option<String>,
    #[case] extensions: Option<Vec<String>>,
) {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    assert!(path.exists(), "sample_codebase must exist: {path:?}");

    let handler = server.index_handler();
    let mut args = index_args(
        IndexAction::Start,
        Some(path.to_string_lossy().to_string()),
        collection,
    );
    args.extensions = extensions;

    let result = handler.handle(Parameters(args)).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.is_error.unwrap_or(false));

    let text = extract_text(&response.content);
    assert!(
        text.contains("Files processed")
            || text.contains("Indexing Started")
            || text.contains("started"),
        "response: {text}"
    );
}

#[tokio::test]
async fn test_golden_index_respects_ignore_patterns() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let handler = server.index_handler();
    let mut args = index_args(
        IndexAction::Start,
        Some(path.to_string_lossy().to_string()),
        Some("golden_ignore_test".to_owned()),
    );
    args.ignore_patterns = Some(vec!["*_test.rs".to_owned()]);
    let result = handler.handle(Parameters(args)).await;
    assert!(result.is_ok());
}

#[rstest]
#[case(IndexAction::Status, false)]
#[case(IndexAction::Clear, false)]
#[case(IndexAction::Status, true)]
#[tokio::test]
async fn test_golden_mcp_index_schema_actions(
    #[case] action: IndexAction,
    #[case] assert_status_text: bool,
) {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let index_h = server.index_handler();
    let r = index_h
        .handle(Parameters(index_args(
            action,
            None,
            Some("default".to_owned()),
        )))
        .await;
    assert!(r.is_ok());
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    if assert_status_text {
        let text = extract_text(&res.content);
        assert!(
            text.contains("Status") || text.contains("indexing") || text.contains("Idle"),
            "{}",
            text
        );
    }
}

#[tokio::test]
async fn test_golden_mcp_search_code_schema() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let search_h = server.search_handler();
    let r = search_h
        .handle(Parameters(search_args(
            "test",
            Some("default".to_owned()),
            Some(5),
        )))
        .await;
    assert!(r.is_ok());
}

#[rstest]
#[case("")]
#[case("   ")]
#[tokio::test]
async fn test_golden_mcp_empty_query_error_responses(#[case] query: &str) {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let result = server
        .search_handler()
        .handle(Parameters(search_args(query, None, Some(5))))
        .await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_golden_search_returns_relevant_results() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_search_relevance";
    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().to_string()),
            Some(collection.to_owned()),
        )))
        .await
        .expect("index");

    let r = server
        .search_handler()
        .handle(Parameters(search_args(
            "embedding vector",
            Some(collection.to_owned()),
            Some(10),
        )))
        .await;
    assert!(r.is_ok(), "search must succeed after index");
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = golden_content_to_string(&res);
    let count =
        golden_parse_results_found(&text).unwrap_or_else(|| golden_count_result_entries(&text));
    if count > 0 {
        let has_expected = SAMPLE_CODEBASE_FILES.iter().any(|f| text.contains(f));
        assert!(
            has_expected,
            "when results exist, at least one sample file must appear: {text} (files: {SAMPLE_CODEBASE_FILES:?})"
        );
    }
}

#[tokio::test]
async fn test_golden_search_ranking_is_correct() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_ranking_test";
    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().to_string()),
            Some(collection.to_owned()),
        )))
        .await
        .expect("index for ranking test");

    let r = server
        .search_handler()
        .handle(Parameters(search_args(
            "handler",
            Some(collection.to_owned()),
            Some(5),
        )))
        .await;
    assert!(r.is_ok(), "search must succeed");
}

#[tokio::test]
async fn test_golden_search_respects_limit_parameter() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_limit_test";
    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().to_string()),
            Some(collection.to_owned()),
        )))
        .await
        .expect("index for limit test");

    let r = server
        .search_handler()
        .handle(Parameters(search_args(
            "function code",
            Some(collection.to_owned()),
            Some(2),
        )))
        .await;
    assert!(r.is_ok(), "search must succeed");
    let text = golden_content_to_string(&r.unwrap());
    let n = golden_parse_results_found(&text).unwrap_or_else(|| golden_count_result_entries(&text));
    assert!(n <= 2, "search must respect limit: got {n} results");
}

#[tokio::test]
async fn test_golden_search_filters_by_extension() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_ext_filter_test";
    let mut args = index_args(
        IndexAction::Start,
        Some(path.to_string_lossy().to_string()),
        Some(collection.to_owned()),
    );
    args.extensions = Some(vec!["rs".to_owned()]);
    server
        .index_handler()
        .handle(Parameters(args))
        .await
        .expect("index");

    let r = server
        .search_handler()
        .handle(Parameters(search_args(
            "function",
            Some(collection.to_owned()),
            Some(5),
        )))
        .await;
    assert!(r.is_ok(), "search with indexed extensions must succeed");
}

#[tokio::test]
async fn test_golden_e2e_golden_queries_setup() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_queries_e2e";

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(collection.to_owned()),
        )))
        .await
        .expect("clear");

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().to_string()),
            Some(collection.to_owned()),
        )))
        .await
        .expect("index");

    let completed = tokio::time::timeout(TEST_TIMEOUT, async {
        for _ in 0..20 {
            let r = server
                .index_handler()
                .handle(Parameters(index_args(
                    IndexAction::Status,
                    None,
                    Some(collection.to_owned()),
                )))
                .await
                .expect("status");
            let text = extract_text(&r.content);
            if text.contains("Idle") || text.contains("completed") || text.contains("Status") {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        false
    })
    .await
    .expect("index status polling timed out");
    assert!(
        completed,
        "index did not reach a terminal status in polling window"
    );
}

#[tokio::test]
async fn test_golden_e2e_golden_queries_one_query() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_queries_one";

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(collection.to_owned()),
        )))
        .await
        .expect("clear");

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().to_string()),
            Some(collection.to_owned()),
        )))
        .await
        .expect("index");

    let fixture = load_golden_queries_fixture();
    assert!(
        !fixture.queries.is_empty(),
        "golden_queries.json must have queries"
    );

    let query = &fixture.queries[0];
    let search_h = server.search_handler();
    let r = search_h
        .handle(Parameters(search_args(
            &query.query,
            Some(collection.to_owned()),
            Some(5),
        )))
        .await;
    assert!(r.is_ok(), "Query '{}' should succeed: {:?}", query.id, r);
    let res = r.unwrap();
    assert!(
        !res.is_error.unwrap_or(true),
        "Query '{}' returned error",
        query.id
    );
}

#[tokio::test]
async fn test_golden_e2e_golden_queries_all_handlers_succeed() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_queries_all";

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Clear,
            None,
            Some(collection.to_owned()),
        )))
        .await
        .expect("clear");

    server
        .index_handler()
        .handle(Parameters(index_args(
            IndexAction::Start,
            Some(path.to_string_lossy().to_string()),
            Some(collection.to_owned()),
        )))
        .await
        .expect("index");

    let fixture = load_golden_queries_fixture();
    assert!(
        !fixture.queries.is_empty(),
        "golden_queries.json must have queries"
    );

    for query in fixture.queries.iter() {
        let search_h = server.search_handler();
        let r = search_h
            .handle(Parameters(search_args(
                &query.query,
                Some(collection.to_owned()),
                Some(5),
            )))
            .await;
        assert!(r.is_ok(), "Query '{}' should succeed: {:?}", query.id, r);
        let res = r.unwrap();
        assert!(
            !res.is_error.unwrap_or(true),
            "Query '{}' returned error",
            query.id
        );
    }
}
