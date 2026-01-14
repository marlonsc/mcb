//! Route discovery and registry helper module
//!
//! Provides dynamic route registration and discovery for the admin API.
//! This allows routes to be registered at runtime and discovered via the API.

use crate::application::admin::helpers::admin_defaults;
use crate::application::admin::types::RouteInfo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Dynamic route registry for runtime route discovery
pub struct RouteRegistry {
    routes: RwLock<HashMap<String, RouteInfo>>,
}

impl RouteRegistry {
    /// Create a new empty route registry
    pub fn new() -> Self {
        Self {
            routes: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new route
    pub async fn register(&self, route: RouteInfo) {
        let mut routes = self.routes.write().await;
        let route_key = format!("{} {}", route.method, route.path);
        routes.insert(route_key, route);
    }

    /// Register multiple routes at once
    pub async fn register_bulk(&self, new_routes: Vec<RouteInfo>) {
        let mut routes = self.routes.write().await;
        for route in new_routes {
            let route_key = format!("{} {}", route.method, route.path);
            routes.insert(route_key, route);
        }
    }

    /// Get all registered routes
    pub async fn get_all(&self) -> Vec<RouteInfo> {
        let routes = self.routes.read().await;
        let mut result: Vec<RouteInfo> = routes.values().cloned().collect();
        // Sort by path for consistent ordering
        result.sort_by(|a, b| {
            let path_cmp = a.path.cmp(&b.path);
            if path_cmp == std::cmp::Ordering::Equal {
                a.method.cmp(&b.method)
            } else {
                path_cmp
            }
        });
        result
    }

    /// Get routes by path pattern
    pub async fn get_by_path(&self, path_pattern: &str) -> Vec<RouteInfo> {
        let routes = self.routes.read().await;
        routes
            .values()
            .filter(|r| r.path.contains(path_pattern))
            .cloned()
            .collect()
    }

    /// Get routes by method
    pub async fn get_by_method(&self, method: &str) -> Vec<RouteInfo> {
        let routes = self.routes.read().await;
        routes
            .values()
            .filter(|r| r.method == method)
            .cloned()
            .collect()
    }

    /// Get protected routes (those requiring authentication)
    pub async fn get_protected_routes(&self) -> Vec<RouteInfo> {
        let routes = self.routes.read().await;
        routes
            .values()
            .filter(|r| r.auth_required)
            .cloned()
            .collect()
    }

    /// Get public routes (no authentication required)
    pub async fn get_public_routes(&self) -> Vec<RouteInfo> {
        let routes = self.routes.read().await;
        routes
            .values()
            .filter(|r| !r.auth_required)
            .cloned()
            .collect()
    }

    /// Get rate-limited routes
    pub async fn get_rate_limited_routes(&self) -> Vec<RouteInfo> {
        let routes = self.routes.read().await;
        routes
            .values()
            .filter(|r| r.rate_limit.is_some())
            .cloned()
            .collect()
    }

    /// Get route count
    pub async fn count(&self) -> usize {
        self.routes.read().await.len()
    }

    /// Clear all routes
    pub async fn clear(&self) {
        self.routes.write().await.clear();
    }

    /// Check if a route exists
    pub async fn exists(&self, method: &str, path: &str) -> bool {
        let routes = self.routes.read().await;
        let key = format!("{} {}", method, path);
        routes.contains_key(&key)
    }

    /// Get route information with discovery stats
    pub async fn get_discovery_info(&self) -> DiscoveryStats {
        let routes = self.routes.read().await;
        let total = routes.len();
        let protected = routes.values().filter(|r| r.auth_required).count();
        let public = total - protected;
        let rate_limited = routes.values().filter(|r| r.rate_limit.is_some()).count();

        // Count routes by method
        let mut methods = HashMap::new();
        for route in routes.values() {
            *methods.entry(route.method.clone()).or_insert(0) += 1;
        }

        DiscoveryStats {
            total_routes: total,
            public_routes: public,
            protected_routes: protected,
            rate_limited_routes: rate_limited,
            routes_by_method: methods,
        }
    }
}

impl Default for RouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for shared route registry
pub type SharedRouteRegistry = Arc<RouteRegistry>;

/// Discovery statistics
#[derive(Debug, Clone)]
pub struct DiscoveryStats {
    /// Total Routes
    pub total_routes: usize,
    /// Public Routes
    pub public_routes: usize,
    /// Protected Routes
    pub protected_routes: usize,
    /// Rate Limited Routes
    pub rate_limited_routes: usize,
    /// Map of routes_by_method entries
    pub routes_by_method: HashMap<String, usize>,
}

/// Build the standard route registry with all known routes
pub async fn build_standard_routes() -> SharedRouteRegistry {
    let registry = Arc::new(RouteRegistry::new());

    let routes = vec![
        // === Public API Routes (no auth) ===
        RouteInfo {
            id: "route_1".to_string(),
            path: "/api/health".to_string(),
            method: "GET".to_string(),
            handler: "health_handler".to_string(),
            auth_required: false,
            rate_limit: None,
        },
        RouteInfo {
            id: "route_2".to_string(),
            path: "/api/context/metrics".to_string(),
            method: "GET".to_string(),
            handler: "comprehensive_metrics_handler".to_string(),
            auth_required: false,
            rate_limit: None,
        },
        RouteInfo {
            id: "route_3".to_string(),
            path: "/api/context/status".to_string(),
            method: "GET".to_string(),
            handler: "status_handler".to_string(),
            auth_required: false,
            rate_limit: None,
        },
        RouteInfo {
            id: "route_4".to_string(),
            path: "/api/context/search".to_string(),
            method: "POST".to_string(),
            handler: "search_handler".to_string(),
            auth_required: false,
            rate_limit: Some(admin_defaults::DEFAULT_ROUTE_RATE_LIMIT_SEARCH),
        },
        RouteInfo {
            id: "route_5".to_string(),
            path: "/api/context/index".to_string(),
            method: "POST".to_string(),
            handler: "index_handler".to_string(),
            auth_required: false,
            rate_limit: Some(admin_defaults::DEFAULT_ROUTE_RATE_LIMIT_INDEXING),
        },
        RouteInfo {
            id: "route_6".to_string(),
            path: "/api/context/indexing-status".to_string(),
            method: "GET".to_string(),
            handler: "indexing_status_handler".to_string(),
            auth_required: false,
            rate_limit: None,
        },
        // Admin Public Routes
        RouteInfo {
            id: "route_7".to_string(),
            path: "/admin/auth/login".to_string(),
            method: "POST".to_string(),
            handler: "login_handler".to_string(),
            auth_required: false,
            rate_limit: Some(admin_defaults::DEFAULT_ROUTE_RATE_LIMIT_INDEXING),
        },
        RouteInfo {
            id: "route_8".to_string(),
            path: "/admin/auth/logout".to_string(),
            method: "POST".to_string(),
            handler: "logout_handler".to_string(),
            auth_required: false,
            rate_limit: None,
        },
        RouteInfo {
            id: "route_9".to_string(),
            path: "/admin/status".to_string(),
            method: "GET".to_string(),
            handler: "get_status_handler".to_string(),
            auth_required: false,
            rate_limit: None,
        },
        RouteInfo {
            id: "route_10".to_string(),
            path: "/admin/dashboard/metrics".to_string(),
            method: "GET".to_string(),
            handler: "get_dashboard_metrics_handler".to_string(),
            auth_required: false,
            rate_limit: None,
        },
        RouteInfo {
            id: "route_11".to_string(),
            path: "/admin/diagnostic/health".to_string(),
            method: "GET".to_string(),
            handler: "health_check_handler".to_string(),
            auth_required: false,
            rate_limit: None,
        },
        RouteInfo {
            id: "route_12".to_string(),
            path: "/admin/config".to_string(),
            method: "GET".to_string(),
            handler: "get_config_handler".to_string(),
            auth_required: true,
            rate_limit: Some(admin_defaults::DEFAULT_ROUTE_RATE_LIMIT_ADMIN),
        },
        RouteInfo {
            id: "route_13".to_string(),
            path: "/admin/config".to_string(),
            method: "PUT".to_string(),
            handler: "update_config_handler".to_string(),
            auth_required: true,
            rate_limit: Some(admin_defaults::DEFAULT_ROUTE_RATE_LIMIT_RELOAD),
        },
        RouteInfo {
            id: "route_14".to_string(),
            path: "/admin/routes".to_string(),
            method: "GET".to_string(),
            handler: "get_routes_handler".to_string(),
            auth_required: true,
            rate_limit: Some(admin_defaults::DEFAULT_ROUTE_RATE_LIMIT_ADMIN),
        },
        RouteInfo {
            id: "route_15".to_string(),
            path: "/mcp".to_string(),
            method: "POST".to_string(),
            handler: "mcp_message_handler".to_string(),
            auth_required: false,
            rate_limit: Some(admin_defaults::DEFAULT_ROUTE_RATE_LIMIT_HEALTH),
        },
        RouteInfo {
            id: "route_16".to_string(),
            path: "/mcp/sse".to_string(),
            method: "GET".to_string(),
            handler: "mcp_sse_handler".to_string(),
            auth_required: false,
            rate_limit: Some(admin_defaults::DEFAULT_ROUTE_RATE_LIMIT_INDEXING),
        },
    ];

    registry.register_bulk(routes).await;
    registry
}
