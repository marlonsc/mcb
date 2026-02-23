use mcb_domain::utils::vcs_context::VcsContext;
use rstest::rstest;

#[rstest]
fn test_vcs_context_fields_optional() {
    let ctx = VcsContext {
        branch: Some("main".to_owned()),
        commit: Some("abc123".to_owned()),
        repo_id: Some("repo123".to_owned()),
    };

    assert_eq!(ctx.branch, Some("main".to_owned()));
    assert_eq!(ctx.commit, Some("abc123".to_owned()));
}
