use mcb_domain::value_objects::project_context::{ProjectContext, parse_owner_repo};

#[test]
fn parse_ssh_shorthand() {
    assert_eq!(
        parse_owner_repo("git@github.com:marlonsc/mcb.git"),
        Some("marlonsc/mcb".to_string())
    );
}

#[test]
fn parse_https() {
    assert_eq!(
        parse_owner_repo("https://github.com/marlonsc/mcb.git"),
        Some("marlonsc/mcb".to_string())
    );
}

#[test]
fn parse_https_no_suffix() {
    assert_eq!(
        parse_owner_repo("https://github.com/marlonsc/mcb"),
        Some("marlonsc/mcb".to_string())
    );
}

#[test]
fn parse_ssh_url() {
    assert_eq!(
        parse_owner_repo("ssh://git@github.com/marlonsc/mcb.git"),
        Some("marlonsc/mcb".to_string())
    );
}

#[test]
fn parse_gitlab_subgroup() {
    assert_eq!(
        parse_owner_repo("git@gitlab.com:org/subgroup/repo.git"),
        Some("org/subgroup/repo".to_string())
    );
}

#[test]
fn parse_empty_returns_none() {
    assert_eq!(parse_owner_repo(""), None);
}

#[test]
fn resolve_returns_cached_consistent_value() {
    let ctx1 = ProjectContext::resolve();
    let ctx2 = ProjectContext::resolve();
    assert_eq!(ctx1.project_id, ctx2.project_id);
    assert_eq!(ctx1.project_name, ctx2.project_name);
}

#[test]
fn parse_gitlab_subgroup_https() {
    assert_eq!(
        parse_owner_repo("https://gitlab.com/org/subgroup/repo.git"),
        Some("org/subgroup/repo".to_string())
    );
}

#[test]
fn parse_unparseable_returns_none() {
    assert_eq!(parse_owner_repo("not-a-url"), None);
}
