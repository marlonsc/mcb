//! Tool Router Tests
//!
//! Tests for the MCP tool registry and definitions.

use mcb_server::tools::{ToolDefinitions, create_tool_list};

#[test]
fn test_tool_definitions_index_codebase() {
    let tool = ToolDefinitions::index_codebase().expect("Should create index_codebase tool");
    assert_eq!(&*tool.name, "index_codebase");
    assert!(tool.description.is_some(), "Tool should have description");
    // input_schema is Arc<JsonObject>, verify it has schema properties
    assert!(
        !tool.input_schema.is_empty(),
        "Tool should have valid input schema"
    );
}

#[test]
fn test_tool_definitions_search_code() {
    let tool = ToolDefinitions::search_code().expect("Should create search_code tool");
    assert_eq!(&*tool.name, "search_code");
    assert!(tool.description.is_some(), "Tool should have description");
    // input_schema is Arc<JsonObject>, verify it has schema properties
    assert!(
        !tool.input_schema.is_empty(),
        "Tool should have valid input schema"
    );
}

#[test]
fn test_tool_definitions_get_indexing_status() {
    let tool =
        ToolDefinitions::get_indexing_status().expect("Should create get_indexing_status tool");
    assert_eq!(&*tool.name, "get_indexing_status");
    assert!(tool.description.is_some(), "Tool should have description");
}

#[test]
fn test_tool_definitions_clear_index() {
    let tool = ToolDefinitions::clear_index().expect("Should create clear_index tool");
    assert_eq!(&*tool.name, "clear_index");
    assert!(tool.description.is_some(), "Tool should have description");
}

#[test]
fn test_create_tool_list() {
    let tools = create_tool_list().expect("Should create tool list");
    assert_eq!(tools.len(), 17, "Should have 17 tools");

    let names: Vec<&str> = tools.iter().map(|t| &*t.name).collect();
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
}
