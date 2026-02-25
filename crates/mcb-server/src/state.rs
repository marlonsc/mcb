//! Application state for MCB server controllers.
//!
//! **Documentation**: [docs/modules/server.md](../../../docs/modules/server.md)
//!
//! [`McbState`] contains only domain port references and is framework-agnostic.
//! It will be injected into Axum handlers via Extension in Wave 4.

use std::sync::Arc;

use mcb_domain::ports::{AuthRepositoryPort, DashboardQueryPort};

use crate::mcp_server::McpServer;

/// Application state for MCB server controllers.
///
/// Contains only domain port references - no framework types (`AppContext`, `DatabaseConnection`, `loco_rs`).
/// This struct is designed to be injected into Axum handlers via Extension.
///
/// # Architecture
/// - Follows Clean Architecture: domain ports only, no framework dependencies
/// - Cloneable for use with Axum Extension
/// - Provides access to core domain operations through port interfaces
#[derive(Clone)]
pub struct McbState {
    /// Dashboard query port for admin/statistics operations
    pub dashboard: Arc<dyn DashboardQueryPort>,
    /// Auth repository port for API key verification
    pub auth_repo: Arc<dyn AuthRepositoryPort>,
    /// MCP server instance for tool execution
    pub mcp_server: Arc<McpServer>,
}

impl McbState {
    /// Create new `McbState` with all required ports.
    ///
    /// # Arguments
    /// * `dashboard` - Dashboard query port for admin operations
    /// * `auth_repo` - Auth repository port for API key verification
    /// * `mcp_server` - MCP server instance
    ///
    /// # Returns
    /// A new `McbState` instance ready for injection into handlers
    #[must_use]
    pub fn new(
        dashboard: Arc<dyn DashboardQueryPort>,
        auth_repo: Arc<dyn AuthRepositoryPort>,
        mcp_server: Arc<McpServer>,
    ) -> Self {
        Self {
            dashboard,
            auth_repo,
            mcp_server,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mcb_state_creation() {
        // Tests will be added when mock ports are available
        // This placeholder ensures the module compiles
    }
}
