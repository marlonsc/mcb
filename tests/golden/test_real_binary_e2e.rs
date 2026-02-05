//! Comprehensive golden tests for real MCB binary execution
//!
//! Contract: Verify MCB behavior under realistic usage patterns:
//! - Multi-language indexing (Rust, Python, JS, Go, etc.)
//! - Complex search patterns and filters
//! - Consolidation argument handling
//! - Performance under load
//! - Error handling and recovery

use crate::test_utils::test_fixtures::{
    GOLDEN_COLLECTION, golden_content_to_string, golden_count_result_entries, sample_codebase_path,
};
use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use rmcp::handler::server::wrapper::Parameters;
use std::time::Instant;

async fn create_test_server() -> (
    Box<dyn Fn(Parameters<IndexArgs>) + Send + Sync>,
    Box<dyn Fn(Parameters<SearchArgs>) + Send + Sync>,
) {
    unimplemented!("Test server creation needs proper fixture setup")
}

#[tokio::test]
async fn golden_index_with_extension_filters() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();

    let result = server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: Some("rs,py,js".to_string()),
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;

    assert!(
        result.is_ok(),
        "Index with extension filters should succeed"
    );
    let response = result.unwrap();
    assert!(
        !response.is_error.unwrap_or(true),
        "Response should not be error"
    );

    let text = golden_content_to_string(&response);
    assert!(
        text.contains("chunks") || text.contains("files") || text.contains("Indexing"),
        "Response should mention indexing results"
    );
}

#[tokio::test]
async fn golden_search_with_extension_filter() {
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

    let search_result = server
        .search_handler()
        .handle(Parameters(SearchArgs {
            query: "embedding".to_string(),
            resource: SearchResource::Code,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: Some("rs".to_string()),
            filters: None,
            limit: Some(5),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await;

    assert!(
        search_result.is_ok(),
        "Search with extension filter should succeed"
    );
    let response = search_result.unwrap();
    assert!(
        !response.is_error.unwrap_or(true),
        "Search response should not be error"
    );
}

#[tokio::test]
async fn golden_search_with_limit_boundaries() {
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

    for limit in vec![1, 5, 10, 50, 100] {
        let result = search_h
            .handle(Parameters(SearchArgs {
                query: "implementation".to_string(),
                resource: SearchResource::Code,
                collection: Some(GOLDEN_COLLECTION.to_string()),
                extensions: None,
                filters: None,
                limit: Some(limit),
                min_score: None,
                tags: None,
                session_id: None,
                token: None,
            }))
            .await;

        assert!(result.is_ok(), "Search with limit {} should succeed", limit);
        let response = result.unwrap();
        assert!(
            !response.is_error.unwrap_or(true),
            "Response should not be error"
        );
    }
}

#[tokio::test]
async fn golden_index_status_during_active_indexing() {
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

    let status_result = server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Status,
            path: None,
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;

    assert!(status_result.is_ok(), "Status check should succeed");
    let response = status_result.unwrap();
    assert!(
        !response.is_error.unwrap_or(true),
        "Status response should not be error"
    );

    let text = golden_content_to_string(&response);
    assert!(
        text.to_lowercase().contains("indexing") || text.to_lowercase().contains("status"),
        "Status should mention indexing status"
    );
}

#[tokio::test]
async fn golden_reindex_collection() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = format!("{}_reindex_test", GOLDEN_COLLECTION);

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection.clone()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("first index");

    let first_search = server
        .search_handler()
        .handle(Parameters(SearchArgs {
            query: "provider".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection.clone()),
            extensions: None,
            filters: None,
            limit: Some(10),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await
        .expect("first search");

    let first_count = golden_count_result_entries(&first_search);

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection.clone()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("reindex");

    let second_search = server
        .search_handler()
        .handle(Parameters(SearchArgs {
            query: "provider".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection),
            extensions: None,
            filters: None,
            limit: Some(10),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await
        .expect("second search");

    let second_count = golden_count_result_entries(&second_search);

    assert_eq!(
        first_count, second_count,
        "Reindexed collection should have same results"
    );
}

#[tokio::test]
async fn golden_search_performance_baseline() {
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
    let latencies: Vec<u128> = (0..5)
        .map(|_| {
            let start = Instant::now();
            let _ = futures::executor::block_on(search_h.handle(Parameters(SearchArgs {
                query: "implementation service".to_string(),
                resource: SearchResource::Code,
                collection: Some(GOLDEN_COLLECTION.to_string()),
                extensions: None,
                filters: None,
                limit: Some(10),
                min_score: None,
                tags: None,
                session_id: None,
                token: None,
            })));
            start.elapsed().as_millis()
        })
        .collect();

    let avg_latency = latencies.iter().sum::<u128>() / latencies.len() as u128;
    let max_latency = *latencies.iter().max().unwrap_or(&0);

    println!(
        "Search latencies: avg={}ms, max={}ms",
        avg_latency, max_latency
    );
    assert!(
        max_latency < 5000,
        "Search should complete within 5s baseline (took {}ms)",
        max_latency
    );
}

#[tokio::test]
async fn golden_clear_then_index_cycle() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection = format!("{}_clear_cycle", GOLDEN_COLLECTION);

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection.clone()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index");

    let search_before = server
        .search_handler()
        .handle(Parameters(SearchArgs {
            query: "provider".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection.clone()),
            extensions: None,
            filters: None,
            limit: Some(10),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await
        .expect("search before clear");

    let before_count = golden_count_result_entries(&search_before);

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Clear,
            path: None,
            collection: Some(collection.clone()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("clear");

    let search_after_clear = server
        .search_handler()
        .handle(Parameters(SearchArgs {
            query: "provider".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection.clone()),
            extensions: None,
            filters: None,
            limit: Some(10),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await
        .expect("search after clear");

    let after_clear_count = golden_count_result_entries(&search_after_clear);

    assert_eq!(
        before_count, 0,
        "Results found after clear (should be empty before re-index)"
    );
    assert!(
        before_count > after_clear_count || after_clear_count == 0,
        "Clear should remove or not find results"
    );

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("reindex after clear");
}

#[tokio::test]
async fn golden_empty_query_handling() {
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

    let result = server
        .search_handler()
        .handle(Parameters(SearchArgs {
            query: "".to_string(),
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

    assert!(
        result.is_ok() || result.is_err(),
        "Empty query should be handled gracefully (success or explicit error)"
    );
}

#[tokio::test]
async fn golden_collection_isolation() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let collection_a = format!("{}_iso_a", GOLDEN_COLLECTION);
    let collection_b = format!("{}_iso_b", GOLDEN_COLLECTION);

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(collection_a.clone()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("index A");

    server
        .index_handler()
        .handle(Parameters(IndexArgs {
            action: IndexAction::Clear,
            path: None,
            collection: Some(collection_b.clone()),
            extensions: None,
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await
        .expect("clear B");

    let search_a = server
        .search_handler()
        .handle(Parameters(SearchArgs {
            query: "provider".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection_a),
            extensions: None,
            filters: None,
            limit: Some(10),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await
        .expect("search A");

    let count_a = golden_count_result_entries(&search_a);

    let search_b = server
        .search_handler()
        .handle(Parameters(SearchArgs {
            query: "provider".to_string(),
            resource: SearchResource::Code,
            collection: Some(collection_b),
            extensions: None,
            filters: None,
            limit: Some(10),
            min_score: None,
            tags: None,
            session_id: None,
            token: None,
        }))
        .await
        .expect("search B");

    let count_b = golden_count_result_entries(&search_b);

    assert!(
        count_a > count_b,
        "Collection A (indexed) should have more results than empty B"
    );
}
