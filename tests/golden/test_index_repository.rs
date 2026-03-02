use mcb_domain::utils::tests::fixtures::{GOLDEN_COLLECTION, create_test_mcp_server, sample_codebase_path};
use mcb_server::args::{IndexAction, IndexArgs};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn golden_index_repository_success() {
    let server = create_test_mcp_server().await;
    let handler = server.index_handler();
    let path = sample_codebase_path();

    let r = handler
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
        .await;

    assert!(r.is_ok(), "index must succeed");
}

#[tokio::test]
async fn golden_index_repository_with_extensions() {
    let server = create_test_mcp_server().await;
    let handler = server.index_handler();
    let path = sample_codebase_path();

    let r = handler
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: Some(vec!["rs".to_string()]),
            exclude_dirs: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;

    assert!(r.is_ok(), "index with extensions must succeed");
}

#[tokio::test]
async fn golden_index_repository_exclude_dirs() {
    let server = create_test_mcp_server().await;
    let handler = server.index_handler();
    let path = sample_codebase_path();

    let r = handler
        .handle(Parameters(IndexArgs {
            action: IndexAction::Start,
            path: Some(path.to_string_lossy().to_string()),
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            exclude_dirs: Some(vec!["target".to_string()]),
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;

    assert!(r.is_ok(), "index with exclude dirs must succeed");
}
