//!
//! Tests for the MCP tool registry and definitions.

use mcb_server::tools::{ToolDefinitions, create_tool_list};

#[test]
fn test_tool_definitions_index() {
    let tool = ToolDefinitions::index().expect("Should create index tool");
    assert_eq!(&*tool.name, "index");
    assert!(tool.description.is_some(), "Tool should have description");
    assert!(
        !tool.input_schema.is_empty(),
        "Tool should have valid input schema"
    );
}

#[test]
fn test_tool_definitions_search() {
    let tool = ToolDefinitions::search().expect("Should create search tool");
    assert_eq!(&*tool.name, "search");
    assert!(tool.description.is_some(), "Tool should have description");
    assert!(
        !tool.input_schema.is_empty(),
        "Tool should have valid input schema"
    );
}

#[test]
fn test_tool_definitions_validate() {
    let tool = ToolDefinitions::validate().expect("Should create validate tool");
    assert_eq!(&*tool.name, "validate");
    assert!(tool.description.is_some(), "Tool should have description");
}

#[test]
fn test_tool_definitions_memory() {
    let tool = ToolDefinitions::memory().expect("Should create memory tool");
    assert_eq!(&*tool.name, "memory");
    assert!(tool.description.is_some(), "Tool should have description");
}

#[test]
fn test_create_tool_list() {
    let tools = create_tool_list().expect("Should create tool list");
    assert_eq!(tools.len(), 10, "Should have 10 tools");

    let names: Vec<&str> = tools.iter().map(|t| &*t.name).collect();
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
