//! HTTP Transport for MCP
//!
#![allow(clippy::redundant_type_annotations)]
//! Implements MCP protocol over HTTP using Server-Sent Events (SSE).
//! This transport allows web clients to connect to the MCP server.
//!
//! # Architecture
//!
//! This transport consolidates all HTTP endpoints into a single port:
//! - MCP protocol endpoints (`/mcp`, `/events`)
//! - Health/readiness probes (`/healthz`, `/readyz`)
//! - Admin API endpoints (`/health`, `/config`, `/collections`, etc.)
//! - Prometheus metrics (`/metrics`)
//!
//! # Supported MCP Methods
//!
//! | Method | Description |
//! | -------- | ------------- |
//! | `initialize` | Initialize the MCP session |
//! | `tools/list` | List available tools |
//! | `tools/call` | Call a tool with arguments |
//! | `ping` | Health check |
//!
//! # Example
//!
//! ```text
//! POST /mcp HTTP/1.1
//! Content-Type: application/json
//!
//! {
//!     "jsonrpc": "2.0",
//!     "method": "tools/list",
//!     "id": 1
//! }
//! ```
//!
//! # Migration Note
//! Consolidated Admin API into single port in v0.2.0.

use std::sync::Arc;

use rocket::{Build, Rocket};
use tracing::info;

use crate::McpServer;
use crate::admin::auth::AdminAuthConfig;
use crate::admin::browse_handlers::BrowseState;
use crate::admin::handlers::AdminState;
use crate::admin::routes::admin_rocket;

#[path = "http/http_bridge.rs"]
mod http_bridge;
#[path = "http/http_config.rs"]
mod http_config;
#[path = "http/http_cors.rs"]
mod http_cors;
#[path = "http/http_health.rs"]
mod http_health;
#[path = "http/http_mcp.rs"]
mod http_mcp;
#[path = "http/http_mcp_tools.rs"]
mod http_mcp_tools;
pub use http_config::HttpTransportConfig;

/// Shared state for HTTP transport
#[derive(Clone)]
pub struct HttpTransportState {
    /// Shared reference to the MCP server instance
    pub server: Arc<McpServer>,
}

/// HTTP transport server with optional admin API integration
pub struct HttpTransport {
    config: HttpTransportConfig,
    state: HttpTransportState,
    admin_state: Option<AdminState>,
    auth_config: Option<Arc<AdminAuthConfig>>,
    browse_state: Option<BrowseState>,
}

impl HttpTransport {
    /// Create a new HTTP transport
    #[must_use]
    pub fn new(config: HttpTransportConfig, server: Arc<McpServer>) -> Self {
        Self {
            config,
            state: HttpTransportState { server },
            admin_state: None,
            auth_config: None,
            browse_state: None,
        }
    }

    /// Add admin API state for consolidated single-port operation
    #[must_use]
    pub fn with_admin(
        mut self,
        admin_state: AdminState,
        auth_config: Arc<AdminAuthConfig>,
        browse_state: Option<BrowseState>,
    ) -> Self {
        self.admin_state = Some(admin_state);
        self.auth_config = Some(auth_config);
        self.browse_state = browse_state;
        self
    }

    /// Build the Rocket application with MCP and optional Admin routes.
    ///
    /// Delegates all admin/web routes to [`admin_rocket()`] as the single source
    /// of truth, then layers MCP-specific routes on top.
    #[must_use]
    pub fn rocket(&self) -> Rocket<Build> {
        let mut rocket = if let Some(ref admin_state) = self.admin_state {
            let auth_config = self
                .auth_config
                .clone()
                .unwrap_or_else(|| Arc::new(AdminAuthConfig::default()));
            admin_rocket(admin_state.clone(), auth_config, self.browse_state.clone())
        } else {
            rocket::custom(rocket::Config::figment())
        };

        rocket = rocket.manage(self.state.clone()).mount(
            "/",
            rocket::routes![
                http_mcp::handle_mcp_request,
                http_health::healthz,
                http_health::readyz
            ],
        );

        if self.config.enable_cors {
            rocket = rocket.attach(http_cors::Cors);
        }

        rocket
    }

    /// Start the HTTP transport server
    ///
    /// # Errors
    /// Returns an error when socket resolution or Rocket launch fails.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = self.config.socket_addr()?;
        info!("HTTP transport listening on {}", addr);

        let figment = rocket::Config::figment()
            .merge(("address", self.config.host.clone()))
            .merge(("port", self.config.port));

        let rocket = self.rocket().configure(figment);

        rocket
            .launch()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    /// Start with graceful shutdown
    ///
    /// Note: Rocket handles graceful shutdown internally via Ctrl+C.
    ///
    /// # Errors
    /// Returns an error when starting the transport fails.
    pub async fn start_with_shutdown(
        self,
        _shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Rocket handles graceful shutdown internally
        self.start().await
    }
}
