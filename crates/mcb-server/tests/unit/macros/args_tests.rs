//! Tool argument validation: agent inputs are validated before dispatch.
//!
//! Tests verify that valid queries/paths pass validation and that dangerous
//! or empty inputs are rejected with clear errors.

use mcb_server::args::{IndexAction, IndexArgs, SearchArgs, SearchResource};
use rstest::rstest;
use validator::Validate;

fn search(query: &str, min_score: Option<f32>, collection: Option<&str>) -> SearchArgs {
    SearchArgs {
        query: query.to_owned(),
        org_id: None,
        resource: SearchResource::Code,
        collection: collection.map(ToOwned::to_owned),
        extensions: None,
        filters: None,
        limit: Some(10),
        min_score,
        tags: None,
        session_id: None,
        token: None,
        repo_id: None,
        repo_path: None,
    }
}

fn index(path: Option<&str>, collection: Option<&str>) -> IndexArgs {
    IndexArgs {
        action: IndexAction::Start,
        path: path.map(ToOwned::to_owned),
        collection: collection.map(ToOwned::to_owned),
        extensions: None,
        exclude_dirs: None,
        ignore_patterns: None,
        max_file_size: None,
        follow_symlinks: None,
        token: None,
        repo_id: None,
    }
}

// ─── Search validation ───────────────────────────────────────────────

#[rstest]
#[case(search("find auth functions", Some(0.5), Some("test")), true)]
#[case(search("", None, None), false)]
#[case(search("test", Some(2.0), None), false)]
fn search_query_validated(#[case] args: SearchArgs, #[case] valid: bool) {
    assert_eq!(args.validate().is_ok(), valid);
}

// ─── Index validation ────────────────────────────────────────────────

#[rstest]
#[case(index(Some("/tmp/project"), Some("test")), true)]
#[case(index(None, None), true)]
#[case(index(Some("../../../etc/passwd"), None), false)]
fn index_path_validated(#[case] args: IndexArgs, #[case] valid: bool) {
    assert_eq!(args.validate().is_ok(), valid);
}
