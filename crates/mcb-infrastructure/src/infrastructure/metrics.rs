//! System Metrics Adapter
//!
//! Null implementation of the system metrics port for testing.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::{SystemMetrics, SystemMetricsCollectorInterface};

/// Null implementation for testing
pub struct NullSystemMetricsCollector;

impl NullSystemMetricsCollector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullSystemMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemMetricsCollectorInterface for NullSystemMetricsCollector {
    async fn collect(&self) -> Result<SystemMetrics> {
        Ok(SystemMetrics::default())
    }
    fn cpu_usage(&self) -> f64 {
        0.0
    }
    fn memory_usage(&self) -> f64 {
        0.0
    }
}

/// No-op metrics provider that discards all metrics
#[derive(Debug, Clone, Default)]
pub struct NullMetricsObservabilityProvider;

impl NullMetricsObservabilityProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn arc() -> std::sync::Arc<dyn mcb_domain::ports::providers::MetricsProvider> {
        std::sync::Arc::new(Self::new())
    }
}

#[async_trait]
impl mcb_domain::ports::providers::MetricsProvider for NullMetricsObservabilityProvider {
    fn name(&self) -> &str {
        "null"
    }

    async fn increment(
        &self,
        _name: &str,
        _labels: &mcb_domain::ports::providers::MetricLabels,
    ) -> mcb_domain::ports::providers::MetricsResult<()> {
        Ok(())
    }

    async fn increment_by(
        &self,
        _name: &str,
        _value: f64,
        _labels: &mcb_domain::ports::providers::MetricLabels,
    ) -> mcb_domain::ports::providers::MetricsResult<()> {
        Ok(())
    }

    async fn gauge(
        &self,
        _name: &str,
        _value: f64,
        _labels: &mcb_domain::ports::providers::MetricLabels,
    ) -> mcb_domain::ports::providers::MetricsResult<()> {
        Ok(())
    }

    async fn histogram(
        &self,
        _name: &str,
        _value: f64,
        _labels: &mcb_domain::ports::providers::MetricLabels,
    ) -> mcb_domain::ports::providers::MetricsResult<()> {
        Ok(())
    }
}
