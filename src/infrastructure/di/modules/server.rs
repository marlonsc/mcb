//! Server DI Module Implementation
//!
//! Contains MCP server metrics and indexing operations.

#![allow(missing_docs)]

use shaku::module;

use super::traits::ServerModule;
use crate::infrastructure::operations::McpIndexingOperations;
use crate::server::metrics::McpPerformanceMetrics;

module! {
    pub ServerModuleImpl: ServerModule {
        components = [McpPerformanceMetrics, McpIndexingOperations],
        providers = []
    }
}
