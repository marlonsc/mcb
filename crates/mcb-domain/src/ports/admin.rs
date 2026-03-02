//! Administrative interfaces for system management and monitoring.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::registry::embedding::EmbeddingProviderConfig;
use crate::registry::language::LanguageProviderConfig;
use crate::registry::vector_store::VectorStoreProviderConfig;
use crate::value_objects::{CollectionId, OperationId};

// ============================================================================
// Dashboard Query Port
// ============================================================================

/// Response DTO for monthly observation counts
#[derive(Debug, Clone)]
pub struct MonthlyCount {
    /// Month identifier (e.g., "2025-02")
    pub month: String,
    /// Number of observations in this month
    pub count: i64,
}

/// Response DTO for daily observation counts
#[derive(Debug, Clone)]
pub struct DailyCount {
    /// Day identifier (e.g., "2025-02-25")
    pub day: String,
    /// Number of observations on this day
    pub count: i64,
}

/// Response DTO for tool call counts
#[derive(Debug, Clone)]
pub struct ToolCallCount {
    /// Name of the tool
    pub tool_name: String,
    /// Number of times this tool was called
    pub count: i64,
}

/// Response DTO for agent session statistics
#[derive(Debug, Clone)]
pub struct AgentSessionStats {
    /// Total number of sessions
    pub total_sessions: i64,
    /// Total number of unique agents
    pub total_agents: i64,
}

/// Port for dashboard/admin queries
///
/// Provides read-only access to aggregated analytics and statistics
/// for dashboard and admin UI consumption.
#[async_trait]
pub trait DashboardQueryPort: Send + Sync {
    /// Get observations aggregated by month
    async fn get_observations_by_month(&self, limit: usize) -> Result<Vec<MonthlyCount>>;

    /// Get observations aggregated by day
    async fn get_observations_by_day(&self, limit: usize) -> Result<Vec<DailyCount>>;

    /// Get tool call counts
    async fn get_tool_call_counts(&self) -> Result<Vec<ToolCallCount>>;

    /// Get agent session statistics
    async fn get_agent_session_stats(&self) -> Result<AgentSessionStats>;
}

// ============================================================================
// Indexing Operations
// ============================================================================

/// Status of an indexing operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IndexingOperationStatus {
    /// Operation is starting
    Starting,
    /// Operation is in progress
    InProgress,
    /// Operation has completed successfully
    Completed,
    /// Operation has failed with an error message
    Failed(String),
}

/// Data about an ongoing indexing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingOperation {
    /// Unique identifier for the operation
    pub id: OperationId,
    /// Target collection for indexing
    pub collection: CollectionId,
    /// Current status of the operation
    pub status: IndexingOperationStatus,
    /// Total number of files to index
    pub total_files: usize,
    /// Number of files processed so far
    pub processed_files: usize,
    /// Current file being processed, if any
    pub current_file: Option<String>,
    /// Timestamp when the operation started
    pub started_at: i64,
}

/// Interface for tracking indexing operations
pub trait IndexingOperationsInterface: Send + Sync {
    /// Get all tracked indexing operations
    fn get_operations(&self) -> HashMap<OperationId, IndexingOperation>;
    /// Start a new indexing operation
    fn start_operation(&self, collection: &CollectionId, total_files: usize) -> OperationId;
    /// Update progress of an operation
    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
    );
    /// Mark an operation as completed
    fn complete_operation(&self, operation_id: &OperationId);
}

// ============================================================================
// Validation Operations
// ============================================================================

/// Status of a validation operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    Queued,
    InProgress,
    Completed,
    Failed(String),
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
    /// Get all tracked validation operations
    fn get_operations(&self) -> HashMap<OperationId, ValidationOperation>;
    /// Get a specific validation operation by ID
    fn get_operation(&self, operation_id: &OperationId) -> Option<ValidationOperation>;
    /// Start a new validation operation
    fn start_operation(&self, workspace: &str, validators: &[String]) -> OperationId;
    /// Update progress of an operation
    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
        total: usize,
    );
    /// Mark an operation as completed with its result
    fn complete_operation(&self, operation_id: &OperationId, result: ValidationOperationResult);
    /// Cancel a running validation operation
    fn cancel_operation(&self, operation_id: &OperationId);
    /// Check if a validation operation is currently in progress
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

// ============================================================================
// Provider Admin
// ============================================================================

/// Information about an available provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Unique name identifier for the provider
    pub name: String,
    /// Human-readable description of the provider
    pub description: String,
}

impl ProviderInfo {
    /// Create a new ProviderInfo instance
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

crate::provider_admin_interface!(
    /// Interface for embedding provider admin operations.
    trait EmbeddingAdminInterface,
    config = EmbeddingProviderConfig,
    list_doc = "List all available embedding providers.",
    extra = {
        /// Get current provider name.
        fn current_provider(&self) -> String;
    }
);

crate::provider_admin_interface!(
    /// Interface for vector store provider admin operations.
    trait VectorStoreAdminInterface,
    config = VectorStoreProviderConfig,
    list_doc = "List all available vector store providers.",
    extra = {}
);

crate::provider_admin_interface!(
    /// Interface for language provider admin operations.
    trait LanguageAdminInterface,
    config = LanguageProviderConfig,
    list_doc = "List all available language providers.",
    extra = {}
);
