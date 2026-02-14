use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use rstest::rstest;
use validator::Validate;

fn build_search_args(query: &str, min_score: Option<f32>, collection: Option<&str>) -> SearchArgs {
    SearchArgs {
        query: query.to_string(),
        org_id: None,
        resource: SearchResource::Code,
        collection: collection.map(str::to_string),
        extensions: None,
        filters: None,
        limit: Some(10),
        min_score,
        tags: None,
        session_id: None,
        token: None,
    }
}

#[rstest]
#[case(
    build_search_args("find authentication functions", Some(0.5), Some("test")),
    true
)]
#[case(build_search_args("", None, None), false)]
#[case(build_search_args("test", Some(2.0), None), false)]
#[test]
fn test_search_args_validation(#[case] args: SearchArgs, #[case] expected_valid: bool) {
    assert_eq!(args.validate().is_ok(), expected_valid);
}

fn build_index_args(path: Option<&str>, collection: Option<&str>) -> IndexArgs {
    IndexArgs {
        action: IndexAction::Start,
        path: path.map(str::to_string),
        collection: collection.map(str::to_string),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    }
}

#[rstest]
#[case(build_index_args(Some("/tmp/test"), Some("test")), true)]
#[case(build_index_args(None, None), true)]
#[case(build_index_args(Some("../../../etc/passwd"), None), false)]
#[test]
fn test_index_args_validation(#[case] args: IndexArgs, #[case] expected_valid: bool) {
    assert_eq!(args.validate().is_ok(), expected_valid);
}
