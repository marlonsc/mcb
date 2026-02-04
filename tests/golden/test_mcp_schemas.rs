use crate::test_utils::test_fixtures::{create_test_mcp_server, golden_content_to_string};
use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn golden_mcp_index_schema() {
    let server = create_test_mcp_server().await;
    let index_h = server.index_handler();

    let r = index_h
        .handle(Parameters(IndexArgs {
            action: IndexAction::Status,
            path: None,
            collection: Some("default".to_string()),
            extensions: None,
            exclude_dirs: None,
        }))
        .await;
    assert!(r.is_ok(), "index must succeed");
    let text = golden_content_to_string(&r.unwrap());
    assert!(!text.is_empty());
}

#[tokio::test]
async fn golden_mcp_search_schema() {
    let server = create_test_mcp_server().await;
    let search_h = server.search_handler();

    let r = search_h
        .handle(Parameters(SearchArgs {
            query: "test".to_string(),
            resource: SearchResource::Code,
            collection: Some("default".to_string()),
            limit: Some(5),
            min_score: None,
            tags: None,
            session_id: None,
        }))
        .await;
    assert!(r.is_ok(), "search must succeed");
}
