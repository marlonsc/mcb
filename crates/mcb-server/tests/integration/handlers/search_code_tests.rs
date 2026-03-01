use mcb_infrastructure::resolution_context::create_default_hybrid_search_provider;
use mcb_server::args::{SearchArgs, SearchResource};
use mcb_server::handlers::SearchHandler;
use rmcp::handler::server::wrapper::Parameters;

use crate::utils::domain_services::create_real_domain_services;
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_search_code_success() {
    let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
        return;
    };
    let handler = SearchHandler::new(
        state.mcp_server.search_service(),
        state.mcp_server.memory_service(),
        create_default_hybrid_search_provider(),
    );

    let args = SearchArgs {
        query: "test query".to_owned(),
        org_id: None,
        resource: SearchResource::Code,
        collection: Some("test".to_owned()),
        limit: Some(10),
        min_score: None,
        tags: None,
        session_id: None,
        extensions: None,
        filters: None,
        token: None,
        repo_id: None,
    };

    let result = handler.handle(Parameters(args)).await;

    let response = result.expect("search handler should succeed for valid code query");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(!response.is_error.unwrap_or(false));
}

#[rstest]
#[tokio::test]
async fn test_search_code_empty_query() {
    let Some((state, _services_temp_dir)) = create_real_domain_services().await else {
        return;
    };
    let handler = SearchHandler::new(
        state.mcp_server.search_service(),
        state.mcp_server.memory_service(),
        create_default_hybrid_search_provider(),
    );

    let args = SearchArgs {
        query: String::new(),
        org_id: None,
        resource: SearchResource::Code,
        collection: Some("test".to_owned()),
        limit: Some(10),
        min_score: None,
        tags: None,
        session_id: None,
        extensions: None,
        filters: None,
        token: None,
        repo_id: None,
    };

    let result = handler.handle(Parameters(args)).await;

    let response = result.expect("search handler should return structured error response");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(response.is_error.unwrap_or(false));
}
