use rstest::rstest;

#[rstest]
#[case("crud_adapter")]
#[case("mcp_server")]
#[case("http_transport")]
fn route_calls_are_routed_via_unified_dispatch(#[case] source_key: &str) {
    let source = match source_key {
        "crud_adapter" => include_str!("../../../src/admin/crud_adapter/unified.rs"),
        "mcp_server" => include_str!("../../../src/mcp_server.rs"),
        "http_transport" => include_str!("../../../src/transport/http/http_mcp_tools.rs"),
        _ => panic!("unknown source"),
    };
    assert!(source.contains("route_tool_call("));
}

#[test]
fn admin_web_handlers_do_not_call_repositories_directly() {
    let web_handlers = include_str!("../../../src/admin/web/entity_handlers.rs");
    assert!(!web_handlers.contains("_repository"));
    assert!(!web_handlers.contains(".create_repository("));
    assert!(!web_handlers.contains(".update_repository("));
    assert!(!web_handlers.contains(".delete_repository("));
}

#[test]
fn admin_crud_adapter_avoids_direct_repository_access() {
    let crud_adapter = include_str!("../../../src/admin/crud_adapter/unified.rs");
    assert!(crud_adapter.contains("project_id is required for repository create"));
    assert!(crud_adapter.contains("project_id is required for repository update"));
    assert!(!crud_adapter.contains("vcs_entity_repository"));
    assert!(!crud_adapter.contains("memory_repository"));
    assert!(!crud_adapter.contains("project_repository"));
}
