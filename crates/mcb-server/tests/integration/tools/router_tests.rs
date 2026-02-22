//!
//! Tests for the MCP tool registry and definitions.

use mcb_server::tools::{ToolDefinitions, create_tool_list};
use rstest::rstest;

#[rstest]
#[case("index")]
#[case("search")]
#[case("validate")]
#[case("memory")]
fn test_tool_definitions_core(#[case] tool_name: &str) {
    let tool = ToolDefinitions::by_name(tool_name).expect("Should create tool");

    assert_eq!(&*tool.name, tool_name);
    assert!(tool.description.is_some(), "Tool should have description");
    if matches!(tool_name, "index" | "search") {
        assert!(
            !tool.input_schema.is_empty(),
            "Tool should have valid input schema"
        );
    }
}

#[test]
fn test_create_tool_list() {
    let tools = create_tool_list().expect("Should create tool list");
    assert_eq!(tools.len(), 9, "Should have 9 tools");

    let names: Vec<&str> = tools.iter().map(|t| &*t.name).collect();
    assert!(names.contains(&"index"));
    assert!(names.contains(&"search"));
    assert!(names.contains(&"validate"));
    assert!(names.contains(&"memory"));
    assert!(names.contains(&"session"));
    assert!(names.contains(&"agent"));
    assert!(names.contains(&"project"));
    assert!(names.contains(&"vcs"));
    assert!(names.contains(&"entity"));
}
