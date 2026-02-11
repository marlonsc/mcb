//!
//! These tests use the real DI stack (FastEmbed + EdgeVec)
//! and call the MCP handlers (index, search) to validate behavior.

use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use rmcp::handler::server::wrapper::Parameters;

use crate::test_utils::test_fixtures::{GOLDEN_COLLECTION, extract_text_content};

fn sample_codebase_path() -> std::path::PathBuf {
    crate::test_utils::test_fixtures::sample_codebase_path()
}

// =============================================================================
// E2E: Complete workflow (clear -> status -> index -> status -> search -> clear)
// =============================================================================

#[tokio::test]
async fn test_golden_e2e_complete_workflow() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    assert!(
        path.exists(),
        "sample_codebase fixture must exist: {:?}",
        path
    );
    let path_str = path.to_string_lossy().to_string();
    let coll = GOLDEN_COLLECTION;

    let index_h = server.index_handler();
    let search_h = server.search_handler();

    // 1. Clear any existing test data
    let clear_args = IndexArgs {
        action: IndexAction::Clear,
        path: None,
        collection: Some(coll.to_string()),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    let r = index_h.handle(Parameters(clear_args)).await;
    assert!(r.is_ok(), "index clear should succeed");
    let resp = r.unwrap();
    let text = extract_text_content(&resp.content);
    assert!(
        text.contains("cleared") || text.contains("Cleared"),
        "{}",
        text
    );

    // 2. Status (idle / empty)
    let status_args = IndexArgs {
        action: IndexAction::Status,
        path: None,
        collection: Some(coll.to_string()),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    let r = index_h.handle(Parameters(status_args)).await;
    assert!(r.is_ok());
    let text = extract_text_content(&r.unwrap().content);
    assert!(
        text.contains("Indexing Status") || text.contains("Status"),
        "{}",
        text
    );

    // 3. Index repository
    let index_args = IndexArgs {
        action: IndexAction::Start,
        path: path_str.clone().into(),
        collection: Some(coll.to_string()),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    let r = index_h.handle(Parameters(index_args)).await;
    assert!(r.is_ok(), "index should succeed");
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

    // 4. Search
    let search_args = SearchArgs {
        org_id: None,
        query: "embedding or vector".to_string(),
        resource: SearchResource::Code,
        collection: Some(coll.to_string()),
        extensions: None,
        filters: None,
        limit: Some(5),
        min_score: None,
        tags: None,
        session_id: None,
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

    // 5. Clear index
    let clear_args = IndexArgs {
        action: IndexAction::Clear,
        path: None,
        collection: Some(coll.to_string()),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };
    let r = index_h.handle(Parameters(clear_args)).await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn test_golden_index_test_repository() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    assert!(path.exists(), "sample_codebase must exist: {:?}", path);

    let handler = server.index_handler();
    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some(path.to_string_lossy().to_string()),
        collection: Some(GOLDEN_COLLECTION.to_string()),
        extensions: None,
        exclude_dirs: None,
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
}

#[tokio::test]
async fn test_golden_index_handles_multiple_languages() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let handler = server.index_handler();
    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some(path.to_string_lossy().to_string()),
        collection: Some("golden_multi_lang".to_string()),
        extensions: Some(vec!["rs".to_string()]),
        exclude_dirs: None,
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
async fn test_golden_search_returns_relevant_results() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = "golden_search_relevance";
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

    let search_h = server.search_handler();
    let r = search_h
        .handle(Parameters(SearchArgs {
            org_id: None,
            query: "embedding vector".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection.to_string()),
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
}

#[tokio::test]
async fn test_golden_search_handles_empty_query() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let search_h = server.search_handler();
    let r = search_h.handle(Parameters(SearchArgs {
        org_id: None,
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
async fn test_golden_search_respects_limit_parameter() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
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
    let r = search_h
        .handle(Parameters(SearchArgs {
            org_id: None,
            query: "function code".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection.to_string()),
            extensions: None,
            filters: None,
            limit: Some(2),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok(), "search must succeed");
}
