//! Indexing operations tracking for MCP server
//!
//! Provides interfaces and implementations for tracking ongoing
//! indexing operations in the MCP server.

use crate::infrastructure::service_helpers::UptimeTracker;
use dashmap::DashMap;
use shaku::{Component, Interface, ModuleBuildContext};

/// Interface for indexing operations tracking
pub trait IndexingOperationsInterface: Interface + Send + Sync {
    fn get_map(&self) -> &DashMap<String, IndexingOperation>;
}

/// Tracks ongoing indexing operations
#[derive(Debug, Clone)]
pub struct IndexingOperation {
    /// Operation ID
    pub id: String,
    /// Collection being indexed
    pub collection: String,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Total files to process
    pub total_files: usize,
    /// Files processed so far
    pub processed_files: usize,
    /// Operation start time tracker
    pub start_time: UptimeTracker,
}

/// Concrete implementation of indexing operations tracking
#[derive(Debug, Default)]
pub struct McpIndexingOperations {
    pub map: DashMap<String, IndexingOperation>,
}

impl<M: shaku::Module> Component<M> for McpIndexingOperations {
    type Interface = dyn IndexingOperationsInterface;
    type Parameters = ();

    fn build(
        _context: &mut ModuleBuildContext<M>,
        _params: Self::Parameters,
    ) -> Box<Self::Interface> {
        Box::new(Self::default())
    }
}

impl IndexingOperationsInterface for McpIndexingOperations {
    fn get_map(&self) -> &DashMap<String, IndexingOperation> {
        &self.map
    }
}
