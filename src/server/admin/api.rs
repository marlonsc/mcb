//! Admin API integration

use axum::Router;
use std::sync::Arc;

use crate::admin::{models::AdminState, routes::create_admin_router, AdminApi, AdminConfig};

/// Admin API server
pub struct AdminApiServer {
    config: AdminConfig,
    mcp_server: Arc<crate::server::McpServer>,
}

impl AdminApiServer {
    /// Create a new admin API server
    pub fn new(config: AdminConfig, mcp_server: Arc<crate::server::McpServer>) -> Self {
        Self { config, mcp_server }
    }

    /// Create the admin router
    pub fn create_router(&self) -> Result<Router, Box<dyn std::error::Error>> {
        let admin_api = Arc::new(AdminApi::new(self.config.clone()));
        let admin_service = self.mcp_server.admin_service();

        // Initialize web interface and templates
        let web_interface = crate::admin::web::WebInterface::new()?;
        let templates = web_interface.templates();

        // Create activity logger for tracking system events
        let activity_logger =
            Arc::new(crate::admin::service::helpers::activity::ActivityLogger::new());
        // Start listening to system events
        activity_logger.start_listening(self.mcp_server.event_bus.clone());

        let state = AdminState {
            admin_api,
            admin_service,
            mcp_server: Arc::clone(&self.mcp_server),
            templates,
            recovery_manager: None, // Will be set during Phase 8 integration
            event_bus: self.mcp_server.event_bus.clone(),
            activity_logger,
        };

        let api_router = create_admin_router(state.clone());
        let web_router = web_interface.routes(state);

        Ok(Router::new().merge(api_router).merge(web_router))
    }

    /// Get admin configuration
    pub fn config(&self) -> &AdminConfig {
        &self.config
    }
}
