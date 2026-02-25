//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Admin Service Implementations
//!
//! Real and null implementations of admin port traits.

use dashmap::DashMap;
use mcb_domain::ports::{IndexingOperation, IndexingOperationStatus, IndexingOperationsInterface};
use mcb_domain::value_objects::{CollectionId, OperationId};
use std::collections::HashMap;
use std::sync::Arc;
// ============================================================================
// Indexing Operations - Real Implementation
// ============================================================================

/// Default indexing operations tracker
///
/// Thread-safe implementation using `DashMap` for concurrent access.
pub struct DefaultIndexingOperations {
    /// Active indexing operations by ID
    operations: Arc<DashMap<OperationId, IndexingOperation>>,
}

impl DefaultIndexingOperations {
    /// Create a new indexing operations tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            operations: Arc::new(DashMap::new()),
        }
    }

    /// Create as Arc for sharing
    #[must_use]
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Start tracking a new indexing operation (inherent impl; trait delegates here).
    #[must_use]
    pub fn start_operation_internal(
        &self,
        collection: &CollectionId,
        total_files: usize,
    ) -> OperationId {
        let id = OperationId::new();
        let operation = IndexingOperation {
            id,
            collection: *collection,
            current_file: None,
            status: IndexingOperationStatus::Starting,
            total_files,
            processed_files: 0,
            started_at: chrono::Utc::now().timestamp(),
        };
        self.operations.insert(id, operation);
        id
    }

    /// Update progress for an operation
    pub fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
    ) {
        if let Some(mut op) = self.operations.get_mut(operation_id) {
            op.current_file = current_file;
            op.processed_files = processed;
        }
    }

    /// Complete and remove an operation
    pub fn complete_operation(&self, operation_id: &OperationId) {
        self.operations.remove(operation_id);
    }

    /// Check if any operations are in progress
    #[must_use]
    pub fn has_active_operations(&self) -> bool {
        !self.operations.is_empty()
    }

    /// Get count of active operations
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.operations.len()
    }
}

impl Default for DefaultIndexingOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexingOperationsInterface for DefaultIndexingOperations {
    fn get_operations(&self) -> HashMap<OperationId, IndexingOperation> {
        self.operations
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    fn start_operation(&self, collection: &CollectionId, total_files: usize) -> OperationId {
        self.start_operation_internal(collection, total_files)
    }

    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
    ) {
        DefaultIndexingOperations::update_progress(self, operation_id, current_file, processed);
    }

    fn complete_operation(&self, operation_id: &OperationId) {
        DefaultIndexingOperations::complete_operation(self, operation_id);
    }
}
