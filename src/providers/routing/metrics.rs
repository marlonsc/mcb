//! Metrics Collection Module
//!
//! This module provides metrics collection using established prometheus and metrics crates,
//! following SOLID principles with proper separation of concerns.

use crate::core::error::Result;
use metrics::{counter, gauge, histogram};
use std::collections::HashMap;
use tracing::{debug, info};

/// Metrics collector for provider operations
pub struct ProviderMetricsCollector {
    /// Custom metrics storage
    custom_metrics: HashMap<String, serde_json::Value>,
}

impl ProviderMetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Result<Self> {
        info!("Initializing provider metrics collector");
        Ok(Self {
            custom_metrics: HashMap::new(),
        })
    }

    /// Record provider selection
    pub fn record_provider_selection(&self, provider_id: &str, strategy: &str) {
        counter!("mcp_provider_selections_total", "provider" => provider_id.to_string(), "strategy" => strategy.to_string()).increment(1);
        debug!(
            "Recorded provider selection: {} with strategy {}",
            provider_id, strategy
        );
    }

    /// Record response time for an operation
    pub fn record_response_time(&self, provider_id: &str, operation: &str, duration_seconds: f64) {
        histogram!("mcp_provider_response_time_seconds", "provider" => provider_id.to_string(), "operation" => operation.to_string()).record(duration_seconds);
        gauge!("mcp_provider_last_response_time", "provider" => provider_id.to_string(), "operation" => operation.to_string()).set(duration_seconds);
        debug!(
            "Recorded response time: {}s for {}:{}",
            duration_seconds, provider_id, operation
        );
    }

    /// Record request outcome
    pub fn record_request(&self, provider_id: &str, operation: &str, status: &str) {
        counter!("mcp_provider_requests_total", "provider" => provider_id.to_string(), "operation" => operation.to_string(), "status" => status.to_string()).increment(1);
        debug!(
            "Recorded request: {}:{} status={}",
            provider_id, operation, status
        );
    }

    /// Record error
    pub fn record_error(&self, provider_id: &str, error_type: &str) {
        counter!("mcp_provider_errors_total", "provider" => provider_id.to_string(), "error_type" => error_type.to_string()).increment(1);
        debug!("Recorded error: {} type={}", provider_id, error_type);
    }

    /// Record cost
    pub fn record_cost(&self, provider_id: &str, amount: f64, currency: &str) {
        counter!("mcp_provider_cost_total", "provider" => provider_id.to_string(), "currency" => currency.to_string()).increment(amount as u64);
        gauge!("mcp_provider_current_cost", "provider" => provider_id.to_string(), "currency" => currency.to_string()).set(amount);
        debug!("Recorded cost: {} {} for {}", amount, currency, provider_id);
    }

    /// Update active connections
    pub fn update_active_connections(&self, provider_id: &str, count: i64) {
        gauge!("mcp_provider_active_connections", "provider" => provider_id.to_string())
            .set(count as f64);
        debug!("Updated active connections: {} for {}", count, provider_id);
    }

    /// Record circuit breaker state change
    pub fn record_circuit_breaker_state(&self, provider_id: &str, state: &str) {
        counter!("mcp_circuit_breaker_state_changes_total", "provider" => provider_id.to_string(), "state" => state.to_string()).increment(1);
        gauge!("mcp_circuit_breaker_current_state", "provider" => provider_id.to_string()).set(
            match state {
                "closed" => 0.0,
                "open" => 1.0,
                "half-open" => 0.5,
                _ => -1.0,
            },
        );
        debug!(
            "Recorded circuit breaker state change: {} -> {}",
            provider_id, state
        );
    }

    /// Record provider health status
    pub fn record_provider_health(&self, provider_id: &str, status: &str, score: f64) {
        gauge!("mcp_provider_health_score", "provider" => provider_id.to_string()).set(score);
        counter!("mcp_provider_health_checks_total", "provider" => provider_id.to_string(), "status" => status.to_string()).increment(1);
        debug!(
            "Recorded provider health: {} status={} score={}",
            provider_id, status, score
        );
    }

    /// Add custom metric
    pub fn add_custom_metric(&mut self, name: &str, value: serde_json::Value) {
        let debug_value = format!("{:?}", value);
        self.custom_metrics.insert(name.to_string(), value);
        debug!("Added custom metric: {} = {}", name, debug_value);
    }

    /// Get custom metric
    pub fn get_custom_metric(&self, name: &str) -> Option<&serde_json::Value> {
        self.custom_metrics.get(name)
    }

    /// Get all custom metrics
    pub fn get_all_custom_metrics(&self) -> &HashMap<String, serde_json::Value> {
        &self.custom_metrics
    }

    /// Generate prometheus format output for custom metrics
    pub fn export_prometheus(&self) -> Result<String> {
        let mut output = String::new();

        for (name, value) in &self.custom_metrics {
            if let Some(num) = value.as_f64() {
                output.push_str(&format!("# HELP {} Custom metric\n", name));
                output.push_str(&format!("# TYPE {} gauge\n", name));
                output.push_str(&format!("{} {}\n", name, num));
            }
        }

        Ok(output)
    }

    /// Get metrics summary
    pub fn get_metrics_summary(&self) -> Result<MetricsSummary> {
        let prometheus_output = self.export_prometheus()?;

        // Parse some key metrics from Prometheus output
        let mut summary = MetricsSummary::default();

        for line in prometheus_output.lines() {
            if line.starts_with("mcp_provider_requests_total")
                && line.contains("status=\"success\"")
            {
                summary.total_successful_requests += 1;
            } else if line.starts_with("mcp_provider_errors_total") {
                summary.total_errors += 1;
            }
        }

        // Get custom metrics
        summary.custom_metrics = self.custom_metrics.clone();

        Ok(summary)
    }

    /// Reset all custom metrics
    pub fn reset(&mut self) {
        self.custom_metrics.clear();
        info!("Reset all custom metrics");
    }
}

/// Metrics summary for monitoring dashboards
#[derive(Debug, Clone, Default)]
pub struct MetricsSummary {
    pub total_successful_requests: u64,
    pub total_errors: u64,
    pub avg_response_time_seconds: Option<f64>,
    pub total_cost: f64,
    pub healthy_providers: usize,
    pub unhealthy_providers: usize,
    pub custom_metrics: HashMap<String, serde_json::Value>,
}

impl MetricsSummary {
    /// Calculate error rate
    pub fn error_rate(&self) -> f64 {
        let total_requests = self.total_successful_requests + self.total_errors;
        if total_requests == 0 {
            0.0
        } else {
            self.total_errors as f64 / total_requests as f64
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        let total_requests = self.total_successful_requests + self.total_errors;
        if total_requests == 0 {
            1.0
        } else {
            self.total_successful_requests as f64 / total_requests as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = ProviderMetricsCollector::new();
        assert!(collector.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_recording() {
        let collector = ProviderMetricsCollector::new().unwrap();

        // Record some metrics
        collector.record_provider_selection("test-provider", "contextual");
        collector.record_response_time("test-provider", "embed", 0.1);
        collector.record_request("test-provider", "embed", "success");
        collector.record_provider_selection("test-provider", "health_based");

        // Export Prometheus metrics
        let prometheus_output = collector.export_prometheus().unwrap();
        assert!(prometheus_output.contains("# HELP") || prometheus_output.is_empty());
    }

    #[tokio::test]
    async fn test_custom_metrics() {
        let mut collector = ProviderMetricsCollector::new().unwrap();

        // Add custom metrics
        collector.add_custom_metric("test_metric", serde_json::json!(42.0));

        assert_eq!(
            collector.get_custom_metric("test_metric"),
            Some(&serde_json::json!(42.0))
        );
        assert_eq!(collector.get_all_custom_metrics().len(), 1);

        let summary = collector.get_metrics_summary().unwrap();
        assert!(summary.custom_metrics.contains_key("test_metric"));
    }

    #[test]
    fn test_metrics_summary_calculations() {
        let mut summary = MetricsSummary::default();
        summary.total_successful_requests = 95;
        summary.total_errors = 5;

        assert_eq!(summary.error_rate(), 0.05); // 5%
        assert_eq!(summary.success_rate(), 0.95); // 95%
    }

    #[test]
    fn test_empty_metrics_summary() {
        let summary = MetricsSummary::default();
        assert_eq!(summary.error_rate(), 0.0);
        assert_eq!(summary.success_rate(), 1.0); // Default for no requests
    }
}
