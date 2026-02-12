//! Event Publisher Domain Port
//!
//! Defines the business contract for publishing system events. This abstraction
//! enables services to publish events without coupling to specific implementations
//! (tokio broadcast, NATS, etc.).

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Service lifecycle state for managed services
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ServiceState {
    /// Service is starting up
    Starting,
    /// Service is running normally
    Running,
    /// Service is stopping
    Stopping,
    /// Service is stopped
    #[default]
    Stopped,
    /// Service failed with error
    Failed {
        /// Reason for failure
        reason: String,
    },
}

/// System-wide event types for decoupled service communication
///
/// These events represent domain-level operations that services can publish
/// and subscribe to without direct coupling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomainEvent {
    // === Indexing Events ===
    /// Index rebuild requested or completed
    IndexRebuild {
        /// Collection being rebuilt (None = all collections)
        collection: Option<String>,
    },
    /// Indexing operation started
    IndexingStarted {
        /// Collection being indexed
        collection: String,
        /// Total number of files to process
        total_files: usize,
    },
    /// Indexing progress update
    IndexingProgress {
        /// Collection being indexed
        collection: String,
        /// Files processed so far
        processed: usize,
        /// Total files to process
        total: usize,
        /// Current file being processed
        current_file: Option<String>,
    },
    /// Indexing operation completed
    IndexingCompleted {
        /// Collection that was indexed
        collection: String,
        /// Total chunks created
        chunks: usize,
        /// Duration in milliseconds
        duration_ms: u64,
    },

    // === Sync Events ===
    /// Sync operation completed
    SyncCompleted {
        /// Path that was synced
        path: String,
        /// Number of files that changed
        files_changed: i32,
    },

    // === Cache Events ===
    /// Cache invalidation requested
    CacheInvalidate {
        /// Namespace to invalidate (None = all)
        namespace: Option<String>,
    },

    // === Snapshot Events ===
    /// Snapshot created for a codebase
    SnapshotCreated {
        /// Root path of the codebase
        root_path: String,
        /// Number of files in snapshot
        file_count: usize,
    },

    // === File Watcher Events ===
    /// File changes detected
    FileChangesDetected {
        /// Root path being monitored
        root_path: String,
        /// Number of added files
        added: usize,
        /// Number of modified files
        modified: usize,
        /// Number of removed files
        removed: usize,
    },

    // === Service Lifecycle Events ===
    /// Service state changed
    ServiceStateChanged {
        /// Name of the service
        name: String,
        /// New state
        state: ServiceState,
        /// Previous state (if known)
        previous_state: Option<ServiceState>,
    },

    // === Configuration Events ===
    /// Configuration section reloaded
    ConfigReloaded {
        /// Section that was reloaded
        section: String,
        /// Timestamp of reload (Unix epoch seconds)
        timestamp: i64,
    },

    // === Health Events ===
    /// Health check completed
    HealthCheckCompleted {
        /// Overall status
        status: String,
        /// Number of healthy dependencies
        healthy_count: usize,
        /// Number of unhealthy dependencies
        unhealthy_count: usize,
    },

    // === Metrics Events ===
    /// Periodic metrics snapshot
    MetricsSnapshot {
        /// Timestamp of snapshot (Unix epoch seconds)
        timestamp: i64,
    },

    // === Search Events ===
    /// Search query executed
    SearchExecuted {
        /// Search query
        query: String,
        /// Collection searched
        collection: String,
        /// Number of results
        results: usize,
        /// Duration in milliseconds
        duration_ms: u64,
    },

    // === Validation Events ===
    /// Validation operation started
    ValidationStarted {
        /// Operation ID for tracking
        operation_id: String,
        /// Workspace being validated
        workspace: String,
        /// Validators being run
        validators: Vec<String>,
        /// Total files to validate
        total_files: usize,
    },
    /// Validation progress update
    ValidationProgress {
        /// Operation ID
        operation_id: String,
        /// Files processed so far
        processed: usize,
        /// Total files to process
        total: usize,
        /// Current file being validated
        current_file: Option<String>,
    },
    /// Validation operation completed
    ValidationCompleted {
        /// Operation ID
        operation_id: String,
        /// Workspace that was validated
        workspace: String,
        /// Total violations found
        total_violations: usize,
        /// Number of errors
        errors: usize,
        /// Number of warnings
        warnings: usize,
        /// Whether validation passed (no errors)
        passed: bool,
        /// Duration in milliseconds
        duration_ms: u64,
    },
}

/// Domain Port for Publishing System Events
///
/// This trait defines the contract for event publishing without coupling to
/// specific implementations. Services use this trait to publish events that
/// other parts of the system can react to.
///
/// # Example
///
/// ```rust,no_run
/// use mcb_domain::events::{EventPublisher, DomainEvent};
///
/// async fn notify_index_rebuild(
///     publisher: &dyn EventPublisher,
///     collection: Option<String>,
/// ) -> mcb_domain::Result<()> {
///     publisher.publish(DomainEvent::IndexRebuild { collection }).await?;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish an event to all subscribers
    ///
    /// Returns Ok(()) if the event was successfully published.
    /// Note: "successfully published" means the event was sent, not necessarily
    /// that subscribers received it (depends on implementation guarantees).
    async fn publish(&self, event: DomainEvent) -> Result<()>;

    /// Check if there are any active subscribers
    ///
    /// Returns true if at least one subscriber is listening for events.
    /// Useful for avoiding unnecessary event creation if no one is listening.
    fn has_subscribers(&self) -> bool;
}

/// Shared event publisher for dependency injection
pub type SharedEventPublisher = Arc<dyn EventPublisher>;
