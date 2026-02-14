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
fn test_parse_owner_repo(#[case] input: &str, #[case] expected: Option<&str>) {
    assert_eq!(
        parse_owner_repo(input),
        expected.map(std::string::ToString::to_string)
    );
}

#[rstest]
#[case(2)]
#[case(3)]
fn resolve_returns_cached_consistent_value(#[case] calls: usize) {
    let first = ProjectContext::resolve();
    for _ in 1..calls {
        let next = ProjectContext::resolve();
        assert_eq!(first.project_id, next.project_id);
        assert_eq!(first.project_name, next.project_name);
    }
}
