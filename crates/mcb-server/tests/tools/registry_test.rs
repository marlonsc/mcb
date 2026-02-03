//! Tool Registry Tests

use mcb_server::tools::registry::create_tool_list;

#[test]
fn test_tool_definitions_create_valid_tools() {
    let tools = create_tool_list().expect("should create tool list");
    assert_eq!(tools.len(), 25);

    let names: Vec<_> = tools.iter().map(|t| t.name.as_ref()).collect();
    // Core tools
    assert!(names.contains(&"index_codebase"));
    assert!(names.contains(&"search_code"));
    assert!(names.contains(&"get_indexing_status"));
    assert!(names.contains(&"clear_index"));
    // Validation tools
    assert!(names.contains(&"validate_architecture"));
    assert!(names.contains(&"validate_file"));
    assert!(names.contains(&"list_validators"));
    assert!(names.contains(&"get_validation_rules"));
    assert!(names.contains(&"analyze_complexity"));
    // Memory tools
    assert!(names.contains(&"store_observation"));
    assert!(names.contains(&"search_memories"));
    assert!(names.contains(&"get_session_summary"));
    assert!(names.contains(&"create_session_summary"));
    // Memory progressive disclosure (memory_ prefix)
    assert!(names.contains(&"memory_timeline"));
    assert!(names.contains(&"memory_get_observations"));
    assert!(names.contains(&"memory_inject_context"));
    assert!(names.contains(&"memory_search"));
    // Agent session tracking tools
    assert!(names.contains(&"create_agent_session"));
    assert!(names.contains(&"get_agent_session"));
    assert!(names.contains(&"update_agent_session"));
    assert!(names.contains(&"list_agent_sessions"));
    assert!(names.contains(&"store_tool_call"));
    assert!(names.contains(&"store_delegation"));
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
