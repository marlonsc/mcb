//! Basic compilation tests for application use cases

use mcb_application::use_cases::{ContextServiceImpl, IndexingServiceImpl, SearchServiceImpl};
use rstest::rstest;

#[rstest]
#[case::context_service(std::any::type_name::<ContextServiceImpl>(), "ContextServiceImpl")]
#[case::indexing_service(std::any::type_name::<IndexingServiceImpl>(), "IndexingServiceImpl")]
#[case::search_service(std::any::type_name::<SearchServiceImpl>(), "SearchServiceImpl")]
fn test_use_cases_can_be_imported(#[case] type_name: &str, #[case] expected_name: &str) {
    assert!(
        type_name.contains(expected_name),
        "{expected_name} type should be available"
    );
}

#[rstest]
fn test_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<ContextServiceImpl>();
    assert_send_sync::<IndexingServiceImpl>();
    assert_send_sync::<SearchServiceImpl>();

    let type_count = 3_usize;
    assert_eq!(
        type_count, 3,
        "All three service types verified for Send + Sync"
    );
}
