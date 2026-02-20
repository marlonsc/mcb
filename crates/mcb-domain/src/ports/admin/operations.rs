//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md)
//!
//! Operations Tracking Port Definitions
//!
//! Interfaces for tracking performance metrics, indexing operations,
//! and validation operations.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::value_objects::{CollectionId, OperationId};

// ============================================================================
// Performance Metrics Types
// ============================================================================

/// Data structure for detailed performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetricsData {
    /// Total queries processed
    pub total_queries: u64,
    /// Total successful queries
    pub successful_queries: u64,
    /// Number of failed queries
    pub failed_queries: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Number of currently active connections
    pub active_connections: usize,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
}

/// Interface for tracking performance metrics
pub trait PerformanceMetricsInterface: Send + Sync {
    /// Get server uptime in seconds
    fn uptime_secs(&self) -> u64;
    /// Record a query execution
    fn record_query(&self, response_time_ms: u64, success: bool, cache_hit: bool);
    /// Update active connections count
    fn update_active_connections(&self, delta: i64);
    /// Get current performance metrics
    fn get_performance_metrics(&self) -> PerformanceMetricsData;
}

// ============================================================================
// Indexing Operations Types
// ============================================================================

/// Status of an indexing operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IndexingOperationStatus {
    /// Indicates the operation is initializing.
    Starting,
    /// Indicates the operation is currently running.
    InProgress,
    /// Indicates the operation finished successfully.
    Completed,
    /// Indicates the operation failed with an error message.
    Failed(String),
}

/// Data about an ongoing indexing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingOperation {
    /// Unique identifier for the operation.
    pub id: OperationId,
    /// Identifier of the collection being indexed.
    pub collection: CollectionId,
    /// Current status of the operation.
    pub status: IndexingOperationStatus,
    /// Total number of files to process.
    pub total_files: usize,
    /// Number of files processed so far.
    pub processed_files: usize,
    /// Path of the file currently being processed.
    pub current_file: Option<String>,
    /// Timestamp when the operation started (Unix epoch seconds).
    pub started_at: i64,
}

/// Interface for tracking indexing operations
pub trait IndexingOperationsInterface: Send + Sync {
    /// Get all tracking operations
    fn get_operations(&self) -> HashMap<OperationId, IndexingOperation>;
    /// Start tracking a new indexing operation
    fn start_operation(&self, collection: &CollectionId, total_files: usize) -> OperationId;
    /// Update progress of an operation
    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
    );
    /// Mark operation as completed
    fn complete_operation(&self, operation_id: &OperationId);
}

// ============================================================================
// Validation Operations Types
// ============================================================================

/// Status of a validation operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Indicates the operation is waiting to start.
    Queued,
    /// Indicates the operation is currently executing.
    InProgress,
    /// Indicates the operation finished.
    Completed,
    /// Indicates the operation failed with an error.
    Failed(String),
    /// Indicates the operation was manually stopped.
    Canceled,
}

/// Result metadata for a completed validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOperationResult {
    /// Total number of rule violations found.
    pub total_violations: usize,
    /// Number of error-level violations.
    pub errors: usize,
    /// Number of warning-level violations.
    pub warnings: usize,
    /// Whether the validation passed acceptance criteria.
    pub passed: bool,
}

/// Data about an ongoing validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOperation {
    /// Unique identifier for the operation.
    pub id: OperationId,
    /// Target workspace or directory being validated.
    pub workspace: String,
    /// Current status of the operation.
    pub status: ValidationStatus,
    /// List of validator names enabled for this operation.
    pub validators: Vec<String>,
    /// Progress percentage (0-100).
    pub progress_percent: u8,
    /// Path of the file currently being validated.
    pub current_file: Option<String>,
    /// Number of items processed so far.
    pub processed_items: usize,
    /// Total number of items to process.
    pub total_items: usize,
    /// Timestamp when the operation started (Unix epoch seconds).
    pub started_at: i64,
    /// Timestamp when the operation completed (Unix epoch seconds, if applicable).
    pub completed_at: Option<i64>,
    /// Final result of the validation (if completed).
    pub result: Option<ValidationOperationResult>,
}

/// Interface for tracking validation operations
pub trait ValidationOperationsInterface: Send + Sync {
    /// Get all validation operations
    fn get_operations(&self) -> HashMap<OperationId, ValidationOperation>;
    /// Get a specific operation
    fn get_operation(&self, operation_id: &OperationId) -> Option<ValidationOperation>;
    /// Start a new validation operation
    fn start_operation(&self, workspace: &str, validators: &[String]) -> OperationId;
    /// Update progress
    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
        total: usize,
    );
    /// Mark as completed
    fn complete_operation(&self, operation_id: &OperationId, result: ValidationOperationResult);
    /// Cancel an operation
    fn cancel_operation(&self, operation_id: &OperationId);
    /// Check if in progress
    fn is_in_progress(&self, operation_id: &OperationId) -> bool;
}
