use mcb_domain::utils::vcs_context::VcsContext;

#[test]
fn test_vcs_context_fields_optional() {
    let ctx = VcsContext {
        branch: Some("main".to_string()),
        commit: Some("abc123".to_string()),
        repo_id: Some("repo123".to_string()),
    };

    assert_eq!(ctx.branch, Some("main".to_string()));
    assert_eq!(ctx.commit, Some("abc123".to_string()));
}
