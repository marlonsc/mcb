//! Admin Service Domain Ports
//!
//! Defines the port interfaces for admin and monitoring services.
//! These traits break the circular dependency where infrastructure/di
//! previously imported from server layer.

use crate::value_objects::{CollectionId, OperationId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Performance Metrics Types
// ============================================================================

/// Performance metrics data
///
/// This type is defined in domain to allow the trait to be used
/// without circular dependencies on server layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsData {
    /// Total Queries
    pub total_queries: u64,
    /// Successful Queries
    pub successful_queries: u64,
    /// Failed Queries
    pub failed_queries: u64,
    /// Average Response Time Ms
    pub average_response_time_ms: f64,
    /// Cache Hit Rate
    pub cache_hit_rate: f64,
    /// Active Connections
    pub active_connections: u32,
    /// Uptime Seconds
    pub uptime_seconds: u64,
}

// ============================================================================
// Performance Metrics Interface
// ============================================================================

/// Real-time performance metrics tracking interface
///
/// Domain port for tracking server performance metrics including
/// queries, response times, cache hits, and active connections.
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::admin::PerformanceMetricsInterface;
/// use std::sync::Arc;
///
/// fn record_metrics(metrics: Arc<dyn PerformanceMetricsInterface>) {
///     // Record a successful query with 50ms response time (cache miss)
///     metrics.record_query(50, true, false);
///
///     // Track active connections
///     metrics.update_active_connections(1);  // connection opened
///     metrics.update_active_connections(-1); // connection closed
///
///     // Get current metrics snapshot
///     let stats = metrics.get_performance_metrics();
///     println!("Uptime: {}s, Queries: {}", stats.uptime_seconds, stats.total_queries);
/// }
/// ```
pub trait PerformanceMetricsInterface: Send + Sync {
    /// Get server uptime in seconds
    fn uptime_secs(&self) -> u64;

    /// Record a query with its metrics
    fn record_query(&self, response_time_ms: u64, success: bool, cache_hit: bool);

    /// Update active connection count (positive to add, negative to remove)
    fn update_active_connections(&self, delta: i64);

    /// Get current performance metrics snapshot
    fn get_performance_metrics(&self) -> PerformanceMetricsData;
}

// ============================================================================
// Indexing Operations Types
// ============================================================================

/// Tracks ongoing indexing operations
#[derive(Debug, Clone)]
pub struct IndexingOperation {
    /// Operation ID
    pub id: OperationId,
    /// Collection being indexed
    pub collection: CollectionId,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Total files to process
    pub total_files: usize,
    /// Files processed so far
    pub processed_files: usize,
    /// Operation start timestamp (Unix timestamp)
    pub start_timestamp: u64,
}

// ============================================================================
// Indexing Operations Interface
// ============================================================================

/// Interface for indexing operations tracking
///
/// Domain port for tracking ongoing indexing operations in the MCP server.
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::admin::IndexingOperationsInterface;
/// use std::sync::Arc;
///
/// fn show_operations(tracker: Arc<dyn IndexingOperationsInterface>) {
///     // Get all active indexing operations
///     let operations = tracker.get_operations();
///     for (id, op) in operations {
///         println!("Operation {}: {}/{} files in {}",
///             id, op.processed_files, op.total_files, op.collection);
///     }
/// }
///
/// fn start_indexing(tracker: Arc<dyn IndexingOperationsInterface>) {
///     // Start tracking a new operation
///     let operation_id = tracker.start_operation("my-collection", 100);
///
///     // Update progress as files are processed
///     tracker.update_progress(&operation_id, Some("src/main.rs".to_string()), 50);
///
///     // Mark operation complete when done
///     tracker.complete_operation(&operation_id);
/// }
/// ```
pub trait IndexingOperationsInterface: Send + Sync {
    /// Get the map of ongoing indexing operations
    fn get_operations(&self) -> HashMap<OperationId, IndexingOperation>;

    /// Start tracking a new indexing operation
    ///
    /// Returns a unique operation ID that can be used to update progress
    /// and complete the operation.
    fn start_operation(&self, collection: &CollectionId, total_files: usize) -> OperationId;

    /// Update progress for an ongoing operation
    ///
    /// # Arguments
    /// * `operation_id` - The ID returned by `start_operation`
    /// * `current_file` - Optional path of the file currently being processed
    /// * `processed` - Number of files processed so far
    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
    );

    /// Complete and remove an operation from tracking
    ///
    /// After calling this, the operation will no longer appear in `get_operations()`.
    fn complete_operation(&self, operation_id: &OperationId);
}

// ============================================================================
// Service Lifecycle Management
// ============================================================================

/// Health status for a service dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DependencyHealth {
    /// Service is healthy and responsive
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy or unresponsive
    Unhealthy,
    /// Health status is unknown (not checked)
    #[default]
    Unknown,
}

/// Detailed health check result for a service dependency
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependencyHealthCheck {
    /// Name of the dependency
    pub name: String,
    /// Health status
    pub status: DependencyHealth,
    /// Optional message providing more details
    pub message: Option<String>,
    /// Latency in milliseconds (if applicable)
    pub latency_ms: Option<u64>,
    /// Last check timestamp (Unix timestamp)
    pub last_check: u64,
}

/// Port service lifecycle state (simplified, Copy-able version)
///
/// This is a simplified version of ServiceState for port interfaces.
/// For domain events with failure reasons, use `mcb_domain::events::ServiceState`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PortServiceState {
    /// Service is starting up
    Starting,
    /// Service is running normally
    Running,
    /// Service is shutting down
    Stopping,
    /// Service is stopped
    #[default]
    Stopped,
}

/// Lifecycle management interface for services
///
/// Domain port for managing service lifecycle including start, stop,
/// restart, and health checks.
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::admin::{LifecycleManaged, PortServiceState};
/// use std::sync::Arc;
///
/// async fn check_service(service: Arc<dyn LifecycleManaged>) -> mcb_domain::Result<()> {
///     // Check service state
///     if service.state() == PortServiceState::Running {
///         // Perform health check
///         let health = service.health_check().await;
///         println!("Service health: {:?}", health.status);
///     }
///
///     // Graceful shutdown
///     service.stop().await?;
///     Ok(())
/// }
/// ```
#[async_trait::async_trait]
pub trait LifecycleManaged: Send + Sync {
    /// Get the service name
    fn name(&self) -> &str;

    /// Get the current service state
    fn state(&self) -> PortServiceState;

    /// Start the service
    async fn start(&self) -> crate::error::Result<()>;

    /// Stop the service gracefully
    async fn stop(&self) -> crate::error::Result<()>;

    /// Restart the service
    async fn restart(&self) -> crate::error::Result<()> {
        self.stop().await?;
        self.start().await
    }

    /// Perform a health check on this service
    async fn health_check(&self) -> DependencyHealthCheck;
}

// ============================================================================
// Shutdown Coordination
// ============================================================================

/// Shutdown coordinator for managing graceful server shutdown
///
/// This interface allows components to signal and check shutdown status.
/// The actual signaling mechanism is implementation-specific (e.g., broadcast channels).
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::admin::ShutdownCoordinator;
/// use std::sync::Arc;
///
/// fn handle_shutdown(coordinator: Arc<dyn ShutdownCoordinator>) {
///     // Check if shutdown has been requested
///     if coordinator.is_shutting_down() {
///         println!("Shutdown in progress, stopping work");
///     }
///
///     // To trigger shutdown (e.g., from admin API)
///     coordinator.signal_shutdown();
/// }
/// ```
pub trait ShutdownCoordinator: Send + Sync {
    /// Signal all components to begin shutdown
    fn signal_shutdown(&self);

    /// Check if shutdown has been signaled
    fn is_shutting_down(&self) -> bool;
}

/// Extended health check response including dependency status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedHealthResponse {
    /// Overall server status
    pub status: &'static str,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Number of active indexing operations
    pub active_indexing_operations: usize,
    /// Health checks for dependencies
    pub dependencies: Vec<DependencyHealthCheck>,
    /// Overall dependencies health status
    pub dependencies_status: DependencyHealth,
}

// ============================================================================
// Validation Operations Types
// ============================================================================

/// Tracks ongoing validation operations
#[derive(Debug, Clone)]
pub struct ValidationOperation {
    /// Operation ID
    pub id: OperationId,
    /// Workspace being validated
    pub workspace: String,
    /// Validators being run
    pub validators: Vec<String>,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Total files to process
    pub total_files: usize,
    /// Files processed so far
    pub processed_files: usize,
    /// Operation start timestamp (Unix timestamp)
    pub start_timestamp: u64,
    /// Validation result (set when complete)
    pub result: Option<ValidationOperationResult>,
}

/// Result of a completed validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOperationResult {
    /// Total violations found
    pub total_violations: usize,
    /// Number of errors
    pub errors: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Number of infos
    pub infos: usize,
    /// Whether validation passed (no errors)
    pub passed: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

// ============================================================================
// Validation Operations Interface
// ============================================================================

/// Interface for validation operations tracking
///
/// Domain port for tracking ongoing validation operations, following the
/// same pattern as [`IndexingOperationsInterface`].
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::admin::ValidationOperationsInterface;
/// use std::sync::Arc;
///
/// fn show_operations(tracker: Arc<dyn ValidationOperationsInterface>) {
///     // Get all active validation operations
///     let operations = tracker.get_operations();
///     for (id, op) in operations {
///         println!("Operation {}: {}/{} files in {}",
///             id, op.processed_files, op.total_files, op.workspace);
///     }
/// }
///
/// fn start_validation(tracker: Arc<dyn ValidationOperationsInterface>) {
///     // Start tracking a new operation
///     let validators = ["clean_architecture", "solid"].map(String::from);
///     let operation_id = tracker.start_operation(".", &validators);
///
///     // Update progress as files are processed
///     tracker.update_progress(&operation_id, Some("src/main.rs".to_string()), 50, 100);
///
///     // Mark operation complete when done
///     use mcb_domain::ports::admin::ValidationOperationResult;
///     let result = ValidationOperationResult {
///         total_violations: 5,
///         errors: 1,
///         warnings: 3,
///         infos: 1,
///         passed: false,
///         duration_ms: 1500,
///     };
///     tracker.complete_operation(&operation_id, result);
/// }
/// ```
pub trait ValidationOperationsInterface: Send + Sync {
    /// Get the map of ongoing validation operations
    fn get_operations(&self) -> HashMap<OperationId, ValidationOperation>;

    /// Get a specific operation by ID
    fn get_operation(&self, operation_id: &OperationId) -> Option<ValidationOperation>;

    /// Start tracking a new validation operation
    ///
    /// # Arguments
    /// * `workspace` - Path to workspace being validated
    /// * `validators` - List of validators being run
    ///
    /// # Returns
    /// A unique operation ID that can be used to update progress
    fn start_operation(&self, workspace: &str, validators: &[String]) -> OperationId;

    /// Update progress for an ongoing operation
    ///
    /// # Arguments
    /// * `operation_id` - The ID returned by `start_operation`
    /// * `current_file` - Optional path of the file currently being processed
    /// * `processed` - Number of files processed so far
    /// * `total` - Total number of files to process
    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
        total: usize,
    );

    /// Complete an operation with its result
    ///
    /// After calling this, the operation status changes to complete
    /// and the result is stored for retrieval.
    ///
    /// # Arguments
    /// * `operation_id` - The operation ID
    /// * `result` - The validation result
    fn complete_operation(&self, operation_id: &OperationId, result: ValidationOperationResult);

    /// Cancel an ongoing operation
    ///
    /// Removes the operation from tracking without storing a result.
    fn cancel_operation(&self, operation_id: &OperationId);

    /// Check if an operation is still in progress
    fn is_in_progress(&self, operation_id: &OperationId) -> bool;
}
