#[test]
fn admin_crud_adapter_routes_through_unified_dispatch() {
    let crud_adapter = include_str!("../../src/admin/crud_adapter.rs");
    assert!(crud_adapter.contains("route_tool_call("));
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
