use mcb_server::args::{IndexAction, IndexArgs};
use mcb_server::handlers::IndexHandler;
use rmcp::handler::server::wrapper::Parameters;

use crate::utils::domain_services::create_real_domain_services;

#[tokio::test]
async fn test_get_indexing_status_success() {
    let Some((services, _services_temp_dir)) = create_real_domain_services().await else {
        return;
    };
    let handler = IndexHandler::new(services.indexing_service);

    let args = IndexArgs {
        action: IndexAction::Status,
        path: None,
        collection: None,
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}
