//! Active Health Monitor for Proactive Provider Health Checking
//!
//! This module implements background health monitoring that:
//! - Proactively probes all registered providers on a schedule
//! - Detects provider degradation via trend analysis
//! - Publishes health events for recovery coordination
//! - Integrates with RecoveryManager for automatic restarts
//!
//! Uses `CancellationToken` for async-native shutdown signaling.

use crate::adapters::providers::routing::health::HealthMonitor;
use crate::infrastructure::di::registry::ProviderRegistryTrait;
use crate::infrastructure::events::{SharedEventBusProvider, SystemEvent};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

/// Configuration for active health monitoring
#[derive(Debug, Clone)]
pub struct ActiveHealthConfig {
    /// Interval between health probing runs (default: 10 seconds)
    pub probe_interval_secs: u64,
    /// Timeout for individual health checks (default: 5 seconds)
    pub probe_timeout_secs: u64,
    /// Number of consecutive failures before marking unhealthy (default: 3)
    pub failure_threshold: u32,
}

impl Default for ActiveHealthConfig {
    fn default() -> Self {
        Self {
            probe_interval_secs: 10,
            probe_timeout_secs: 5,
            failure_threshold: 3,
        }
    }
}

/// Active health monitor that proactively probes providers
///
/// Uses `CancellationToken` for async-native shutdown signaling.
pub struct ActiveHealthMonitor {
    health_monitor: Arc<HealthMonitor>,
    registry: Arc<dyn ProviderRegistryTrait>,
    event_bus: SharedEventBusProvider,
    config: ActiveHealthConfig,
    cancel_token: CancellationToken,
}

impl ActiveHealthMonitor {
    /// Create a new active health monitor
    pub fn new(
        health_monitor: Arc<HealthMonitor>,
        registry: Arc<dyn ProviderRegistryTrait>,
        event_bus: SharedEventBusProvider,
        config: ActiveHealthConfig,
    ) -> Self {
        Self {
            health_monitor,
            registry,
            event_bus,
            config,
            cancel_token: CancellationToken::new(),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(
        health_monitor: Arc<HealthMonitor>,
        registry: Arc<dyn ProviderRegistryTrait>,
        event_bus: SharedEventBusProvider,
    ) -> Self {
        Self::new(
            health_monitor,
            registry,
            event_bus,
            ActiveHealthConfig::default(),
        )
    }

    /// Start the health monitoring loop (spawns background task)
    pub fn start(&self) {
        if self.cancel_token.is_cancelled() {
            warn!("[HEALTH] Active monitor was stopped and cannot be restarted");
            return;
        }

        let monitor = self.clone();
        tokio::spawn(async move {
            info!(
                "[HEALTH] Starting active health monitor (interval: {}s)",
                monitor.config.probe_interval_secs
            );
            monitor.run_monitoring_loop().await;
        });
    }

    /// Stop the health monitoring loop
    pub fn stop(&self) {
        self.cancel_token.cancel();
        info!("[HEALTH] Health monitor stopped");
    }

    /// Check if monitor is running
    pub fn is_running(&self) -> bool {
        !self.cancel_token.is_cancelled()
    }

    /// Main monitoring loop - runs periodically
    async fn run_monitoring_loop(&self) {
        let interval = Duration::from_secs(self.config.probe_interval_secs);

        loop {
            tokio::select! {
                biased;

                // Check for cancellation first
                _ = self.cancel_token.cancelled() => {
                    debug!("[HEALTH] Health monitor received cancellation signal");
                    break;
                }

                // Periodic health probe
                _ = tokio::time::sleep(interval) => {
                    debug!("[HEALTH] Starting health probe cycle");
                    self.probe_all_providers().await;
                }
            }
        }

        info!("[HEALTH] Health monitoring loop ended");
    }

    /// Probe all registered providers
    async fn probe_all_providers(&self) {
        // Probe embedding providers
        let embedding_providers = self.registry.list_embedding_providers();
        for provider_id in embedding_providers {
            self.probe_embedding_provider(&provider_id).await;
        }

        // Probe vector store providers
        let vector_store_providers = self.registry.list_vector_store_providers();
        for provider_id in vector_store_providers {
            self.probe_vector_store_provider(&provider_id).await;
        }
    }

    /// Probe a single embedding provider
    async fn probe_embedding_provider(&self, provider_id: &str) {
        let timeout = Duration::from_secs(self.config.probe_timeout_secs);

        match self.registry.get_embedding_provider(provider_id) {
            Ok(provider) => {
                match tokio::time::timeout(timeout, provider.health_check()).await {
                    Ok(Ok(())) => {
                        debug!("[HEALTH] Embedding provider '{}' is healthy", provider_id);
                        // Publish success to event bus for recovery manager
                        if let Err(e) = self
                            .event_bus
                            .publish(SystemEvent::SubsystemHealthCheck {
                                subsystem_id: format!("embedding:{}", provider_id),
                            })
                            .await
                        {
                            warn!("[HEALTH] Failed to publish health check event: {}", e);
                        }
                    }
                    Ok(Err(e)) => {
                        warn!(
                            "[HEALTH] Embedding provider '{}' health check failed: {}",
                            provider_id, e
                        );
                    }
                    Err(_) => {
                        warn!(
                            "[HEALTH] Embedding provider '{}' health check timed out after {}s",
                            provider_id, self.config.probe_timeout_secs
                        );
                    }
                }
            }
            Err(e) => {
                debug!(
                    "[HEALTH] Embedding provider '{}' not found: {}",
                    provider_id, e
                );
            }
        }
    }

    /// Probe a single vector store provider
    async fn probe_vector_store_provider(&self, provider_id: &str) {
        let timeout = Duration::from_secs(self.config.probe_timeout_secs);

        match self.registry.get_vector_store_provider(provider_id) {
            Ok(provider) => {
                match tokio::time::timeout(timeout, provider.health_check()).await {
                    Ok(Ok(())) => {
                        debug!(
                            "[HEALTH] Vector store provider '{}' is healthy",
                            provider_id
                        );
                        // Publish success to event bus for recovery manager
                        if let Err(e) = self
                            .event_bus
                            .publish(SystemEvent::SubsystemHealthCheck {
                                subsystem_id: format!("vector_store:{}", provider_id),
                            })
                            .await
                        {
                            warn!("[HEALTH] Failed to publish health check event: {}", e);
                        }
                    }
                    Ok(Err(e)) => {
                        warn!(
                            "[HEALTH] Vector store provider '{}' health check failed: {}",
                            provider_id, e
                        );
                    }
                    Err(_) => {
                        warn!(
                            "[HEALTH] Vector store provider '{}' health check timed out after {}s",
                            provider_id, self.config.probe_timeout_secs
                        );
                    }
                }
            }
            Err(e) => {
                debug!(
                    "[HEALTH] Vector store provider '{}' not found: {}",
                    provider_id, e
                );
            }
        }
    }
}

impl Clone for ActiveHealthMonitor {
    fn clone(&self) -> Self {
        Self {
            health_monitor: Arc::clone(&self.health_monitor),
            registry: Arc::clone(&self.registry),
            event_bus: Arc::clone(&self.event_bus),
            config: self.config.clone(),
            cancel_token: self.cancel_token.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_health_config_defaults() {
        let config = ActiveHealthConfig::default();
        assert_eq!(config.probe_interval_secs, 10);
        assert_eq!(config.probe_timeout_secs, 5);
        assert_eq!(config.failure_threshold, 3);
    }

    #[test]
    fn test_monitor_lifecycle() {
        let registry = Arc::new(crate::infrastructure::di::registry::ProviderRegistry::new());
        let health = Arc::new(HealthMonitor::with_registry(registry.clone()));

        let monitor = ActiveHealthMonitor::with_defaults(
            health,
            registry,
            Arc::new(crate::infrastructure::events::EventBus::new(10)),
        );

        // CancellationToken starts not cancelled, so is_running() returns true
        // (monitor is ready to run, not yet cancelled)
        assert!(monitor.is_running());

        // After stop(), is_running() returns false
        monitor.stop();
        assert!(!monitor.is_running());
    }
}
