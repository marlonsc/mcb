//! Admin Service Domain Ports
//!
//! Defines the port interfaces for admin and monitoring services.
//! These traits break the circular dependency where infrastructure/di
//! previously imported from server layer.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::registry::cache::CacheProviderConfig;
use crate::registry::embedding::EmbeddingProviderConfig;
use crate::registry::language::LanguageProviderConfig;
use crate::registry::vector_store::VectorStoreProviderConfig;
use crate::value_objects::{CollectionId, OperationId};

/// Information about an available provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider name (used in config)
    pub name: String,
    /// Human-readable description
    pub description: String,
}

impl ProviderInfo {
    /// Create new provider info
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

/// Interface for embedding provider admin operations
#[async_trait::async_trait]
pub trait EmbeddingAdminInterface: Send + Sync + std::fmt::Debug {
    /// List all available embedding providers
    fn list_providers(&self) -> Vec<ProviderInfo>;
    /// Get current provider name
    fn current_provider(&self) -> String;
    /// Switch to a different embedding provider
    fn switch_provider(&self, config: EmbeddingProviderConfig) -> Result<(), String>;
    /// Reload provider from current application config
    fn reload_from_config(&self) -> Result<(), String>;
}

/// Interface for vector store provider admin operations
#[async_trait::async_trait]
pub trait VectorStoreAdminInterface: Send + Sync + std::fmt::Debug {
    /// List all available vector store providers
    fn list_providers(&self) -> Vec<ProviderInfo>;
    /// Switch to a different vector store provider
    fn switch_provider(&self, config: VectorStoreProviderConfig) -> Result<(), String>;
    /// Reload provider from current application config
    fn reload_from_config(&self) -> Result<(), String>;
}

/// Interface for cache provider admin operations
#[async_trait::async_trait]
pub trait CacheAdminInterface: Send + Sync + std::fmt::Debug {
    /// List all available cache providers
    fn list_providers(&self) -> Vec<ProviderInfo>;
    /// Get current provider name
    fn current_provider(&self) -> String;
    /// Switch to a different cache provider
    fn switch_provider(&self, config: CacheProviderConfig) -> Result<(), String>;
    /// Reload provider from current application config
    fn reload_from_config(&self) -> Result<(), String>;
}

/// Interface for language provider admin operations
#[async_trait::async_trait]
pub trait LanguageAdminInterface: Send + Sync + std::fmt::Debug {
    /// List all available language providers
    fn list_providers(&self) -> Vec<ProviderInfo>;
    /// Switch to a different language provider
    fn switch_provider(&self, config: LanguageProviderConfig) -> Result<(), String>;
    /// Reload provider from current application config
    fn reload_from_config(&self) -> Result<(), String>;
}

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
// Indexing & Validation Operations Types
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

// ============================================================================
// Lifecycle & Health Types
// ============================================================================

/// Current state of a port service
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PortServiceState {
    /// Service is initializing.
    Starting,
    /// Service is fully operational.
    Running,
    /// Service is shutting down.
    Stopping,
    /// Service is stopped.
    #[default]
    Stopped,
}

/// Health status for a system dependency
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DependencyHealth {
    /// Dependency is operating normally.
    Healthy,
    /// Dependency is operating with reduced functionality or high latency.
    Degraded,
    /// Dependency is unavailable or malfunctioning.
    Unhealthy,
    /// Health status has not yet been determined.
    #[default]
    Unknown,
}

/// Health information for a system dependency
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependencyHealthCheck {
    /// Name of the dependency.
    pub name: String,
    /// Current health status.
    pub status: DependencyHealth,
    /// Optional status message or error description.
    pub message: Option<String>,
    /// Response latency in milliseconds (if applicable).
    pub latency_ms: Option<u64>,
    /// Timestamp of the last check (Unix epoch).
    pub last_check: u64,
}

/// Extended health response with detailed dependency info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedHealthResponse {
    /// Overall system status string.
    pub status: &'static str,
    /// System uptime in seconds.
    pub uptime_seconds: u64,
    /// Number of active indexing operations.
    pub active_indexing_operations: usize,
    /// List of health checks for individual dependencies.
    pub dependencies: Vec<DependencyHealthCheck>,
    /// Aggregated status of all dependencies.
    pub dependencies_status: DependencyHealth,
}

/// Interface for graceful shutdown coordination
pub trait ShutdownCoordinator: Send + Sync {
    /// Signals the system to initiate shutdown sequence.
    fn signal_shutdown(&self);
    /// Checks if a shutdown has been signaled.
    fn is_shutting_down(&self) -> bool;
}

/// Managed lifecycle for background services
#[async_trait::async_trait]
pub trait LifecycleManaged: Send + Sync {
    /// Returns the name of the service.
    fn name(&self) -> &str;
    /// Starts the service.
    async fn start(&self) -> crate::error::Result<()>;
    /// Stops the service.
    async fn stop(&self) -> crate::error::Result<()>;
    /// Restarts the service by stopping and then starting it.
    async fn restart(&self) -> crate::error::Result<()> {
        self.stop().await?;
        self.start().await
    }
    /// Returns the current state of the service.
    fn state(&self) -> PortServiceState;
    /// Performs a health check on the service.
    async fn health_check(&self) -> DependencyHealthCheck {
        DependencyHealthCheck {
            name: self.name().to_string(),
            status: match self.state() {
                PortServiceState::Running => DependencyHealth::Healthy,
                PortServiceState::Starting => DependencyHealth::Unknown,
                _ => DependencyHealth::Unhealthy,
            },
            message: None,
            latency_ms: None,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}
