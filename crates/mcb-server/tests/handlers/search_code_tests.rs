use std::sync::Arc;

use mcb_server::args::{SearchArgs, SearchResource};
use mcb_server::handlers::SearchHandler;
use rmcp::handler::server::wrapper::Parameters;

use crate::test_utils::mock_services::{MockMemoryService, MockSearchService};
use crate::test_utils::test_fixtures::create_test_search_results;

#[tokio::test]
async fn test_search_code_success() {
    let search_results = create_test_search_results(5);
    let search_service = MockSearchService::new().with_results(search_results);
    let memory_service = MockMemoryService::new();
    let handler = SearchHandler::new(Arc::new(search_service), Arc::new(memory_service));

    let args = SearchArgs {
        query: "test query".to_string(),
        org_id: None,
        resource: SearchResource::Code,
        collection: Some("test".to_string()),
        limit: Some(10),
        min_score: None,
        tags: None,
        session_id: None,
        extensions: None,
        filters: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_search_code_empty_query() {
    let search_service = MockSearchService::new();
    let memory_service = MockMemoryService::new();
    let handler = SearchHandler::new(Arc::new(search_service), Arc::new(memory_service));

    let args = SearchArgs {
        query: "".to_string(),
        org_id: None,
        resource: SearchResource::Code,
        collection: Some("test".to_string()),
        limit: Some(10),
        min_score: None,
        tags: None,
        session_id: None,
        extensions: None,
        filters: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(response.is_error.unwrap_or(false));
}
