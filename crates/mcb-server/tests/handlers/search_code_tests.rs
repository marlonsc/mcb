use mcb_server::args::{SearchArgs, SearchResource};
use mcb_server::handlers::SearchHandler;
use rmcp::handler::server::wrapper::Parameters;

use crate::handlers::test_helpers::create_real_domain_services;

#[tokio::test]
async fn test_search_code_success() {
    let (services, _services_temp_dir) = create_real_domain_services().await;
    let handler = SearchHandler::new(services.search_service, services.memory_service);

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
    let (services, _services_temp_dir) = create_real_domain_services().await;
    let handler = SearchHandler::new(services.search_service, services.memory_service);

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
