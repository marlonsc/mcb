//! Unit tests for route discovery and registry
//!
//! Tests for dynamic route registration and discovery for the admin API.

use mcp_context_browser::application::admin::helpers::route_discovery::{
    build_standard_routes, RouteRegistry,
};
use mcp_context_browser::application::admin::types::RouteInfo;
use std::sync::Arc;

#[tokio::test]
async fn test_route_registry_register() {
    let registry = RouteRegistry::new();
    let route = RouteInfo {
        id: "test_1".to_string(),
        path: "/test".to_string(),
        method: "GET".to_string(),
        handler: "test_handler".to_string(),
        auth_required: false,
        rate_limit: None,
    };

    registry.register(route.clone()).await;
    assert_eq!(registry.count().await, 1);
    assert!(registry.exists("GET", "/test").await);
}

#[tokio::test]
async fn test_route_registry_filter() {
    let _registry = Arc::new(RouteRegistry::new());
    let routes = build_standard_routes().await;
    let stats = routes.get_discovery_info().await;
    assert!(stats.total_routes > 0);
    assert!(stats.public_routes > 0);
}
