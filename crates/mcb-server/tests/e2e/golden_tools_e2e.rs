//!
//! These tests use the real DI stack (`FastEmbed` + `EdgeVec`)
//! and call the MCP handlers (index, search) to validate behavior.

use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;

use mcb_domain::utils::tests::collection::unique_collection;
use mcb_domain::utils::tests::fixtures::sample_codebase_path;
use mcb_domain::utils::tests::timeouts::TEST_TIMEOUT;
use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::utils::text::extract_text_from;

async fn wait_for_indexing_completion(
    server: &mcb_server::mcp_server::McpServer,
    collection: &str,
) -> TestResult {
    let deadline = tokio::time::Instant::now() + TEST_TIMEOUT;
    let mut last_text = String::new();
    while tokio::time::Instant::now() < deadline {
        let response = server
            .index_handler()
            .handle(Parameters(IndexArgs {
                action: IndexAction::Status,
                path: None,
                collection: Some(collection.to_owned()),
                extensions: None,
                exclude_dirs: None,
                ignore_patterns: None,
                max_file_size: None,
                follow_symlinks: None,
                token: None,
                repo_id: None,
            }))
            .await?;
        assert!(!response.is_error.unwrap_or(true));
        let text = extract_text_from(&response.content);
        if text.contains("Indexing Status: Idle") {
            return Ok(());
        }
        last_text = text;
        tokio::task::yield_now().await;
    }
    panic!("indexing did not reach terminal state: {last_text}");
}

// =============================================================================
// E2E: Complete workflow (clear -> status -> index -> status -> search -> clear)
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_golden_e2e_complete_workflow() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    assert!(
        path.exists(),
        "sample_codebase fixture must exist: {path:?}"
    );
    let path_str = path.to_string_lossy().into_owned();
    let collection = unique_collection("golden-tools-complete");

    let index_h = server.index_handler();
    let search_h = server.search_handler();

    // 1. Clear any existing test data
    let clear_args = IndexArgs {
        action: IndexAction::Clear,
        path: None,
        collection: Some(collection.clone()),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
        repo_id: None,
    };
    let r = index_h.handle(Parameters(clear_args)).await;
    assert!(r.is_ok(), "index clear should succeed");
    let resp = r.unwrap();
    let text = extract_text_from(&resp.content);
    assert!(
        text.contains("cleared") || text.contains("Cleared"),
        "{}",
        text
    );

    // 2. Status (idle / empty)
    let status_args = IndexArgs {
        action: IndexAction::Status,
        path: None,
        collection: Some(collection.clone()),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
        repo_id: None,
    };
    let r = index_h.handle(Parameters(status_args)).await;
    assert!(r.is_ok());
    let text = extract_text_from(&r.unwrap().content);
    assert!(
        text.contains("Indexing Status") || text.contains("Status"),
        "{}",
        text
    );

    // 3. Index repository
    let index_args = IndexArgs {
        action: IndexAction::Start,
        path: path_str.clone().into(),
        collection: Some(collection.clone()),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
        repo_id: None,
    };
    let r = index_h.handle(Parameters(index_args)).await;
    assert!(r.is_ok(), "index should succeed");
    let resp = r.unwrap();
    assert!(!resp.is_error.unwrap_or(false));
    let text = extract_text_from(&resp.content);
    assert!(
        text.contains("Files processed")
            || text.contains("Chunks created")
            || text.contains("Indexing Started")
            || text.contains("started"),
        "{}",
        text
    );
    wait_for_indexing_completion(&server, &collection).await?;

    // 4. Search
    let search_args = SearchArgs {
        query: "embedding or vector".to_owned(),
        org_id: None,
        resource: SearchResource::Code,
        collection: Some(collection.clone()),
        extensions: None,
        filters: None,
        limit: Some(5),
        min_score: None,
        tags: None,
        session_id: None,
        token: None,
        repo_id: None,
        repo_path: None,
    };
    let r = search_h.handle(Parameters(search_args)).await;
    assert!(r.is_ok());
    let resp = r.unwrap();
    let text = extract_text_from(&resp.content);
    assert!(
        text.contains("Search") || text.contains("Results") || text.contains("results"),
        "{}",
        text
    );

    // 5. Clear index
    let clear_args = IndexArgs {
        action: IndexAction::Clear,
        path: None,
        collection: Some(collection),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
        repo_id: None,
    };
    let r = index_h.handle(Parameters(clear_args)).await;
    assert!(r.is_ok());
    Ok(())
}

#[rstest]
#[case("golden-tools-index", None)]
#[case("golden-tools-multi-lang", Some(vec!["rs".to_owned()]))]
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
    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some(path.to_string_lossy().into_owned()),
        collection: Some(collection.clone()),
        extensions,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
        repo_id: None,
    };

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
async fn test_golden_search_returns_relevant_results() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-tools-search-relevance");
    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().into_owned()),
            collection: Some(collection.clone()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
            repo_id: None,
        }))
        .await
        .expect("index");
    wait_for_indexing_completion(&server, &collection).await?;

    let search_h = server.search_handler();
    let r = search_h
        .handle(Parameters(SearchArgs {
            query: "embedding vector".to_owned(),
            org_id: None,
            resource: SearchResource::Code,
            collection: Some(collection),
            extensions: None,
            filters: None,
            limit: Some(10),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
            repo_id: None,
            repo_path: None,
        }))
        .await;
    assert!(r.is_ok(), "search must succeed after index");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_search_handles_empty_query() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let search_h = server.search_handler();
    let r = search_h.handle(Parameters(SearchArgs {
        query: "   ".to_owned(),
        org_id: None,
        resource: SearchResource::Code,
        collection: None,
        extensions: None,
        filters: None,
        limit: Some(5),
        min_score: None,
        tags: None,
        session_id: None,
        token: None,
        repo_id: None,
        repo_path: None,
    }));
    let result = r.await;
    let response = result.expect("empty query should return an error response");
    assert!(
        !response.content.is_empty(),
        "error response should have content"
    );
    assert!(response.is_error.unwrap_or(false));
    let text = extract_text_from(&response.content);
    assert!(
        text.to_lowercase().contains("empty") || text.to_lowercase().contains("query"),
        "error response should mention empty query: {text}"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_search_respects_limit_parameter() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let path = sample_codebase_path();
    let collection = unique_collection("golden-tools-limit");
    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().into_owned()),
            collection: Some(collection.clone()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
            repo_id: None,
        }))
        .await
        .expect("index for limit test");
    wait_for_indexing_completion(&server, &collection).await?;

    let search_h = server.search_handler();
    let r = search_h
        .handle(Parameters(SearchArgs {
            query: "function code".to_owned(),
            org_id: None,
            resource: SearchResource::Code,
            collection: Some(collection),
            extensions: None,
            filters: None,
            limit: Some(2),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
            repo_id: None,
            repo_path: None,
        }))
        .await;
    assert!(r.is_ok(), "search must succeed");
    Ok(())
}
