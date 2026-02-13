#[test]
fn admin_crud_adapter_routes_through_unified_dispatch() {
    let crud_adapter = include_str!("../../src/admin/crud_adapter.rs");
    assert!(crud_adapter.contains("route_tool_call("));
    assert!(crud_adapter.contains("project_id is required for repository create"));
    assert!(crud_adapter.contains("project_id is required for repository update"));
    assert!(!crud_adapter.contains("vcs_entity_repository"));
    assert!(!crud_adapter.contains("memory_repository"));
    assert!(!crud_adapter.contains("project_repository"));
}

#[test]
fn admin_web_handlers_do_not_call_repositories_directly() {
    let web_handlers = include_str!("../../src/admin/web/entity_handlers.rs");
    assert!(!web_handlers.contains("_repository"));
    assert!(!web_handlers.contains(".create_repository("));
    assert!(!web_handlers.contains(".update_repository("));
    assert!(!web_handlers.contains(".delete_repository("));
}

#[test]
fn mcp_server_routes_tool_calls_via_unified_router() {
    let mcp_server = include_str!("../../src/mcp_server.rs");
    assert!(mcp_server.contains("route_tool_call("));
}

#[test]
fn http_transport_routes_tool_calls_via_unified_router() {
    let http_transport = include_str!("../../src/transport/http.rs");
    assert!(http_transport.contains("route_tool_call("));
}
