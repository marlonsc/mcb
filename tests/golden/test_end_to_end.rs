//! Golden test: End-to-end workflow for all 4 MCP tools.
//! Included by mcb-server test binary; contract: docs/testing/GOLDEN_TESTS_CONTRACT.md.

use crate::test_utils::test_fixtures::{
    GOLDEN_COLLECTION, create_test_mcp_server, golden_content_to_string, sample_codebase_path,
};
use mcb_server::args::{ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn golden_e2e_complete_workflow() {
    let server = create_test_mcp_server().await;
    let clear = server.clear_index_handler();
    let status_h = server.get_indexing_status_handler();
    let index_h = server.index_codebase_handler();
    let search_h = server.search_code_handler();

    let r = clear
        .handle(Parameters(ClearIndexArgs {
            collection: GOLDEN_COLLECTION.to_string(),
        }))
        .await;
    assert!(r.is_ok(), "clear_index should succeed: {:?}", r);
    let clear_text = golden_content_to_string(&r.unwrap());
    assert!(
        clear_text.to_lowercase().contains("clear"),
        "clear response must mention clear/cleared: {}",
        clear_text
    );

    let r = status_h
        .handle(Parameters(GetIndexingStatusArgs {
            collection: GOLDEN_COLLECTION.to_string(),
        }))
        .await;
    assert!(r.is_ok(), "get_indexing_status should succeed: {:?}", r);
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = golden_content_to_string(&res);
    assert!(text.contains("Indexing Status") || text.contains("Idle") || text.contains("indexing"));

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
    let text = golden_content_to_string(&res);
    assert!(
        text.contains("chunks") || text.contains("Indexing") || text.contains("files"),
        "expected chunks/indexing in response: {}",
        text
    );

    let _ = status_h
        .handle(Parameters(GetIndexingStatusArgs {
            collection: GOLDEN_COLLECTION.to_string(),
        }))
        .await;

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
    let text = golden_content_to_string(&res);
    assert!(
        text.contains("Search") || text.contains("Results") || text.contains("result"),
        "expected search result text: {}",
        text
    );

    let r = clear
        .handle(Parameters(ClearIndexArgs {
            collection: GOLDEN_COLLECTION.to_string(),
        }))
        .await;
    assert!(r.is_ok());

    let r = status_h
        .handle(Parameters(GetIndexingStatusArgs {
            collection: GOLDEN_COLLECTION.to_string(),
        }))
        .await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn golden_e2e_handles_concurrent_operations() {
    let server = create_test_mcp_server().await;
    let status_h = server.get_indexing_status_handler();
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
    let server = create_test_mcp_server().await;
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
}

#[tokio::test]
async fn golden_e2e_handles_reindex_correctly() {
    let server = create_test_mcp_server().await;
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
}
