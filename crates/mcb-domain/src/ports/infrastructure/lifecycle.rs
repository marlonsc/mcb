//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md)
//!
//! Lifecycle & Health Port Definitions
//!
//! Types and traits for service lifecycle management, health checks,
//! and graceful shutdown coordination.

use serde::{Deserialize, Serialize};

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
            last_check: crate::utils::time::epoch_secs_u64().unwrap_or(0), // INTENTIONAL: Use 0 if system time is unavailable
        }
    }
}
