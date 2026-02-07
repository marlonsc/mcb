//! Admin Service Domain Ports
//!
//! Defines the port interfaces for admin and monitoring services.
//! These traits break the circular dependency where infrastructure/di
//! previously imported from server layer.

use crate::registry::{
    CacheProviderConfig, EmbeddingProviderConfig, LanguageProviderConfig, VectorStoreProviderConfig,
};
use crate::value_objects::{CollectionId, OperationId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub enum IndexingStatus {
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
    pub status: IndexingStatus,
    pub total_files: usize,
    pub processed_files: usize,
    pub current_file: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
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
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PortServiceState {
    Starting,
    Running,
    Stopping,
    Stopped,
}

/// Health information for a system dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealthCheck {
    pub name: String,
    pub status: PortServiceState,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

/// Extended health response with detailed dependency info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedHealthResponse {
    pub status: PortServiceState,
    pub version: String,
    pub dependencies: Vec<DependencyHealthCheck>,
}

/// Interface for graceful shutdown coordination
pub trait ShutdownCoordinator: Send + Sync {
    fn signal_shutdown(&self);
    fn is_shutting_down(&self) -> bool;
}

/// Managed lifecycle for background services
#[async_trait::async_trait]
pub trait LifecycleManaged: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&self) -> crate::error::Result<()>;
    async fn stop(&self) -> crate::error::Result<()>;
    async fn restart(&self) -> crate::error::Result<()> {
        self.stop().await?;
        self.start().await
    }
    fn state(&self) -> PortServiceState;
    async fn health_check(&self) -> DependencyHealthCheck {
        DependencyHealthCheck {
            name: self.name().to_string(),
            status: self.state(),
            message: None,
            latency_ms: None,
        }
    }
}
