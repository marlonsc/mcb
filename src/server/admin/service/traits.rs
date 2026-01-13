//! Admin service trait definition
//!
//! Defines the interface for admin service operations.

use super::types::*;
use async_trait::async_trait;
use shaku::Interface;
use std::collections::HashMap;

/// Core admin service trait
#[async_trait]
pub trait AdminService: Interface + Send + Sync {
    /// Get system information
    async fn get_system_info(&self) -> Result<SystemInfo, AdminError>;

    /// Get all registered providers
    async fn get_providers(&self) -> Result<Vec<ProviderInfo>, AdminError>;

    /// Add a new provider
    async fn add_provider(
        &self,
        provider_type: &str,
        config: serde_json::Value,
    ) -> Result<ProviderInfo, AdminError>;

    /// Remove a provider
    async fn remove_provider(&self, provider_id: &str) -> Result<(), AdminError>;

    /// Search indexed content
    async fn search(
        &self,
        query: &str,
        collection: Option<&str>,
        limit: Option<usize>,
    ) -> Result<SearchResults, AdminError>;

    /// Get indexing status
    async fn get_indexing_status(&self) -> Result<IndexingStatus, AdminError>;

    /// Get performance metrics
    async fn get_performance_metrics(&self) -> Result<PerformanceMetricsData, AdminError>;

    /// Get dashboard data
    async fn get_dashboard_data(&self) -> Result<DashboardData, AdminError>;

    /// Get current system configuration
    async fn get_configuration(&self) -> Result<ConfigurationData, AdminError>;

    /// Update configuration dynamically
    async fn update_configuration(
        &self,
        updates: HashMap<String, serde_json::Value>,
        user: &str,
    ) -> Result<ConfigurationUpdateResult, AdminError>;

    /// Validate configuration changes
    async fn validate_configuration(
        &self,
        updates: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<String>, AdminError>;

    /// Get configuration change history
    async fn get_configuration_history(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<ConfigurationChange>, AdminError>;

    /// Get recent log entries with filtering
    async fn get_logs(&self, filter: LogFilter) -> Result<LogEntries, AdminError>;

    /// Export logs to file
    async fn export_logs(
        &self,
        filter: LogFilter,
        format: LogExportFormat,
    ) -> Result<String, AdminError>;

    /// Get log statistics
    async fn get_log_stats(&self) -> Result<LogStats, AdminError>;

    /// Clear system cache
    async fn clear_cache(&self, cache_type: CacheType) -> Result<MaintenanceResult, AdminError>;

    /// Restart provider connection
    async fn restart_provider(&self, provider_id: &str) -> Result<MaintenanceResult, AdminError>;

    /// Reconfigure a provider without restart (hot-update configuration)
    async fn reconfigure_provider(
        &self,
        provider_id: &str,
        config: serde_json::Value,
    ) -> Result<MaintenanceResult, AdminError>;

    /// Rebuild search index
    async fn rebuild_index(&self, index_id: &str) -> Result<MaintenanceResult, AdminError>;

    /// Cleanup old data
    async fn cleanup_data(
        &self,
        cleanup_config: CleanupConfig,
    ) -> Result<MaintenanceResult, AdminError>;

    /// Run comprehensive health check
    async fn run_health_check(&self) -> Result<HealthCheckResult, AdminError>;

    /// Test provider connectivity
    async fn test_provider_connectivity(
        &self,
        provider_id: &str,
    ) -> Result<ConnectivityTestResult, AdminError>;

    /// Run performance benchmark
    async fn run_performance_test(
        &self,
        test_config: PerformanceTestConfig,
    ) -> Result<PerformanceTestResult, AdminError>;

    /// Create system backup
    async fn create_backup(&self, backup_config: BackupConfig) -> Result<BackupResult, AdminError>;

    /// List available backups
    async fn list_backups(&self) -> Result<Vec<BackupInfo>, AdminError>;

    /// Restore from backup
    async fn restore_backup(&self, backup_id: &str) -> Result<RestoreResult, AdminError>;

    // === Subsystem Control Methods (ADR-007) ===

    /// Get all subsystems and their current status
    async fn get_subsystems(&self) -> Result<Vec<SubsystemInfo>, AdminError>;

    /// Send a control signal to a subsystem
    async fn send_subsystem_signal(
        &self,
        subsystem_id: &str,
        signal: SubsystemSignal,
    ) -> Result<SignalResult, AdminError>;

    /// Get all registered HTTP routes
    async fn get_routes(&self) -> Result<Vec<RouteInfo>, AdminError>;

    /// Reload router configuration
    async fn reload_routes(&self) -> Result<MaintenanceResult, AdminError>;

    /// Persist current runtime configuration to file
    async fn persist_configuration(&self) -> Result<ConfigPersistResult, AdminError>;

    /// Get difference between runtime and file configuration
    async fn get_config_diff(&self) -> Result<ConfigDiff, AdminError>;
}
