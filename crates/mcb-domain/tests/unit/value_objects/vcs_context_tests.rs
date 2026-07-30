use mcb_utils::utils::vcs_context::VcsContext;
use rstest::{fixture, rstest};

#[fixture]
fn vcs_context() -> VcsContext {
    VcsContext {
        branch: Some("main".to_owned()),
        commit: Some("abc123".to_owned()),
        repo_id: Some("repo123".to_owned()),
    }
}

#[rstest]
fn test_vcs_context_fields_optional(vcs_context: VcsContext) {
    assert_eq!(vcs_context.branch, Some("main".to_owned()));
    assert_eq!(vcs_context.commit, Some("abc123".to_owned()));
}
