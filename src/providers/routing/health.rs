//! Health Monitoring Module
//!
//! This module provides health monitoring capabilities using the established `health` crate
//! following SOLID principles with proper separation of concerns.

use crate::core::error::{Error, Result};
use crate::di::registry::ProviderRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Provider health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderHealthStatus {
    /// Provider is healthy and ready
    Healthy,
    /// Provider is unhealthy but may recover
    Unhealthy,
    /// Provider health is unknown
    Unknown,
}

/// Health information for a provider
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub provider_id: String,
    pub status: ProviderHealthStatus,
    pub last_check: Instant,
    pub consecutive_failures: u32,
    pub total_checks: u64,
    pub response_time: Option<Duration>,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub provider_id: String,
    pub status: ProviderHealthStatus,
    pub response_time: Duration,
    pub error_message: Option<String>,
}

/// Trait for provider health checkers
#[async_trait::async_trait]
pub trait ProviderHealthChecker: Send + Sync {
    /// Perform a health check for a specific provider
    async fn check_health(&self, provider_id: &str) -> Result<HealthCheckResult>;
}

/// Real provider health checker that performs actual health checks
pub struct RealProviderHealthChecker {
    registry: Arc<ProviderRegistry>,
    timeout: Duration,
}

impl RealProviderHealthChecker {
    /// Create a new real provider health checker
    pub fn new(registry: Arc<ProviderRegistry>) -> Self {
        Self {
            registry,
            timeout: Duration::from_secs(10), // Default timeout
        }
    }

    /// Create with custom timeout
    pub fn with_timeout(registry: Arc<ProviderRegistry>, timeout: Duration) -> Self {
        Self { registry, timeout }
    }

    /// Check health of an embedding provider
    async fn check_embedding_provider(&self, provider_id: &str) -> Result<HealthCheckResult> {
        let start_time = Instant::now();

        match self.registry.get_embedding_provider(provider_id) {
            Ok(provider) => {
                // Perform a lightweight health check - try to get dimensions
                // This is a minimal operation that verifies the provider is accessible
                match tokio::time::timeout(self.timeout, async {
                    provider.dimensions();
                    Ok::<(), Error>(())
                })
                .await
                {
                    Ok(Ok(_)) => {
                        let response_time = start_time.elapsed();
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Healthy,
                            response_time,
                            error_message: None,
                        })
                    }
                    Ok(Err(e)) => {
                        let response_time = start_time.elapsed();
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Unhealthy,
                            response_time,
                            error_message: Some(format!("Health check failed: {}", e)),
                        })
                    }
                    Err(_) => {
                        // Timeout
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Unhealthy,
                            response_time: self.timeout,
                            error_message: Some("Health check timed out".to_string()),
                        })
                    }
                }
            }
            Err(_) => {
                // Provider not found
                Ok(HealthCheckResult {
                    provider_id: provider_id.to_string(),
                    status: ProviderHealthStatus::Unhealthy,
                    response_time: Duration::from_millis(0),
                    error_message: Some("Provider not registered".to_string()),
                })
            }
        }
    }

    /// Check health of a vector store provider
    async fn check_vector_store_provider(&self, provider_id: &str) -> Result<HealthCheckResult> {
        let start_time = Instant::now();

        match self.registry.get_vector_store_provider(provider_id) {
            Ok(provider) => {
                // Perform a lightweight health check - try to get collection stats for a test collection
                // This verifies the provider can connect and respond
                match tokio::time::timeout(self.timeout, async {
                    provider.collection_exists("__health_check__").await
                })
                .await
                {
                    Ok(Ok(_)) => {
                        let response_time = start_time.elapsed();
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Healthy,
                            response_time,
                            error_message: None,
                        })
                    }
                    Ok(Err(e)) => {
                        // Try alternative check - some providers might not have stats
                        // Fallback to a simple connectivity check
                        let response_time = start_time.elapsed();
                        if e.to_string().contains("collection not found")
                            || e.to_string().contains("Collection not found")
                            || e.to_string().contains("does not exist")
                        {
                            // This is expected for test collection, provider is healthy
                            Ok(HealthCheckResult {
                                provider_id: provider_id.to_string(),
                                status: ProviderHealthStatus::Healthy,
                                response_time,
                                error_message: None,
                            })
                        } else {
                            Ok(HealthCheckResult {
                                provider_id: provider_id.to_string(),
                                status: ProviderHealthStatus::Unhealthy,
                                response_time,
                                error_message: Some(format!("Health check failed: {}", e)),
                            })
                        }
                    }
                    Err(_) => {
                        // Timeout
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Unhealthy,
                            response_time: self.timeout,
                            error_message: Some("Health check timed out".to_string()),
                        })
                    }
                }
            }
            Err(_) => {
                // Provider not found
                Ok(HealthCheckResult {
                    provider_id: provider_id.to_string(),
                    status: ProviderHealthStatus::Unhealthy,
                    response_time: Duration::from_millis(0),
                    error_message: Some("Provider not registered".to_string()),
                })
            }
        }
    }
}

#[async_trait::async_trait]
impl ProviderHealthChecker for RealProviderHealthChecker {
    async fn check_health(&self, provider_id: &str) -> Result<HealthCheckResult> {
        // Determine provider type and perform appropriate health check
        let embedding_providers = self.registry.list_embedding_providers();
        let vector_store_providers = self.registry.list_vector_store_providers();

        if embedding_providers.contains(&provider_id.to_string()) {
            self.check_embedding_provider(provider_id).await
        } else if vector_store_providers.contains(&provider_id.to_string()) {
            self.check_vector_store_provider(provider_id).await
        } else {
            // Provider not found in registry
            Ok(HealthCheckResult {
                provider_id: provider_id.to_string(),
                status: ProviderHealthStatus::Unhealthy,
                response_time: Duration::from_millis(0),
                error_message: Some("Provider not registered".to_string()),
            })
        }
    }
}

/// Health monitor that manages provider health states
pub struct HealthMonitor {
    /// Health data for all providers
    health_data: Arc<RwLock<HashMap<String, ProviderHealth>>>,
    /// Health checker implementation
    checker: Arc<dyn ProviderHealthChecker>,
    /// Failure threshold for marking unhealthy
    failure_threshold: u32,
    /// Check interval (for future periodic health checks)
    #[allow(dead_code)]
    check_interval: Duration,
}

impl HealthMonitor {
    /// Create a new health monitor with registry
    pub fn new(registry: Arc<ProviderRegistry>) -> Self {
        Self::with_registry_and_timeout(registry, Duration::from_secs(10))
    }

    /// Create a new health monitor with registry and custom timeout
    pub fn with_registry_and_timeout(registry: Arc<ProviderRegistry>, timeout: Duration) -> Self {
        let checker = Arc::new(RealProviderHealthChecker::with_timeout(
            Arc::clone(&registry),
            timeout,
        ));
        Self {
            health_data: Arc::new(RwLock::new(HashMap::new())),
            checker,
            failure_threshold: 3,
            check_interval: Duration::from_secs(30),
        }
    }

    /// Create a new health monitor with custom checker (advanced usage)
    pub fn with_checker(checker: Arc<dyn ProviderHealthChecker>) -> Self {
        Self {
            health_data: Arc::new(RwLock::new(HashMap::new())),
            checker,
            failure_threshold: 3,
            check_interval: Duration::from_secs(30),
        }
    }

    /// Check if a provider is healthy
    pub async fn is_healthy(&self, provider_id: &str) -> bool {
        let health_data = self.health_data.read().await;
        matches!(
            health_data.get(provider_id).map(|h| h.status),
            Some(ProviderHealthStatus::Healthy)
        )
    }

    /// Get health information for a provider
    pub async fn get_health(&self, provider_id: &str) -> Option<ProviderHealth> {
        let health_data = self.health_data.read().await;
        health_data.get(provider_id).cloned()
    }

    /// Perform health check for a provider
    pub async fn check_provider(&self, provider_id: &str) -> Result<ProviderHealth> {
        let check_result = self.checker.check_health(provider_id).await?;

        let mut health_data = self.health_data.write().await;
        let health = health_data
            .entry(provider_id.to_string())
            .or_insert_with(|| ProviderHealth {
                provider_id: provider_id.to_string(),
                status: ProviderHealthStatus::Unknown,
                last_check: Instant::now(),
                consecutive_failures: 0,
                total_checks: 0,
                response_time: None,
            });

        health.total_checks += 1;
        health.last_check = Instant::now();
        health.response_time = Some(check_result.response_time);

        match check_result.status {
            ProviderHealthStatus::Healthy => {
                health.status = ProviderHealthStatus::Healthy;
                health.consecutive_failures = 0;
                debug!("Provider {} health check passed", provider_id);
            }
            ProviderHealthStatus::Unhealthy => {
                health.consecutive_failures += 1;
                // For unregistered providers or critical failures, mark unhealthy immediately
                if check_result
                    .error_message
                    .as_ref()
                    .map(|msg| msg.contains("Provider not registered"))
                    .unwrap_or(false)
                    || health.consecutive_failures >= self.failure_threshold
                {
                    health.status = ProviderHealthStatus::Unhealthy;
                    warn!(
                        "Provider {} marked unhealthy after {} failures",
                        provider_id, health.consecutive_failures
                    );
                } else {
                    // Still healthy but incrementing failure count
                    debug!(
                        "Provider {} health check failed ({}/{})",
                        provider_id, health.consecutive_failures, self.failure_threshold
                    );
                }
            }
            ProviderHealthStatus::Unknown => {
                health.status = ProviderHealthStatus::Unknown;
            }
        }

        Ok(health.clone())
    }

    /// Get all provider health statuses
    pub async fn get_all_health(&self) -> HashMap<String, ProviderHealth> {
        let health_data = self.health_data.read().await;
        health_data.clone()
    }

    /// Get healthy providers from a list
    pub async fn get_healthy_providers(&self, providers: &[String]) -> Vec<String> {
        let mut healthy = Vec::new();
        for provider_id in providers {
            if self.is_healthy(provider_id).await {
                healthy.push(provider_id.clone());
            }
        }
        healthy
    }

    /// Reset health status for a provider
    pub async fn reset_provider(&self, provider_id: &str) {
        let mut health_data = self.health_data.write().await;
        if let Some(health) = health_data.get_mut(provider_id) {
            health.status = ProviderHealthStatus::Unknown;
            health.consecutive_failures = 0;
            health.total_checks = 0;
            health.response_time = None;
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        // Default implementation requires registry - use new() for real usage
        // This is kept for compatibility but should not be used in production
        let registry = Arc::new(ProviderRegistry::new());
        Self::new(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::di::registry::ProviderRegistry;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let registry = Arc::new(ProviderRegistry::new());
        let monitor = HealthMonitor::new(Arc::clone(&registry));
        assert!(!monitor.is_healthy("test-provider").await);
    }

    #[tokio::test]
    async fn test_provider_health_check_unregistered() {
        let registry = Arc::new(ProviderRegistry::new());
        let monitor = HealthMonitor::new(Arc::clone(&registry));

        // Check unregistered provider
        let result = monitor.check_provider("unregistered-provider").await;
        assert!(result.is_ok());

        // Should be unhealthy after check
        assert!(!monitor.is_healthy("unregistered-provider").await);

        let health = monitor.get_health("unregistered-provider").await.unwrap();
        assert_eq!(health.status, ProviderHealthStatus::Unhealthy); // Should be Unhealthy after failed check
    }

    #[tokio::test]
    async fn test_real_provider_health_checker() {
        let registry = Arc::new(ProviderRegistry::new());
        let checker = RealProviderHealthChecker::new(Arc::clone(&registry));

        // Test with unregistered provider
        let result = checker.check_health("nonexistent").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, ProviderHealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_get_healthy_providers() {
        let registry = Arc::new(ProviderRegistry::new());
        let monitor = HealthMonitor::new(Arc::clone(&registry));

        // Test with empty registry
        let providers = vec!["provider1".to_string(), "provider2".to_string()];

        let healthy = monitor.get_healthy_providers(&providers).await;
        assert_eq!(healthy.len(), 0); // No providers registered
    }
}
