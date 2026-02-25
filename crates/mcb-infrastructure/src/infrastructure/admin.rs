//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Admin Service Implementations
//!
//! Real and null implementations of admin port traits.

use dashmap::DashMap;
use mcb_domain::ports::{
    IndexingOperation, IndexingOperationStatus, IndexingOperationsInterface, ValidationOperation,
    ValidationOperationResult, ValidationOperationsInterface, ValidationStatus,
};
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

// ============================================================================
// Validation Operations - Real Implementation
// ============================================================================

/// Default validation operations tracker (in-memory).
///
/// Thread-safe implementation using `DashMap` for concurrent access.
pub struct DefaultValidationOperations {
    operations: Arc<DashMap<OperationId, ValidationOperation>>,
}

impl DefaultValidationOperations {
    /// Create a new validation operations tracker
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
}

impl Default for DefaultValidationOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationOperationsInterface for DefaultValidationOperations {
    fn get_operations(&self) -> HashMap<OperationId, ValidationOperation> {
        self.operations
            .iter()
            .map(|e| (*e.key(), e.value().clone()))
            .collect()
    }

    fn get_operation(&self, operation_id: &OperationId) -> Option<ValidationOperation> {
        self.operations.get(operation_id).map(|r| r.clone())
    }

    fn start_operation(&self, workspace: &str, validators: &[String]) -> OperationId {
        let id = OperationId::new();
        let op = ValidationOperation {
            id,
            workspace: workspace.to_owned(),
            status: ValidationStatus::Queued,
            validators: validators.to_vec(),
            progress_percent: 0,
            current_file: None,
            processed_items: 0,
            total_items: 0,
            started_at: chrono::Utc::now().timestamp(),
            completed_at: None,
            result: None,
        };
        self.operations.insert(id, op);
        id
    }

    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
        total: usize,
    ) {
        if let Some(mut op) = self.operations.get_mut(operation_id) {
            op.current_file = current_file;
            op.processed_items = processed;
            op.total_items = total;
            op.progress_percent = if total > 0 {
                ((processed as f64 / total as f64) * 100.0).min(100.0) as u8
            } else {
                0
            };
            if op.status == ValidationStatus::Queued {
                op.status = ValidationStatus::InProgress;
            }
        }
    }

    fn complete_operation(&self, operation_id: &OperationId, result: ValidationOperationResult) {
        if let Some(mut op) = self.operations.get_mut(operation_id) {
            op.status = ValidationStatus::Completed;
            op.progress_percent = 100;
            op.completed_at = Some(chrono::Utc::now().timestamp());
            op.result = Some(result);
        }
    }

    fn cancel_operation(&self, operation_id: &OperationId) {
        if let Some(mut op) = self.operations.get_mut(operation_id) {
            op.status = ValidationStatus::Canceled;
            op.completed_at = Some(chrono::Utc::now().timestamp());
        }
    }

    fn is_in_progress(&self, operation_id: &OperationId) -> bool {
        self.operations.get(operation_id).is_some_and(|op| {
            matches!(
                op.status,
                ValidationStatus::Queued | ValidationStatus::InProgress
            )
        })
    }
}
