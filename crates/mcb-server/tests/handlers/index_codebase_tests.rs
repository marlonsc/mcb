use mcb_server::args::{IndexAction, IndexArgs};
use mcb_server::handlers::IndexHandler;
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;

use crate::handlers::test_helpers::create_real_domain_services;
use crate::test_utils::test_fixtures::create_temp_codebase;

#[rstest]
#[case(true, None, Some("test"), true)]
#[case(false, Some("/nonexistent/path/to/codebase"), Some("test"), false)]
#[case(false, None, Some("test"), false)]
#[case(true, None, None, false)]
#[case(false, Some("/definitely/nonexistent/mcb-path"), Some("test"), false)]
#[tokio::test]
async fn test_index_codebase(
    #[case] create_codebase: bool,
    #[case] path_override: Option<&str>,
    #[case] collection: Option<&str>,
    #[case] should_succeed: bool,
) {
    let Some((services, _services_temp_dir)) = create_real_domain_services().await else {
        return;
    };
    let handler = IndexHandler::new(services.indexing_service);

    let _temp_dir_guard;
    let path_val = if create_codebase {
        let (td, p) = create_temp_codebase();
        _temp_dir_guard = Some(td);
        Some(p.to_string_lossy().to_string())
    } else {
        _temp_dir_guard = None;
        path_override.map(std::borrow::ToOwned::to_owned)
    };

    let args = IndexArgs {
        action: IndexAction::Start,
        path: path_val,
        collection: collection.map(std::borrow::ToOwned::to_owned),
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
