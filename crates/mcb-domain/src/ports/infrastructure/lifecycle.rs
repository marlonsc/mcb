//! Lifecycle management and health check ports.

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Current state of a port service
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PortServiceState {
    /// Service is in the process of starting up
    Starting,
    /// Service is active and running correctly
    Running,
    /// Service is in the process of shutting down
    Stopping,
    /// Service is inactive and fully stopped
    #[default]
    Stopped,
}

/// Health status for a system dependency
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DependencyHealth {
    /// Dependency is operational and communicating correctly
    Healthy,
    /// Dependency is responding but with issues or high latency
    Degraded,
    /// Dependency is not responding or in a failed state
    Unhealthy,
    /// Health status is not yet determined
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
    /// Signal that a shutdown has been initiated.
    fn signal_shutdown(&self);
    /// Check if the system is currently shutting down.
    fn is_shutting_down(&self) -> bool;
}

/// Managed lifecycle for background services
#[async_trait::async_trait]
pub trait LifecycleManaged: Send + Sync {
    /// Human-readable name of the service.
    fn name(&self) -> &str;
    /// Start the service.
    async fn start(&self) -> Result<()>;
    /// Stop the service gracefully.
    async fn stop(&self) -> Result<()>;
    /// Restart the service by calling stop then start.
    async fn restart(&self) -> Result<()> {
        self.stop().await?;
        self.start().await
    }
    /// Get the current operational state.
    fn state(&self) -> PortServiceState;
    /// Perform a health check on the service.
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
            last_check: mcb_utils::utils::time::epoch_secs_u64().unwrap_or(0),
        }
    }
}
