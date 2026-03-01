use mcb_server::tools::registry::create_tool_list;
use rstest::rstest;

#[rstest]
#[case("index")]
#[case("search")]
#[case("validate")]
#[case("memory")]
#[case("session")]
#[case("agent")]
#[case("project")]
#[case("vcs")]
#[case("entity")]
fn test_tool_definitions_create_valid_tools(#[case] expected_name: &str) {
    let tools = create_tool_list().expect("should create tool list");
    assert_eq!(tools.len(), 9);

    let names: Vec<_> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(names.contains(&expected_name));
}

#[rstest]
#[test]
fn test_each_tool_has_description() {
    let tools = create_tool_list().expect("should create tool list");
    for tool in tools {
        assert!(
            tool.description.is_some(),
            "Tool {} should have description",
            tool.name
        );
    }
}
