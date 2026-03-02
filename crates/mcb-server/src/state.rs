//! Application state for MCB server controllers.
//!
//! **Documentation**: [docs/modules/server.md](../../../docs/modules/server.md)
//!
//! [`McbState`] contains only domain port references and is framework-agnostic.
//! It will be injected into Axum handlers via Extension in Wave 4.

use std::sync::Arc;

use mcb_domain::ports::{
    AuthRepositoryPort, DashboardQueryPort, EmbeddingProvider, IndexingOperationsInterface,
    ValidationOperationsInterface, VectorStoreProvider,
};

use crate::mcp_server::McpServer;

/// Result of MCP server composition: server plus ports for dashboard/auth.
///
/// Built by the Loco bridge so all Loco-exposed services go through centralized DI.
#[derive(Clone)]
pub struct McpServerBootstrap {
    /// MCP server instance
    pub mcp_server: Arc<McpServer>,
    /// Dashboard query port (built via bridge from Loco DB)
    pub dashboard: Arc<dyn DashboardQueryPort>,
    /// Auth repository port (built via bridge from Loco DB)
    pub auth_repo: Arc<dyn AuthRepositoryPort>,
    /// Shared embedding provider for health checks and metadata (single-resolution DI)
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    /// Shared vector store provider for collections and health (single-resolution DI)
    pub vector_store: Arc<dyn VectorStoreProvider>,
    /// Shared indexing operations tracker for jobs admin (single-resolution DI)
    pub indexing_ops: Arc<dyn IndexingOperationsInterface>,
    /// Shared validation operations tracker for jobs admin (single-resolution DI)
    pub validation_ops: Arc<dyn ValidationOperationsInterface>,
}

impl McpServerBootstrap {
    /// Build [`McbState`] from this bootstrap (for use in route layers).
    #[must_use]
    pub fn into_mcb_state(self) -> McbState {
        McbState::new(
            self.dashboard,
            self.auth_repo,
            self.mcp_server,
            self.embedding_provider,
            self.vector_store,
            self.indexing_ops,
            self.validation_ops,
        )
    }
}

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
    /// Shared embedding provider for health checks and metadata
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    /// Shared vector store provider for collections and health
    pub vector_store: Arc<dyn VectorStoreProvider>,
    /// Shared indexing operations tracker for jobs admin
    pub indexing_ops: Arc<dyn IndexingOperationsInterface>,
    /// Shared validation operations tracker for jobs admin
    pub validation_ops: Arc<dyn ValidationOperationsInterface>,
}

impl McbState {
    /// Create new `McbState` with all required ports.
    ///
    /// # Arguments
    /// * `dashboard` - Dashboard query port for admin operations
    /// * `auth_repo` - Auth repository port for API key verification
    /// * `mcp_server` - MCP server instance
    /// * `embedding_provider` - Shared embedding provider for health/metadata
    /// * `vector_store` - Shared vector store provider for collections/health
    /// * `indexing_ops` - Shared indexing operations tracker
    /// * `validation_ops` - Shared validation operations tracker
    ///
    /// # Returns
    /// A new `McbState` instance ready for injection into handlers
    #[must_use]
    pub fn new(
        dashboard: Arc<dyn DashboardQueryPort>,
        auth_repo: Arc<dyn AuthRepositoryPort>,
        mcp_server: Arc<McpServer>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store: Arc<dyn VectorStoreProvider>,
        indexing_ops: Arc<dyn IndexingOperationsInterface>,
        validation_ops: Arc<dyn ValidationOperationsInterface>,
    ) -> Self {
        Self {
            dashboard,
            auth_repo,
            mcp_server,
            embedding_provider,
            vector_store,
            indexing_ops,
            validation_ops,
        }
    }
}
