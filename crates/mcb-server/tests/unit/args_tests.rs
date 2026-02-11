use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use validator::Validate;

#[test]
fn test_search_args_valid() {
    let args = SearchArgs {
        org_id: None,
        query: "find authentication functions".to_string(),
        resource: SearchResource::Code,
        collection: Some("test".to_string()),
        extensions: None,
        filters: None,
        limit: Some(10),
        min_score: Some(0.5),
        tags: None,
        session_id: None,
        token: None,
    };

    assert!(args.validate().is_ok());
}

#[test]
fn test_search_args_empty_query() {
    let args = SearchArgs {
        org_id: None,
        query: "".to_string(),
        resource: SearchResource::Code,
        collection: None,
        extensions: None,
        filters: None,
        limit: Some(10),
        min_score: None,
        tags: None,
        session_id: None,
        token: None,
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_search_args_invalid_score() {
    let args = SearchArgs {
        org_id: None,
        query: "test".to_string(),
        resource: SearchResource::Code,
        collection: None,
        extensions: None,
        filters: None,
        limit: Some(10),
        min_score: Some(2.0),
        tags: None,
        session_id: None,
        token: None,
    };

    assert!(args.validate().is_err());
}

#[test]
fn test_index_args_valid() {
    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some("/tmp/test".to_string()),
        collection: Some("test".to_string()),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    assert!(args.validate().is_ok());
}

#[test]
fn test_index_args_missing_path() {
    let args = IndexArgs {
        action: IndexAction::Start,
        path: None,
        collection: None,
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    assert!(args.validate().is_ok());
}

#[test]
fn test_index_args_path_traversal() {
    let args = IndexArgs {
        action: IndexAction::Start,
        path: Some("../../../etc/passwd".to_string()),
        collection: None,
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
    };

    assert!(args.validate().is_err());
}
