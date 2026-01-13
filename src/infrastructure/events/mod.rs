//! Event Bus System for Decoupled Communication
//!
//! This module provides a pluggable event bus system supporting multiple backends:
//! - Tokio broadcast (in-process, default for single-instance)
//! - NATS JetStream (cross-process, persistent, for cluster deployments)

pub mod nats;
pub mod tokio_impl;

pub use nats::NatsEventBus;
pub use tokio_impl::{create_shared_event_bus, EventBus, TokioEventReceiver};

use crate::domain::error::Result;
use std::sync::Arc;

/// System-wide events for internal communication
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SystemEvent {
    /// Request to clear all caches
    CacheClear {
        /// Optional namespace to clear (None = all)
        namespace: Option<String>,
    },
    /// Request to create a backup
    BackupCreate {
        /// Target path for backup
        path: String,
    },
    /// Request to restore a backup
    BackupRestore {
        /// Path to backup to restore
        path: String,
    },
    /// Request to rebuild the index
    IndexRebuild {
        /// Collection to rebuild (None = all)
        collection: Option<String>,
    },
    /// Request to clear an index
    IndexClear {
        /// Collection to clear (None = all)
        collection: Option<String>,
    },
    /// Request to optimize an index
    IndexOptimize {
        /// Collection to optimize (None = all)
        collection: Option<String>,
    },
    /// Configuration was reloaded
    ConfigReloaded,
    /// Configuration has been changed by an administrator
    ConfigurationChanged {
        /// User who made the change
        user: String,
        /// List of changes that were applied
        changes: Vec<String>,
        /// Whether restart is required to apply all changes
        requires_restart: bool,
        /// When the change was made
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Server is shutting down
    Shutdown,
    /// Request to reload configuration (SIGHUP)
    Reload,
    /// Request to respawn the server binary (SIGUSR1)
    Respawn,
    /// Binary file was updated, prepare for respawn
    BinaryUpdated {
        /// New binary path
        path: String,
    },
    /// Sync operation completed
    SyncCompleted {
        /// Path that was synced
        path: String,
        /// Number of files that changed
        files_changed: i32,
    },

    // === Subsystem Control Events (ADR-007) ===
    /// Request to restart a provider
    ProviderRestart {
        /// Type of provider (embedding, vector_store, etc.)
        provider_type: String,
        /// Unique identifier for the provider instance
        provider_id: String,
    },
    /// Request to reconfigure a provider without full restart
    ProviderReconfigure {
        /// Type of provider (embedding, vector_store, etc.)
        provider_type: String,
        /// New configuration to apply
        config: serde_json::Value,
    },
    /// Request a health check on a specific subsystem
    SubsystemHealthCheck {
        /// Subsystem identifier to check
        subsystem_id: String,
    },
    /// Request to reload router configuration
    RouterReload,

    // === Recovery Events ===
    /// Provider has been successfully restarted
    ProviderRestarted {
        /// Type of provider (embedding, vector_store, etc.)
        provider_type: String,
        /// Unique identifier for the provider instance
        provider_id: String,
    },
    /// Recovery process has started for a subsystem
    RecoveryStarted {
        /// Subsystem identifier being recovered
        subsystem_id: String,
        /// Current retry attempt number
        retry_attempt: u32,
    },
    /// Recovery process completed for a subsystem
    RecoveryCompleted {
        /// Subsystem identifier
        subsystem_id: String,
        /// Whether the recovery was successful
        success: bool,
        /// Message describing the outcome
        message: String,
    },
    /// Recovery exhausted all retries for a subsystem
    RecoveryExhausted {
        /// Subsystem identifier
        subsystem_id: String,
        /// Total retries attempted
        total_retries: u32,
        /// Last error message if any
        last_error: Option<String>,
    },
}

/// Abstract trait for event receiver implementations
/// Allows events to be received from different backends (tokio, NATS, etc.)
#[async_trait::async_trait]
pub trait EventReceiver: Send {
    /// Receive the next event from the event bus
    async fn recv(&mut self) -> Result<SystemEvent>;
}

/// Abstract trait for event bus provider implementations
///
/// Allows pluggable backends (tokio broadcast, NATS, etc.)
/// Each backend should implement this trait to provide:
/// - Async publish with error handling
/// - Async subscribe with event receiver
/// - Subscriber count tracking
#[async_trait::async_trait]
pub trait EventBusProvider: Send + Sync {
    /// Publish an event to all subscribers
    ///
    /// Returns the number of subscribers that received the event
    async fn publish(&self, event: SystemEvent) -> Result<usize>;

    /// Subscribe to receive events
    async fn subscribe(&self) -> Result<Box<dyn EventReceiver>>;

    /// Get the number of active subscribers
    fn subscriber_count(&self) -> usize;
}

/// Shared trait object for event bus provider
pub type SharedEventBusProvider = Arc<dyn EventBusProvider>;

/// Configuration for event bus selection and behavior
#[derive(Debug, Clone)]
pub enum EventBusConfig {
    /// Use in-process tokio broadcast (default)
    Tokio {
        /// Channel capacity
        capacity: usize,
    },
    /// Use NATS JetStream for cross-process communication
    Nats {
        /// NATS server URL (e.g., "nats://localhost:4222")
        url: String,
        /// Stream retention time in hours
        retention_hours: u64,
        /// Maximum messages per subject
        max_msgs_per_subject: i64,
    },
}

impl Default for EventBusConfig {
    fn default() -> Self {
        EventBusConfig::Tokio { capacity: 100 }
    }
}

impl EventBusConfig {
    /// Create from environment variables
    ///
    /// Respects:
    /// - `MCP_EVENT_BUS_TYPE` - "tokio" or "nats" (default: "tokio")
    /// - `MCP_NATS_URL` - NATS server URL (default: "nats://localhost:4222")
    /// - `MCP_NATS_RETENTION_HOURS` - Event retention (default: 1)
    /// - `MCP_EVENT_BUS_CAPACITY` - Tokio channel capacity (default: 100)
    pub fn from_env() -> Self {
        let bus_type = std::env::var("MCP_EVENT_BUS_TYPE").unwrap_or_else(|_| "tokio".to_string());

        match bus_type.as_str() {
            "nats" => {
                let url = std::env::var("MCP_NATS_URL")
                    .unwrap_or_else(|_| "nats://localhost:4222".to_string());
                let retention_hours = std::env::var("MCP_NATS_RETENTION_HOURS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1);
                let max_msgs = std::env::var("MCP_NATS_MAX_MSGS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10000);

                EventBusConfig::Nats {
                    url,
                    retention_hours,
                    max_msgs_per_subject: max_msgs,
                }
            }
            _ => {
                let capacity = std::env::var("MCP_EVENT_BUS_CAPACITY")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(100);

                EventBusConfig::Tokio { capacity }
            }
        }
    }
}

/// Create an event bus provider from configuration
pub async fn create_event_bus(config: &EventBusConfig) -> Result<SharedEventBusProvider> {
    match config {
        EventBusConfig::Tokio { capacity } => {
            tracing::info!(
                "[EVENT_BUS] Using tokio broadcast backend (capacity: {})",
                capacity
            );
            Ok(Arc::new(EventBus::new(*capacity)) as SharedEventBusProvider)
        }
        EventBusConfig::Nats { url, .. } => {
            tracing::info!("[EVENT_BUS] Using NATS JetStream backend (url: {})", url);
            let bus = NatsEventBus::new(url).await?;
            Ok(Arc::new(bus) as SharedEventBusProvider)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_event_bus_config_default() {
        let config = EventBusConfig::default();
        match config {
            EventBusConfig::Tokio { capacity } => assert_eq!(capacity, 100),
            _ => panic!("Expected Tokio config"),
        }
    }

    #[test]
    #[serial]
    fn test_event_bus_config_from_env_tokio() {
        // Clean environment before test
        std::env::remove_var("MCP_EVENT_BUS_TYPE");
        std::env::remove_var("MCP_NATS_URL");

        std::env::set_var("MCP_EVENT_BUS_TYPE", "tokio");
        let config = EventBusConfig::from_env();

        // Clean up after test
        std::env::remove_var("MCP_EVENT_BUS_TYPE");

        match config {
            EventBusConfig::Tokio { capacity } => assert!(capacity > 0),
            _ => panic!("Expected Tokio config"),
        }
    }

    #[test]
    #[serial]
    fn test_event_bus_config_from_env_nats() {
        // Clean environment before test
        std::env::remove_var("MCP_EVENT_BUS_TYPE");
        std::env::remove_var("MCP_NATS_URL");

        std::env::set_var("MCP_EVENT_BUS_TYPE", "nats");
        std::env::set_var("MCP_NATS_URL", "nats://test:4222");
        let config = EventBusConfig::from_env();

        // Clean up after test
        std::env::remove_var("MCP_EVENT_BUS_TYPE");
        std::env::remove_var("MCP_NATS_URL");

        match config {
            EventBusConfig::Nats { url, .. } => assert_eq!(url, "nats://test:4222"),
            _ => panic!("Expected NATS config"),
        }
    }
}
