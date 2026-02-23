//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Service Lifecycle Management
//!
//! This module provides the `ServiceManager` which orchestrates the lifecycle
//! (start, stop, restart) of all registered services. It ensures consistent
//! state transitions and publishes domain events for system-wide observability.
//!
//! ## Architecture
//!
//! ```text
//!                    ┌─────────────────┐
//!                    │   ServiceManager   │
//!                    └────────┬────────┘
//!                              │
//!        ┌──────────── ─────┼─────────────────┐
//!        │                     │                    │
//!        ▼                    ▼                    ▼
//! ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
//! │    Service A   │    │    Service B    │    │    Service C   │
//! │  (Embedding)   │    │  (VectorStore)  │    │    (Cache)     │
//! └──────────────┘    └──────────────┘    └──────────────┘
//!        │                    │                    │
//!        └─────────────────┼─────────────────┘
//!                             │
//!                             ▼
//!                    ┌─────────────────┐
//!                    │      EventBus      │
//!                    │  (SSE/WebSocket)   │
//!                    └─────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```text
//! // Create manager with event bus
//! let manager = ServiceManager::new(event_bus);
//!
//! // Register services
//! manager.register(Arc::new(embedding_service));
//! manager.register(Arc::new(vector_store_service));
//!
//! // List all services
//! for info in manager.list() {
//!     println!("{}: {:?}", info.name, info.state);
//! }
//!
//! // Restart a specific service
//! manager.restart("embedding").await?;
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use dashmap::DashMap;
use mcb_domain::events::{DomainEvent, ServiceState as EventServiceState};
use mcb_domain::ports::{
    DependencyHealthCheck, EventBusProvider, LifecycleManaged, PortServiceState,
    ShutdownCoordinator,
};
use serde::Serialize;
use tokio::sync::Notify;
/// Information about a registered service
#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Current state
    pub state: PortServiceState,
}

/// Error type for service manager operations
#[derive(Debug, thiserror::Error)]
pub enum ServiceManagerError {
    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    /// Service operation failed
    #[error("Service operation failed: {0}")]
    OperationFailed(#[from] mcb_domain::error::Error),
}

/// Central coordinator for service lifecycle management.
///
/// Tracks all managed services and provides operations to start, stop,
/// and restart them. State changes are published as domain events for
/// real-time monitoring via the [`EventBusProvider`].
///
/// Used by admin API handlers in `mcb-server` for runtime service control.
pub struct ServiceManager {
    /// Registered services by name
    services: DashMap<String, Arc<dyn LifecycleManaged>>,
    /// Event bus for publishing state changes
    event_bus: Arc<dyn EventBusProvider>,
}

impl ServiceManager {
    /// Create a new service manager with the given event bus
    pub fn new(event_bus: Arc<dyn EventBusProvider>) -> Self {
        Self {
            services: DashMap::new(),
            event_bus,
        }
    }

    /// Register a service for lifecycle management
    ///
    /// The service will be tracked and can be controlled via this manager.
    pub fn register(&self, service: Arc<dyn LifecycleManaged>) {
        let name = service.name().to_owned();
        mcb_domain::info!(
            "lifecycle",
            "Registering service for lifecycle management",
            &name
        );
        self.services.insert(name, service);
    }

    /// Unregister a service from lifecycle management
    #[must_use]
    pub fn unregister(&self, name: &str) -> Option<Arc<dyn LifecycleManaged>> {
        mcb_domain::info!(
            "lifecycle",
            "Unregistering service from lifecycle management",
            &name
        );
        self.services.remove(name).map(|(_, v)| v)
    }

    /// Get information about all registered services
    #[must_use]
    pub fn list(&self) -> Vec<ServiceInfo> {
        self.services
            .iter()
            .map(|entry| ServiceInfo {
                name: entry.key().clone(),
                state: entry.value().state(),
            })
            .collect()
    }

    /// Get information about a specific service
    #[must_use]
    pub fn get(&self, name: &str) -> Option<ServiceInfo> {
        self.services.get(name).map(|entry| ServiceInfo {
            name: entry.key().clone(),
            state: entry.value().state(),
        })
    }

    /// Check if a service is registered
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    /// Get the number of registered services
    #[must_use]
    pub fn count(&self) -> usize {
        self.services.len()
    }

    /// Start a specific service.
    ///
    /// # Errors
    ///
    /// Returns an error if the service is not found or fails to start.
    pub async fn start(&self, name: &str) -> Result<(), ServiceManagerError> {
        self.execute_service_op(
            name,
            "starting",
            "started",
            |s| async move { s.start().await },
        )
        .await
    }

    /// Stop a specific service
    ///
    /// # Errors
    ///
    /// Returns an error if the service is not found or fails to stop.
    pub async fn stop(&self, name: &str) -> Result<(), ServiceManagerError> {
        self.execute_service_op(
            name,
            "stopping",
            "stopped",
            |s| async move { s.stop().await },
        )
        .await
    }

    /// Restart a specific service
    ///
    /// # Errors
    ///
    /// Returns an error if the service is not found or fails to restart.
    pub async fn restart(&self, name: &str) -> Result<(), ServiceManagerError> {
        self.execute_service_op(name, "restarting", "restarted", |s| async move {
            s.restart().await
        })
        .await
    }

    /// Internal helper to execute a service lifecycle operation with state tracking.
    async fn execute_service_op<F, Fut>(
        &self,
        name: &str,
        op_present: &str,
        op_past: &str,
        operation: F,
    ) -> Result<(), ServiceManagerError>
    where
        F: FnOnce(Arc<dyn LifecycleManaged>) -> Fut,
        Fut: std::future::Future<Output = mcb_domain::error::Result<()>>,
    {
        let service = {
            let entry = self
                .services
                .get(name)
                .ok_or_else(|| ServiceManagerError::ServiceNotFound(name.to_owned()))?;
            Arc::clone(entry.value())
        };

        let previous_state = service.state();
        mcb_domain::info!(
            "lifecycle",
            "service op",
            &format!("service = {name}, previous = {previous_state:?}, op = {op_present}")
        );

        operation(Arc::clone(&service)).await?;

        let new_state = service.state();
        self.emit_state_change(name, new_state, Some(previous_state))
            .await;

        mcb_domain::info!(
            "lifecycle",
            "Service state changed",
            &format!("service = {name}, state = {new_state:?}, op_past = {op_past}")
        );
        Ok(())
    }

    /// Start all registered services
    pub async fn start_all(&self) -> Vec<(String, Result<(), ServiceManagerError>)> {
        let names: Vec<String> = self.services.iter().map(|e| e.key().clone()).collect();

        let mut results = Vec::with_capacity(names.len());
        for name in names {
            let result = self.start(&name).await;
            results.push((name, result));
        }
        results
    }

    /// Stop all registered services
    pub async fn stop_all(&self) -> Vec<(String, Result<(), ServiceManagerError>)> {
        let names: Vec<String> = self.services.iter().map(|e| e.key().clone()).collect();

        let mut results = Vec::with_capacity(names.len());
        for name in names {
            let result = self.stop(&name).await;
            results.push((name, result));
        }
        results
    }

    /// Perform health checks on all registered services
    pub async fn health_check_all(&self) -> Vec<DependencyHealthCheck> {
        // Clone the Arc pointers to avoid holding DashMap references across await points
        let services: Vec<Arc<dyn LifecycleManaged>> = self
            .services
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect();

        let mut checks = Vec::with_capacity(services.len());
        for service in services {
            let check = service.health_check().await;
            checks.push(check);
        }

        checks
    }

    /// Emit a service state change event
    async fn emit_state_change(
        &self,
        name: &str,
        state: PortServiceState,
        previous: Option<PortServiceState>,
    ) {
        let event_state = port_to_event_state(state);
        let event_previous = previous.map(port_to_event_state);

        let event = DomainEvent::ServiceStateChanged {
            name: name.to_owned(),
            state: event_state,
            previous_state: event_previous,
        };

        let payload = match serde_json::to_vec(&event) {
            Ok(p) => p,
            Err(e) => {
                mcb_domain::warn!(
                    "lifecycle",
                    "Failed to serialize state change event",
                    &e.to_string()
                );
                return;
            }
        };

        if let Err(e) = self.event_bus.publish("service.state", &payload).await {
            mcb_domain::error!(
                "lifecycle",
                "Failed to publish state change event",
                &e.to_string()
            );
        }
    }
}

/// Convert port `ServiceState` to domain event `ServiceState`
fn port_to_event_state(state: PortServiceState) -> EventServiceState {
    match state {
        PortServiceState::Starting => EventServiceState::Starting,
        PortServiceState::Running => EventServiceState::Running,
        PortServiceState::Stopping => EventServiceState::Stopping,
        PortServiceState::Stopped => EventServiceState::Stopped,
    }
}

impl std::fmt::Debug for ServiceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceManager")
            .field("service_count", &self.services.len())
            .field(
                "services",
                &self
                    .services
                    .iter()
                    .map(|e| e.key().clone())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

// ============================================================================
// Default Shutdown Coordinator
// ============================================================================

/// Default implementation of `ShutdownCoordinator` using atomics and Notify
///
/// This coordinator uses Tokio's Notify for efficient async waiting
/// and an `AtomicBool` for fast shutdown status checks.
pub struct DefaultShutdownCoordinator {
    /// Shutdown signal flag
    shutdown_signal: AtomicBool,
    /// Notification channel for async waiting
    notify: Notify,
}

impl DefaultShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new() -> Self {
        Self {
            shutdown_signal: AtomicBool::new(false),
            notify: Notify::new(),
        }
    }
}

impl Default for DefaultShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DefaultShutdownCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultShutdownCoordinator")
            .field("is_shutting_down", &self.is_shutting_down())
            .finish()
    }
}

impl ShutdownCoordinator for DefaultShutdownCoordinator {
    fn signal_shutdown(&self) {
        mcb_domain::info!("lifecycle", "Shutdown signal received");
        self.shutdown_signal.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    fn is_shutting_down(&self) -> bool {
        self.shutdown_signal.load(Ordering::SeqCst)
    }
}
