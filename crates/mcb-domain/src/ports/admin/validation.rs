//! Validation operation tracking ports.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::value_objects::OperationId;

/// Status of a validation operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Validation is queued and waiting for execution
    Queued,
    /// Validation is currently running
    InProgress,
    /// Validation has finished successfully
    Completed,
    /// Validation has failed with an error
    Failed(String),
    /// Validation was manually canceled
    Canceled,
}

/// Result metadata for a completed validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOperationResult {
    /// Total number of violations found
    pub total_violations: usize,
    /// Number of error-level violations
    pub errors: usize,
    /// Number of warning-level violations
    pub warnings: usize,
    /// Whether the validation passed overall
    pub passed: bool,
}

/// Data about an ongoing validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOperation {
    /// Unique identifier for the operation
    pub id: OperationId,
    /// Path or identifier of the workspace being validated
    pub workspace: String,
    /// Current status of the validation
    pub status: ValidationStatus,
    /// List of validators being executed
    pub validators: Vec<String>,
    /// Progress as a percentage (0-100)
    pub progress_percent: u8,
    /// Current file being validated, if any
    pub current_file: Option<String>,
    /// Number of items processed so far
    pub processed_items: usize,
    /// Total number of items to validate
    pub total_items: usize,
    /// Timestamp when validation started
    pub started_at: i64,
    /// Timestamp when validation completed, if finished
    pub completed_at: Option<i64>,
    /// Final result of the validation, if finished
    pub result: Option<ValidationOperationResult>,
}

/// Interface for tracking validation operations
pub trait ValidationOperationsInterface: Send + Sync {
    /// Get all tracked validation operations.
    fn get_operations(&self) -> HashMap<OperationId, ValidationOperation>;
    /// Get a specific validation operation by ID.
    fn get_operation(&self, operation_id: &OperationId) -> Option<ValidationOperation>;
    /// Start a new validation operation.
    fn start_operation(&self, workspace: &str, validators: &[String]) -> OperationId;
    /// Update progress of an operation.
    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
        total: usize,
    );
    /// Mark an operation as completed with its result.
    fn complete_operation(&self, operation_id: &OperationId, result: ValidationOperationResult);
    /// Cancel a running validation operation.
    fn cancel_operation(&self, operation_id: &OperationId);
    /// Check if a validation operation is currently in progress.
    fn is_in_progress(&self, operation_id: &OperationId) -> bool;
}

/// Port for submitting validation jobs to the execution infrastructure.
pub trait ValidatorJobRunner: Send + Sync {
    /// Submit a validation job for the given workspace and validators.
    ///
    /// # Errors
    /// Returns a domain error if the job cannot be submitted.
    fn submit_validation_job(&self, workspace: &str, validators: &[String]) -> Result<OperationId>;
}
