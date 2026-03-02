//! Infrastructure service ports.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

use std::collections::HashMap;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::entities::codebase::{CodebaseSnapshot, SnapshotChanges};
use crate::error::Result;
use crate::events::DomainEvent;
use crate::value_objects::config::SyncBatch;

// ============================================================================
// Events
// ============================================================================

/// Boxed async stream of domain events
pub type DomainEventStream = Pin<Box<dyn Stream<Item = DomainEvent> + Send + Sync + 'static>>;

/// Event bus provider interface for typed event pub/sub
#[async_trait]
pub trait EventBusProvider: Send + Sync {
    /// Publish a domain event to the bus
    async fn publish_event(&self, event: DomainEvent) -> Result<()>;
    /// Subscribe to all domain events
    async fn subscribe_events(&self) -> Result<DomainEventStream>;
    /// Check if there are any active subscribers
    fn has_subscribers(&self) -> bool;

    // Low-Level Raw API
    /// Publish raw payload to a specific topic
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()>;
    /// Subscribe to a specific topic
    async fn subscribe(&self, topic: &str) -> Result<String>;
}

// ============================================================================
// Lifecycle
// ============================================================================

/// Current state of a port service
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PortServiceState {
    Starting,
    Running,
    Stopping,
    #[default]
    Stopped,
}

/// Health status for a system dependency
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DependencyHealth {
    Healthy,
    Degraded,
    Unhealthy,
    #[default]
    Unknown,
}

/// Health information for a system dependency
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependencyHealthCheck {
    /// Name of the dependency
    pub name: String,
    /// Current health status
    pub status: DependencyHealth,
    /// Optional status message or error detail
    pub message: Option<String>,
    /// Latency of the last check in milliseconds, if applicable
    pub latency_ms: Option<u64>,
    /// Timestamp (epoch seconds) of the last check
    pub last_check: u64,
}

/// Extended health response with detailed dependency info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedHealthResponse {
    /// Overall system status
    pub status: &'static str,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Number of indexing operations currently running
    pub active_indexing_operations: usize,
    /// List of individual dependency health reports
    pub dependencies: Vec<DependencyHealthCheck>,
    /// Combined status of all dependencies
    pub dependencies_status: DependencyHealth,
}

/// Interface for graceful shutdown coordination
pub trait ShutdownCoordinator: Send + Sync {
    /// Signal that a shutdown has been initiated
    fn signal_shutdown(&self);
    /// Check if the system is currently shutting down
    fn is_shutting_down(&self) -> bool;
}

/// Managed lifecycle for background services
#[async_trait::async_trait]
pub trait LifecycleManaged: Send + Sync {
    /// Human-readable name of the service
    fn name(&self) -> &str;
    /// Start the service
    async fn start(&self) -> Result<()>;
    /// Stop the service gracefully
    async fn stop(&self) -> Result<()>;
    /// Restart the service by calling stop then start
    async fn restart(&self) -> Result<()> {
        self.stop().await?;
        self.start().await
    }
    /// Get the current operational state
    fn state(&self) -> PortServiceState;
    /// Perform a health check on the service
    async fn health_check(&self) -> DependencyHealthCheck {
        DependencyHealthCheck {
            name: self.name().to_owned(),
            status: match self.state() {
                PortServiceState::Running => DependencyHealth::Healthy,
                PortServiceState::Starting => DependencyHealth::Unknown,
                PortServiceState::Stopping | PortServiceState::Stopped => {
                    DependencyHealth::Unhealthy
                }
            },
            message: None,
            latency_ms: None,
            last_check: crate::utils::time::epoch_secs_u64().unwrap_or(0),
        }
    }
}

// ============================================================================
// Logging
// ============================================================================

/// Log level for the unified `log` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Operation logger interface: one method for all levels, optional detail.
pub trait OperationLogger: Send + Sync {
    /// Logs a message with a specific level, context, and optional detail.
    ///
    /// # Parameters
    /// - `level`: The severity level of the log message.
    /// - `context`: The name of the module or component generating the log.
    /// - `message`: The primary log message text.
    /// - `detail`: An optional displayable value providing extra context (e.g., a struct).
    fn log(
        &self,
        level: LogLevel,
        context: &str,
        message: &str,
        detail: Option<&dyn std::fmt::Display>,
    );
}

// ============================================================================
// Routing
// ============================================================================

/// Health status for a provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProviderHealthStatus {
    #[default]
    Healthy,
    Degraded,
    Unhealthy,
}

/// Context for provider selection decisions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderContext {
    /// Type of operation being performed (e.g., "fast", "accurate")
    pub operation_type: String,
    /// Importance of cost (0.0 to 1.0)
    pub cost_sensitivity: f64,
    /// Minimum quality threshold (0.0 to 1.0)
    pub quality_requirement: f64,
    /// Importance of low latency (0.0 to 1.0)
    pub latency_sensitivity: f64,
    /// List of providers to prefer if available
    pub preferred_providers: Vec<String>,
    /// List of providers that should not be used
    pub excluded_providers: Vec<String>,
}

impl ProviderContext {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation_type = operation.into();
        self
    }

    #[must_use]
    pub fn prefer(mut self, provider: impl Into<String>) -> Self {
        self.preferred_providers.push(provider.into());
        self
    }

    #[must_use]
    pub fn exclude(mut self, provider: impl Into<String>) -> Self {
        self.excluded_providers.push(provider.into());
        self
    }
}

/// Provider routing interface
#[async_trait]
pub trait ProviderRouter: Send + Sync {
    /// Select the best embedding provider for the given context
    async fn select_embedding_provider(&self, context: &ProviderContext) -> Result<String>;
    /// Select the best vector store provider for the given context
    async fn select_vector_store_provider(&self, context: &ProviderContext) -> Result<String>;
    /// Get health status of a specific provider
    async fn get_provider_health(&self, provider_id: &str) -> Result<ProviderHealthStatus>;
    /// Report an operation failure for a provider
    async fn report_failure(&self, provider_id: &str, error: &str) -> Result<()>;
    /// Report an operation success for a provider
    async fn report_success(&self, provider_id: &str) -> Result<()>;
    /// Get health status of all known providers
    async fn get_all_health(&self) -> Result<HashMap<String, ProviderHealthStatus>>;
    /// Get detailed statistics for all providers
    async fn get_stats(&self) -> HashMap<String, serde_json::Value>;
}

// ============================================================================
// Snapshot & Sync Providers
// ============================================================================

/// Sync Provider Interface
#[async_trait]
pub trait SyncProvider: Send + Sync {
    /// Check if a sync operation should be debounced for the given path
    async fn should_debounce(&self, codebase_path: &Path) -> Result<bool>;
    /// Update the timestamp of the last successful sync
    async fn update_last_sync(&self, codebase_path: &Path);
    /// Attempt to acquire a slot for a sync batch
    async fn acquire_sync_slot(&self, codebase_path: &Path) -> Result<Option<SyncBatch>>;
    /// Release a previously acquired sync slot
    async fn release_sync_slot(&self, codebase_path: &Path, batch: SyncBatch) -> Result<()>;
    /// Get list of files that have changed since last sync
    async fn get_changed_files(&self, codebase_path: &Path) -> Result<Vec<String>>;
    /// Desired interval between syncs
    fn sync_interval(&self) -> Duration;
    /// Desired debounce duration
    fn debounce_interval(&self) -> Duration;
}

/// Snapshot Provider Interface
#[async_trait]
pub trait SnapshotProvider: Send + Sync {
    /// Create a new snapshot of the filesystem at root_path
    async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot>;
    /// Load a previously saved snapshot for root_path
    async fn load_snapshot(&self, root_path: &Path) -> Result<Option<CodebaseSnapshot>>;
    /// Compare two snapshots and find the differences
    async fn compare_snapshots(
        &self,
        old_snapshot: &CodebaseSnapshot,
        new_snapshot: &CodebaseSnapshot,
    ) -> Result<SnapshotChanges>;
    /// Efficiently get files changed on disk since last snapshot
    async fn get_changed_files(&self, root_path: &Path) -> Result<Vec<String>>;
}

// ============================================================================
// Sync Coordination
// ============================================================================

/// Configuration for sync operations
#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// Minimum time between consecutive sync attempts
    pub debounce_duration: Duration,
    /// Whether to force a sync even if debouncing would normally skip it
    pub force: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_secs(60),
            force: false,
        }
    }
}

/// Result of a sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Whether the sync operation actually ran
    pub performed: bool,
    /// Number of files identified as changed
    pub files_changed: usize,
    /// List of paths for the changed files
    pub changed_files: Vec<String>,
}

impl SyncResult {
    #[must_use]
    pub fn skipped() -> Self {
        Self {
            performed: false,
            files_changed: 0,
            changed_files: Vec::new(),
        }
    }

    #[must_use]
    pub fn completed(changed_files: Vec<String>) -> Self {
        let files_changed = changed_files.len();
        Self {
            performed: true,
            files_changed,
            changed_files,
        }
    }
}

/// Domain Port for File Synchronization Coordination
#[async_trait]
pub trait SyncCoordinator: Send + Sync {
    /// Check if a sync should be debounced for the given path
    async fn should_debounce(&self, codebase_path: &Path) -> Result<bool>;
    /// Perform the synchronization operation
    async fn sync(&self, codebase_path: &Path, options: SyncOptions) -> Result<SyncResult>;
    /// Get list of changed files according to the coordinator's state
    async fn get_changed_files(&self, codebase_path: &Path) -> Result<Vec<String>>;
    /// Explicitly mark a path as successfully synced
    async fn mark_synced(&self, codebase_path: &Path) -> Result<()>;
    /// Total number of files currently tracked by the coordinator
    fn tracked_file_count(&self) -> usize;
}

/// Shared sync coordinator for dependency injection
pub type SharedSyncCoordinator = Arc<dyn SyncCoordinator>;

// ============================================================================
// Database Migrations
// ============================================================================

/// Domain port for database migration providers.
///
/// Implementations supply ordered migration objects as type-erased
/// `Box<dyn Any + Send>`.  The concrete migration trait (e.g.
/// `sea_orm_migration::MigrationTrait`) lives in the infrastructure layer;
/// the domain remains free of ORM dependencies.
#[async_trait::async_trait]
pub trait MigrationProvider: Send + Sync {
    /// Returns the ordered list of migrations as type-erased boxes.
    ///
    /// Each element is expected to be a `Box<dyn MigrationTrait>` from the
    /// ORM framework used by the implementing provider.
    fn migrations(&self) -> Vec<Box<dyn std::any::Any + Send>>;

    /// Apply all pending migrations (up) to the given database connection.
    ///
    /// The `db` parameter is a type-erased database connection
    /// (e.g. `DatabaseConnection` from SeaORM).
    ///
    /// `steps` limits how many pending migrations to apply; `None` applies all.
    async fn migrate_up(
        &self,
        db: Box<dyn std::any::Any + Send + Sync>,
        steps: Option<u32>,
    ) -> crate::error::Result<()>;

    /// Rollback applied migrations on the given database connection.
    ///
    /// `steps` limits how many migrations to rollback; `None` rolls back all.
    async fn migrate_down(
        &self,
        db: Box<dyn std::any::Any + Send + Sync>,
        steps: Option<u32>,
    ) -> crate::error::Result<()>;
}

/// Shared migration provider for dependency injection.
pub type SharedMigrationProvider = Arc<dyn MigrationProvider>;

// ============================================================================
// GraphQL Schema
// ============================================================================

/// Domain port for GraphQL schema providers.
///
/// Implementations build a GraphQL schema from the database connection
/// and return it as a type-erased `Box<dyn Any + Send + Sync>`.
/// The concrete schema type (e.g. `async_graphql::dynamic::Schema`) lives
/// in the provider layer; the domain remains free of GraphQL dependencies.
pub trait GraphQLSchemaProvider: Send + Sync {
    /// Build a GraphQL schema using the given opaque database connection.
    ///
    /// # Arguments
    /// * `db` - Database connection as `Box<dyn Any + Send + Sync>`
    /// * `depth` - Optional query depth limit
    /// * `complexity` - Optional query complexity limit
    ///
    /// # Errors
    /// Returns an error if the schema build fails.
    fn build_schema(
        &self,
        db: Box<dyn std::any::Any + Send + Sync>,
        depth: Option<usize>,
        complexity: Option<usize>,
    ) -> crate::error::Result<Box<dyn std::any::Any + Send + Sync>>;
}

/// Shared GraphQL schema provider for dependency injection.
pub type SharedGraphQLSchemaProvider = Arc<dyn GraphQLSchemaProvider>;
