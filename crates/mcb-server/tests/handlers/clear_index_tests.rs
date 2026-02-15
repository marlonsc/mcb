use mcb_server::args::{IndexAction, IndexArgs};
use mcb_server::handlers::IndexHandler;
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;

use crate::handlers::test_helpers::create_real_domain_services;

#[rstest]
#[case(Some("test_collection".to_string()), true)]
#[case(None, false)]
#[tokio::test]
async fn test_clear_index(#[case] collection: Option<String>, #[case] should_succeed: bool) {
    let Some((services, _services_temp_dir)) = create_real_domain_services().await else {
        return;
    };
    let handler = IndexHandler::new(services.indexing_service);

    let args = IndexArgs {
        action: IndexAction::Clear,
        path: None,
        collection,
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    let result = handler.handle(Parameters(args)).await;

    if should_succeed {
        assert!(result.is_ok());
        let response = result.expect("Expected successful response");
        assert!(!response.is_error.unwrap_or(false));
    } else {
        assert!(result.is_err());
    }
}
