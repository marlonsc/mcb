//! Failover Management Module
//!
//! This module provides failover management capabilities using established patterns,
//! following SOLID principles with proper separation of concerns.

use crate::core::error::{Error, Result};
use crate::di::registry::ProviderRegistry;
use crate::providers::routing::health::HealthMonitor;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Failover strategy trait
#[async_trait::async_trait]
pub trait FailoverStrategy: Send + Sync {
    /// Select the best available provider from candidates
    async fn select_provider(
        &self,
        candidates: &[String],
        health_monitor: &HealthMonitor,
        context: &FailoverContext,
    ) -> Result<String>;
}

/// Context for failover decisions
#[derive(Debug, Clone)]
pub struct FailoverContext {
    /// Operation type
    pub operation_type: String,
    /// Preferred providers (higher priority)
    pub preferred_providers: Vec<String>,
    /// Excluded providers
    pub excluded_providers: Vec<String>,
    /// Maximum failover attempts
    pub max_attempts: usize,
    /// Current attempt number
    pub current_attempt: usize,
}

impl Default for FailoverContext {
    fn default() -> Self {
        Self {
            operation_type: "general".to_string(),
            preferred_providers: Vec::new(),
            excluded_providers: Vec::new(),
            max_attempts: 3,
            current_attempt: 0,
        }
    }
}

/// Priority-based failover strategy
#[derive(Default)]
pub struct PriorityBasedStrategy {
    /// Provider priorities (lower number = higher priority)
    priorities: HashMap<String, u32>,
}

impl PriorityBasedStrategy {
    /// Create a new priority-based strategy
    pub fn new() -> Self {
        Self::default()
    }

    /// Set priority for a provider (lower number = higher priority)
    pub fn set_priority(&mut self, provider_id: &str, priority: u32) {
        self.priorities.insert(provider_id.to_string(), priority);
    }

    /// Get default priority for providers without explicit priority
    fn get_default_priority(&self, provider_id: &str) -> u32 {
        // Default priorities based on provider characteristics
        match provider_id {
            id if id.contains("ollama") => 1,    // Local, fast, free
            id if id.contains("openai") => 2,    // Cloud, reliable
            id if id.contains("anthropic") => 3, // Cloud, good quality
            id if id.contains("gemini") => 4,    // Cloud, good balance
            _ => 100,                            // Default priority
        }
    }
}

#[async_trait::async_trait]
impl FailoverStrategy for PriorityBasedStrategy {
    async fn select_provider(
        &self,
        candidates: &[String],
        health_monitor: &HealthMonitor,
        _context: &FailoverContext,
    ) -> Result<String> {
        let mut scored_providers: Vec<(String, u32)> = Vec::new();

        for candidate in candidates {
            if !health_monitor.is_healthy(candidate).await {
                continue;
            }

            let priority = self
                .priorities
                .get(candidate)
                .copied()
                .unwrap_or_else(|| self.get_default_priority(candidate));

            scored_providers.push((candidate.clone(), priority));
        }

        if scored_providers.is_empty() {
            return Err(Error::not_found("No healthy providers available"));
        }

        // Sort by priority (lower priority number = higher preference)
        scored_providers.sort_by_key(|(_, priority)| *priority);

        let selected = scored_providers[0].0.clone();
        debug!(
            "Selected provider by priority: {} (priority: {})",
            selected, scored_providers[0].1
        );

        Ok(selected)
    }
}

/// Round-robin failover strategy
pub struct RoundRobinStrategy {
    /// Current index for round-robin selection
    index: std::sync::atomic::AtomicUsize,
}

impl RoundRobinStrategy {
    /// Create a new round-robin strategy
    pub fn new() -> Self {
        Self {
            index: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl FailoverStrategy for RoundRobinStrategy {
    async fn select_provider(
        &self,
        candidates: &[String],
        health_monitor: &HealthMonitor,
        _context: &FailoverContext,
    ) -> Result<String> {
        // Filter candidates by actual health status
        let mut healthy_candidates = Vec::new();
        for candidate in candidates {
            if health_monitor.is_healthy(candidate).await {
                healthy_candidates.push(candidate.clone());
            }
        }

        if healthy_candidates.is_empty() {
            return Err(Error::not_found("No healthy providers available"));
        }

        if healthy_candidates.is_empty() {
            return Err(Error::not_found("No healthy providers available"));
        }

        let current_index = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let selected = healthy_candidates[current_index % healthy_candidates.len()].clone();

        debug!("Selected provider by round-robin: {}", selected);
        Ok(selected)
    }
}

/// Failover strategy types for configuration
#[derive(Debug, Clone)]
pub enum FailoverStrategyType {
    /// Priority-based strategy with configurable priorities
    PriorityBased { priorities: HashMap<String, u32> },
    /// Round-robin strategy
    RoundRobin,
}

/// Failover manager that coordinates failover strategies
pub struct FailoverManager {
    /// Health monitor for checking provider status
    health_monitor: Arc<HealthMonitor>,
    /// Current failover strategy implementation
    strategy: Box<dyn FailoverStrategy>,
    /// Maximum number of failover attempts
    max_attempts: usize,
}

impl FailoverManager {
    /// Create a new failover manager with default priority-based strategy
    pub fn new(health_monitor: Arc<HealthMonitor>) -> Self {
        let strategy = Box::new(PriorityBasedStrategy::new());
        Self {
            health_monitor,
            strategy,
            max_attempts: 3,
        }
    }

    /// Create a new failover manager with custom strategy
    pub fn with_strategy(
        health_monitor: Arc<HealthMonitor>,
        strategy: Box<dyn FailoverStrategy>,
    ) -> Self {
        Self {
            health_monitor,
            strategy,
            max_attempts: 3,
        }
    }

    /// Select the best available provider
    pub async fn select_provider(
        &self,
        candidates: &[String],
        context: &FailoverContext,
    ) -> Result<String> {
        if candidates.is_empty() {
            return Err(Error::not_found("No provider candidates available"));
        }

        self.strategy
            .select_provider(candidates, &self.health_monitor, context)
            .await
    }

    /// Execute operation with automatic failover
    pub async fn execute_with_failover<F, Fut, T>(
        &self,
        candidates: &[String],
        context: &FailoverContext,
        operation: F,
    ) -> Result<T>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
        T: Send,
    {
        let mut attempts = 0;
        let mut last_error = None;
        let mut tried_providers = Vec::new();

        while attempts < self.max_attempts {
            let mut failover_context = context.clone();
            failover_context.current_attempt = attempts;
            failover_context
                .excluded_providers
                .extend(tried_providers.clone());

            let provider = match self.select_provider(candidates, &failover_context).await {
                Ok(provider) => provider,
                Err(e) => {
                    warn!(
                        "Failed to select provider on attempt {}: {}",
                        attempts + 1,
                        e
                    );
                    attempts += 1;
                    last_error = Some(e);
                    continue;
                }
            };

            tried_providers.push(provider.clone());

            debug!(
                "Attempting operation with provider: {} (attempt {})",
                provider,
                attempts + 1
            );

            match operation(provider.clone()).await {
                Ok(result) => {
                    if attempts > 0 {
                        info!(
                            "Operation succeeded with provider {} after {} attempts",
                            provider,
                            attempts + 1
                        );
                    }
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Operation failed with provider {}: {}", provider, e);
                    attempts += 1;
                    last_error = Some(e);

                    // Mark provider as potentially unhealthy
                    if let Err(health_err) = self.health_monitor.check_provider(&provider).await {
                        debug!(
                            "Failed to update health status for {}: {}",
                            provider, health_err
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::generic("All failover attempts exhausted")))
    }

    /// Get failover candidates excluding unhealthy providers
    pub async fn get_failover_candidates(
        &self,
        all_providers: &[String],
        exclude: &[String],
    ) -> Vec<String> {
        let mut candidates = Vec::new();

        for provider in all_providers {
            if exclude.contains(provider) {
                continue;
            }

            if self.health_monitor.is_healthy(provider).await {
                candidates.push(provider.clone());
            }
        }

        candidates
    }

    /// Check if failover is needed for a provider
    pub async fn should_failover(&self, provider_id: &str) -> bool {
        !self.health_monitor.is_healthy(provider_id).await
    }

    /// Set the failover strategy
    pub fn set_strategy(&mut self, strategy: Box<dyn FailoverStrategy>) {
        self.strategy = strategy;
        info!("Failover strategy updated");
    }

    /// Configure priority-based strategy
    pub fn configure_priorities(&mut self, priorities: HashMap<String, u32>) {
        // For now, replace the current strategy with a new priority-based one
        // In a more advanced implementation, we could use downcasting
        let priority_count = priorities.len();
        let mut new_strategy = PriorityBasedStrategy::new();
        new_strategy.priorities = priorities;
        self.strategy = Box::new(new_strategy);
        info!(
            "Provider priorities configured with {} entries",
            priority_count
        );
    }
}

impl FailoverManager {
    /// Create a new failover manager with registry
    pub fn with_registry(registry: Arc<ProviderRegistry>) -> Self {
        let health_monitor = Arc::new(HealthMonitor::new(Arc::clone(&registry)));
        Self::new(health_monitor)
    }
}

impl Default for FailoverManager {
    fn default() -> Self {
        // Default implementation requires registry - use with_registry() for real usage
        // This is kept for compatibility but should not be used in production
        let registry = Arc::new(ProviderRegistry::new());
        Self::with_registry(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_priority_based_failover() {
        let registry = Arc::new(ProviderRegistry::new());
        let health_monitor = Arc::new(HealthMonitor::new(Arc::clone(&registry)));

        // Mark providers as healthy (they will be unhealthy since not registered)
        let _ = health_monitor.check_provider("primary").await;
        let _ = health_monitor.check_provider("secondary").await;
        let _ = health_monitor.check_provider("tertiary").await;

        let mut strategy = PriorityBasedStrategy::new();

        // Set priorities
        strategy.set_priority("primary", 1);
        strategy.set_priority("secondary", 2);
        strategy.set_priority("tertiary", 3);

        let manager = FailoverManager::with_strategy(health_monitor.clone(), Box::new(strategy));

        let candidates = vec![
            "primary".to_string(),
            "secondary".to_string(),
            "tertiary".to_string(),
        ];

        let context = FailoverContext::default();

        // Should fail since no providers are healthy
        let result = manager.select_provider(&candidates, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_round_robin_failover() {
        let registry = Arc::new(ProviderRegistry::new());
        let health_monitor = Arc::new(HealthMonitor::new(Arc::clone(&registry)));
        let strategy = RoundRobinStrategy::new();
        let manager = FailoverManager::with_strategy(health_monitor, Box::new(strategy));

        let candidates = vec![
            "provider1".to_string(),
            "provider2".to_string(),
            "provider3".to_string(),
        ];

        let context = FailoverContext::default();

        // Should fail since providers are not healthy
        let result = manager.select_provider(&candidates, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_failover_candidates() {
        let registry = Arc::new(ProviderRegistry::new());
        let health_monitor = Arc::new(HealthMonitor::new(Arc::clone(&registry)));
        let manager = FailoverManager::new(Arc::clone(&health_monitor));

        // Providers will be unhealthy since not registered
        let _ = health_monitor.check_provider("healthy1").await;
        let _ = health_monitor.check_provider("healthy2").await;

        let all_providers = vec![
            "healthy1".to_string(),
            "healthy2".to_string(),
            "unhealthy".to_string(),
        ];

        let exclude = vec!["unhealthy".to_string()];

        let candidates = manager
            .get_failover_candidates(&all_providers, &exclude)
            .await;
        assert_eq!(candidates.len(), 0); // No providers are healthy
    }

    #[tokio::test]
    async fn test_failover_manager_creation() {
        let registry = Arc::new(ProviderRegistry::new());
        let _manager = FailoverManager::with_registry(Arc::clone(&registry));
        // Test passes if creation succeeds
    }

    #[tokio::test]
    async fn test_execute_with_failover() {
        use crate::providers::routing::health::ProviderHealthChecker;

        // Create a mock health checker that marks test providers as healthy
        struct MockHealthChecker;
        #[async_trait::async_trait]
        impl ProviderHealthChecker for MockHealthChecker {
            async fn check_health(
                &self,
                provider_id: &str,
            ) -> crate::core::error::Result<crate::providers::routing::health::HealthCheckResult>
            {
                use crate::providers::routing::health::{HealthCheckResult, ProviderHealthStatus};
                use std::time::Duration;
                Ok(HealthCheckResult {
                    provider_id: provider_id.to_string(),
                    status: ProviderHealthStatus::Healthy,
                    response_time: Duration::from_millis(10),
                    error_message: None,
                })
            }
        }

        let mock_checker = Arc::new(MockHealthChecker);
        let health_monitor = Arc::new(HealthMonitor::with_checker(mock_checker));

        // Initialize health status for test providers
        health_monitor.check_provider("failing").await.unwrap();
        health_monitor.check_provider("success").await.unwrap();

        // Use round-robin strategy to ensure it tries different providers
        let strategy = RoundRobinStrategy::new();
        let manager = FailoverManager::with_strategy(health_monitor, Box::new(strategy));

        let candidates = vec!["failing".to_string(), "success".to_string()];
        let context = FailoverContext {
            max_attempts: 2,
            ..Default::default()
        };

        let result = manager
            .execute_with_failover(&candidates, &context, |provider| async move {
                if provider == "failing" {
                    Err(Error::generic("Operation failed"))
                } else {
                    Ok(format!("Success with {}", provider))
                }
            })
            .await;

        // Debug: print the error if it fails
        if let Err(e) = &result {
            println!("Test failed with error: {}", e);
        }

        assert!(result.is_ok());
    }
}
