//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Validation Operations — Default Implementation
//!
//! In-memory validation operations tracker using `DashMap` for thread-safe concurrent access.

use dashmap::DashMap;
use mcb_domain::ports::{
    ValidationOperation, ValidationOperationResult, ValidationOperationsInterface, ValidationStatus,
};
use mcb_domain::registry::admin_operations::{
    VALIDATION_OPERATIONS_PROVIDERS, ValidationOperationsProviderEntry,
};
use mcb_domain::value_objects::OperationId;
use std::collections::HashMap;
use std::sync::Arc;

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

#[linkme::distributed_slice(VALIDATION_OPERATIONS_PROVIDERS)]
static DEFAULT_VALIDATION_OPERATIONS_PROVIDER_ENTRY: ValidationOperationsProviderEntry =
    ValidationOperationsProviderEntry {
        name: "default",
        description: "In-memory validation operations tracker",
        build: |_config| Ok(Arc::new(DefaultValidationOperations::default())),
    };
