//! Infrastructure service ports.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

#![allow(missing_docs)]

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
    async fn publish_event(&self, event: DomainEvent) -> Result<()>;
    async fn subscribe_events(&self) -> Result<DomainEventStream>;
    fn has_subscribers(&self) -> bool;

    // Low-Level Raw API
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()>;
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
    pub name: String,
    pub status: DependencyHealth,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
    pub last_check: u64,
}

/// Extended health response with detailed dependency info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedHealthResponse {
    pub status: &'static str,
    pub uptime_seconds: u64,
    pub active_indexing_operations: usize,
    pub dependencies: Vec<DependencyHealthCheck>,
    pub dependencies_status: DependencyHealth,
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
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn restart(&self) -> Result<()> {
        self.stop().await?;
        self.start().await
    }
    fn state(&self) -> PortServiceState;
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
    pub operation_type: String,
    pub cost_sensitivity: f64,
    pub quality_requirement: f64,
    pub latency_sensitivity: f64,
    pub preferred_providers: Vec<String>,
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
    async fn select_embedding_provider(&self, context: &ProviderContext) -> Result<String>;
    async fn select_vector_store_provider(&self, context: &ProviderContext) -> Result<String>;
    async fn get_provider_health(&self, provider_id: &str) -> Result<ProviderHealthStatus>;
    async fn report_failure(&self, provider_id: &str, error: &str) -> Result<()>;
    async fn report_success(&self, provider_id: &str) -> Result<()>;
    async fn get_all_health(&self) -> Result<HashMap<String, ProviderHealthStatus>>;
    async fn get_stats(&self) -> HashMap<String, serde_json::Value>;
}

// ============================================================================
// Snapshot & Sync Providers
// ============================================================================

/// Sync Provider Interface
#[async_trait]
pub trait SyncProvider: Send + Sync {
    async fn should_debounce(&self, codebase_path: &Path) -> Result<bool>;
    async fn update_last_sync(&self, codebase_path: &Path);
    async fn acquire_sync_slot(&self, codebase_path: &Path) -> Result<Option<SyncBatch>>;
    async fn release_sync_slot(&self, codebase_path: &Path, batch: SyncBatch) -> Result<()>;
    async fn get_changed_files(&self, codebase_path: &Path) -> Result<Vec<String>>;
    fn sync_interval(&self) -> Duration;
    fn debounce_interval(&self) -> Duration;
}

/// Snapshot Provider Interface
#[async_trait]
pub trait SnapshotProvider: Send + Sync {
    async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot>;
    async fn load_snapshot(&self, root_path: &Path) -> Result<Option<CodebaseSnapshot>>;
    async fn compare_snapshots(
        &self,
        old_snapshot: &CodebaseSnapshot,
        new_snapshot: &CodebaseSnapshot,
    ) -> Result<SnapshotChanges>;
    async fn get_changed_files(&self, root_path: &Path) -> Result<Vec<String>>;
}

// ============================================================================
// Sync Coordination
// ============================================================================

/// Configuration for sync operations
#[derive(Debug, Clone)]
pub struct SyncOptions {
    pub debounce_duration: Duration,
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
    pub performed: bool,
    pub files_changed: usize,
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
    async fn should_debounce(&self, codebase_path: &Path) -> Result<bool>;
    async fn sync(&self, codebase_path: &Path, options: SyncOptions) -> Result<SyncResult>;
    async fn get_changed_files(&self, codebase_path: &Path) -> Result<Vec<String>>;
    async fn mark_synced(&self, codebase_path: &Path) -> Result<()>;
    fn tracked_file_count(&self) -> usize;
}

/// Shared sync coordinator for dependency injection
pub type SharedSyncCoordinator = Arc<dyn SyncCoordinator>;
