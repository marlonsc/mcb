//! Provider routing ports.

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Health status for a provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProviderHealthStatus {
    /// Provider is responding optimally.
    #[default]
    Healthy,
    /// Provider is responding but with errors or latency.
    Degraded,
    /// Provider is unreachable or failing consistently.
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
    /// Create a new empty provider context with default sensitivities.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the operation type for this context.
    #[must_use]
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation_type = operation.into();
        self
    }

    /// Add a provider to the preferred list.
    #[must_use]
    pub fn prefer(mut self, provider: impl Into<String>) -> Self {
        self.preferred_providers.push(provider.into());
        self
    }

    /// Add a provider to the excluded list.
    #[must_use]
    pub fn exclude(mut self, provider: impl Into<String>) -> Self {
        self.excluded_providers.push(provider.into());
        self
    }
}

/// Provider routing interface
#[async_trait]
pub trait ProviderRouter: Send + Sync {
    /// Select the best embedding provider for the given context.
    async fn select_embedding_provider(&self, context: &ProviderContext) -> Result<String>;
    /// Select the best vector store provider for the given context.
    async fn select_vector_store_provider(&self, context: &ProviderContext) -> Result<String>;
    /// Get health status of a specific provider.
    async fn get_provider_health(&self, provider_id: &str) -> Result<ProviderHealthStatus>;
    /// Report an operation failure for a provider.
    async fn report_failure(&self, provider_id: &str, error: &str) -> Result<()>;
    /// Report an operation success for a provider.
    async fn report_success(&self, provider_id: &str) -> Result<()>;
    /// Get health status of all known providers.
    async fn get_all_health(&self) -> Result<HashMap<String, ProviderHealthStatus>>;
    /// Get detailed statistics for all providers.
    async fn get_stats(&self) -> HashMap<String, serde_json::Value>;
}
