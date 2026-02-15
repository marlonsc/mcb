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
fn new_constructs_project_context() {
    let context = ProjectContext::new("marlonsc/mcb", "mcb");

    assert_eq!(context.project_id, "marlonsc/mcb");
    assert_eq!(context.project_name, "mcb");
    assert!(!context.is_submodule);
    assert_eq!(context.superproject_id, None);
}
