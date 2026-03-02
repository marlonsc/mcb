use mcb_infrastructure::services::indexing_service::IndexingServiceImpl;
use rstest::rstest;
use std::path::Path;

#[rstest]
#[test]
fn workspace_relative_path_normalizes_within_workspace() {
    let workspace = Path::new("/repo");
    let file = Path::new("/repo/src/main.rs");
    let relative =
        IndexingServiceImpl::workspace_relative_path(file, workspace).expect("relative path");
    assert_eq!(relative, "src/main.rs");
}

#[rstest]
#[test]
fn workspace_relative_path_rejects_outside_workspace() {
    let workspace = Path::new("/repo");
    let file = Path::new("/other/main.rs");
    let err = IndexingServiceImpl::workspace_relative_path(file, workspace)
        .expect_err("outside path must fail");
    assert!(err.to_string().contains("is not under root"));
}
