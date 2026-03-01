//! Administrative interfaces for system management and monitoring.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

#![allow(missing_docs)]

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
    Starting,
    InProgress,
    Completed,
    Failed(String),
}

/// Data about an ongoing indexing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingOperation {
    pub id: OperationId,
    pub collection: CollectionId,
    pub status: IndexingOperationStatus,
    pub total_files: usize,
    pub processed_files: usize,
    pub current_file: Option<String>,
    pub started_at: i64,
}

/// Interface for tracking indexing operations
pub trait IndexingOperationsInterface: Send + Sync {
    fn get_operations(&self) -> HashMap<OperationId, IndexingOperation>;
    fn start_operation(&self, collection: &CollectionId, total_files: usize) -> OperationId;
    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
    );
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
    pub total_violations: usize,
    pub errors: usize,
    pub warnings: usize,
    pub passed: bool,
}

/// Data about an ongoing validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOperation {
    pub id: OperationId,
    pub workspace: String,
    pub status: ValidationStatus,
    pub validators: Vec<String>,
    pub progress_percent: u8,
    pub current_file: Option<String>,
    pub processed_items: usize,
    pub total_items: usize,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub result: Option<ValidationOperationResult>,
}

/// Interface for tracking validation operations
pub trait ValidationOperationsInterface: Send + Sync {
    fn get_operations(&self) -> HashMap<OperationId, ValidationOperation>;
    fn get_operation(&self, operation_id: &OperationId) -> Option<ValidationOperation>;
    fn start_operation(&self, workspace: &str, validators: &[String]) -> OperationId;
    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
        total: usize,
    );
    fn complete_operation(&self, operation_id: &OperationId, result: ValidationOperationResult);
    fn cancel_operation(&self, operation_id: &OperationId);
    fn is_in_progress(&self, operation_id: &OperationId) -> bool;
}

/// Port for submitting validation jobs to the execution infrastructure.
pub trait ValidatorJobRunner: Send + Sync {
    /// Submit a validation job for the given workspace and validators.
    fn submit_validation_job(
        &self,
        workspace: &str,
        validators: &[String],
    ) -> std::result::Result<OperationId, String>;
}

// ============================================================================
// Provider Admin
// ============================================================================

/// Information about an available provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub description: String,
}

impl ProviderInfo {
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
