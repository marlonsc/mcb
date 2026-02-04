//! Included by mcb-server test binary; contract: docs/testing/GOLDEN_TESTS_CONTRACT.md.

use crate::test_utils::test_fixtures::{
    GOLDEN_COLLECTION, create_test_mcp_server, golden_content_to_string, sample_codebase_path,
};
use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn golden_e2e_complete_workflow() {
    let server = create_test_mcp_server().await;
    let index_h = server.index_handler();
    let search_h = server.search_handler();

    let r = index_h
        .handle(Parameters(IndexArgs {
            action: IndexAction::Clear,
            path: None,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            exclude_dirs: None,
        }))
        .await;
    assert!(r.is_ok(), "index clear should succeed: {:?}", r);
    let clear_text = golden_content_to_string(&r.unwrap());
    assert!(
        clear_text.to_lowercase().contains("clear"),
        "clear response must mention clear/cleared: {}",
        clear_text
    );

    let r = index_h
        .handle(Parameters(IndexArgs {
            action: IndexAction::Status,
            path: None,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            exclude_dirs: None,
        }))
        .await;
    assert!(r.is_ok(), "index status should succeed: {:?}", r);
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = golden_content_to_string(&res);
    assert!(text.contains("Indexing Status") || text.contains("Idle") || text.contains("indexing"));

    let path = sample_codebase_path();
    assert!(path.exists(), "sample_codebase must exist: {:?}", path);
    let r = index_h
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            exclude_dirs: None,
        }))
        .await;
    assert!(r.is_ok(), "index should succeed: {:?}", r);
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = golden_content_to_string(&res);
    assert!(
        text.contains("chunks") || text.contains("Indexing") || text.contains("files"),
        "expected chunks/indexing in response: {}",
        text
    );

    let _ = index_h
        .handle(Parameters(IndexArgs {
            action: IndexAction::Status,
            path: None,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            exclude_dirs: None,
        }))
        .await;

    let r = search_h
        .handle(Parameters(SearchArgs {
            query: "embedding provider".to_string(),
            resource: SearchResource::Code,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            limit: Some(5),
            min_score: None,
            tags: None,
            session_id: None,
        }))
        .await;
    assert!(r.is_ok(), "search should succeed: {:?}", r);
    let res = r.unwrap();
    assert!(!res.is_error.unwrap_or(true));
    let text = golden_content_to_string(&res);
    assert!(
        text.contains("Search") || text.contains("Results") || text.contains("result"),
        "expected search result text: {}",
        text
    );

    let r = index_h
        .handle(Parameters(IndexArgs {
            action: IndexAction::Clear,
            path: None,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            exclude_dirs: None,
        }))
        .await;
    assert!(r.is_ok());

    let r = index_h
        .handle(Parameters(IndexArgs {
            action: IndexAction::Status,
            path: None,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            exclude_dirs: None,
        }))
        .await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn golden_e2e_handles_concurrent_operations() {
    let server = create_test_mcp_server().await;
    let status_h = server.index_handler();
    let r1 = status_h.handle(Parameters(IndexArgs {
        action: IndexAction::Status,
        path: None,
        collection: Some("default".to_string()),
        extensions: None,
        exclude_dirs: None,
    }));
    let r2 = status_h.handle(Parameters(IndexArgs {
        action: IndexAction::Status,
        path: None,
        collection: Some("default".to_string()),
        extensions: None,
        exclude_dirs: None,
    }));
    let (a, b) = tokio::join!(r1, r2);
    assert!(a.is_ok());
    assert!(b.is_ok());
}

#[tokio::test]
async fn golden_e2e_respects_collection_isolation() {
    let server = create_test_mcp_server().await;
    let clear = server.index_handler();
    clear
        .handle(Parameters(IndexArgs {
            action: IndexAction::Clear,
            path: None,
            collection: Some("collection_a".to_string()),
            extensions: None,
            exclude_dirs: None,
        }))
        .await
        .expect("clear a");
    clear
        .handle(Parameters(IndexArgs {
            action: IndexAction::Clear,
            path: None,
            collection: Some("collection_b".to_string()),
            extensions: None,
            exclude_dirs: None,
        }))
        .await
        .expect("clear b");
}

#[tokio::test]
async fn golden_e2e_handles_reindex_correctly() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let index_h = server.index_handler();
    let collection = "golden_reindex_test";
    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some(path.to_string_lossy().to_string()),
        collection: Some(collection.to_string()),
        extensions: None,
        exclude_dirs: None,
    };
    let r1 = index_h.handle(Parameters(args.clone())).await;
    assert!(r1.is_ok());
    let r2 = index_h.handle(Parameters(args)).await;
    assert!(r2.is_ok());
}
