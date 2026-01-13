//! Server DI Module Implementation
//!
//! Contains MCP server metrics and indexing operations.

use shaku::module;

use super::traits::ServerModule;
use crate::server::metrics::McpPerformanceMetrics;
use crate::server::operations::McpIndexingOperations;

module! {
    pub ServerModuleImpl: ServerModule {
        components = [McpPerformanceMetrics, McpIndexingOperations],
        providers = []
    }
}
