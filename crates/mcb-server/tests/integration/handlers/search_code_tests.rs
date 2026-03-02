use std::sync::Arc;

use mcb_domain::registry::hybrid_search::{
    HybridSearchProviderConfig, resolve_hybrid_search_provider,
};
use mcb_server::args::{SearchArgs, SearchResource};
use mcb_server::handlers::SearchHandler;
use rmcp::handler::server::wrapper::Parameters;

use mcb_domain::utils::tests::fixtures::create_test_mcb_state;
use rstest::rstest;

/// Resolve hybrid search via domain registry instead of direct infra constructor
fn resolve_default_hybrid_search() -> Arc<dyn mcb_domain::ports::HybridSearchProvider> {
    resolve_hybrid_search_provider(&HybridSearchProviderConfig::new("default"))
        .expect("default hybrid search provider should be registered")
}

#[rstest]
#[tokio::test]
async fn test_search_code_success() {
    let Some((state, _services_temp_dir)) = create_test_mcb_state().await else {
        return;
    };
    let handler = SearchHandler::new(
        state.mcp_server.search_service(),
        state.mcp_server.memory_service(),
        resolve_default_hybrid_search(),
        state.mcp_server.indexing_service(),
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
        repo_path: None,
    };

    let result = handler.handle(Parameters(args)).await;

    let response = result.expect("search handler should succeed for valid code query");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(!response.is_error.unwrap_or(false));
}

#[rstest]
#[tokio::test]
async fn test_search_code_empty_query() {
    let Some((state, _services_temp_dir)) = create_test_mcb_state().await else {
        return;
    };
    let handler = SearchHandler::new(
        state.mcp_server.search_service(),
        state.mcp_server.memory_service(),
        resolve_default_hybrid_search(),
        state.mcp_server.indexing_service(),
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
        repo_path: None,
    };

    let result = handler.handle(Parameters(args)).await;

    let response = result.expect("search handler should return structured error response");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(response.is_error.unwrap_or(false));
}
