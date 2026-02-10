use mcb_server::tools::registry::create_tool_list;

#[test]
fn test_tool_definitions_create_valid_tools() {
    let tools = create_tool_list().expect("should create tool list");
    assert_eq!(tools.len(), 10);

    let names: Vec<_> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(names.contains(&"index"));
    assert!(names.contains(&"search"));
    assert!(names.contains(&"validate"));
    assert!(names.contains(&"memory"));
    assert!(names.contains(&"session"));
    assert!(names.contains(&"agent"));
    assert!(names.contains(&"project"));
    assert!(names.contains(&"vcs"));
    assert!(names.contains(&"vcs_entity"));
    assert!(names.contains(&"plan_entity"));
}

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
