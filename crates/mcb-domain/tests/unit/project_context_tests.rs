use mcb_domain::value_objects::project_context::{ProjectContext, parse_owner_repo};
use rstest::rstest;

#[rstest]
#[case("git@github.com:marlonsc/mcb.git", Some("marlonsc/mcb"))]
#[case("https://github.com/marlonsc/mcb.git", Some("marlonsc/mcb"))]
#[case("https://github.com/marlonsc/mcb", Some("marlonsc/mcb"))]
#[case("ssh://git@github.com/marlonsc/mcb.git", Some("marlonsc/mcb"))]
#[case("git@gitlab.com:org/subgroup/repo.git", Some("org/subgroup/repo"))]
#[case("https://gitlab.com/org/subgroup/repo.git", Some("org/subgroup/repo"))]
#[case("", None)]
#[case("not-a-url", None)]
#[test]
fn test_parse_owner_repo(#[case] input: &str, #[case] expected: Option<&str>) {
    assert_eq!(
        parse_owner_repo(input),
        expected.map(std::string::ToString::to_string)
    );
}

#[test]
// TODO(TEST003): Bad test function name 'resolve_returns_cached_consistent_value'. Use 'test_resolve_returns_cached_consistent_value'.
fn resolve_returns_cached_consistent_value() {
    let ctx1 = ProjectContext::resolve();
    let ctx2 = ProjectContext::resolve();
    assert_eq!(ctx1.project_id, ctx2.project_id);
    assert_eq!(ctx1.project_name, ctx2.project_name);
}
