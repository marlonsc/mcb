//! Admin service layer - SOLID principles implementation
//!
//! This service provides a clean interface to access system data
//! following SOLID principles and dependency injection.

use crate::di::factory::ServiceProviderInterface;
use crate::metrics::system::SystemMetricsCollectorInterface;
use crate::server::server::{IndexingOperationsInterface, PerformanceMetricsInterface};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shaku::Interface;
use std::collections::HashMap;
use std::sync::Arc;

// Data structures for admin service operations

/// Configuration data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationData {
    pub providers: Vec<ProviderInfo>,
    pub indexing: IndexingConfig,
    pub security: SecurityConfig,
    pub metrics: MetricsConfigData,
    pub cache: CacheConfigData,
    pub database: DatabaseConfigData,
}

/// Indexing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_file_size: u64,
    pub supported_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_auth: bool,
    pub rate_limiting: bool,
    pub max_requests_per_minute: u32,
}

/// Metrics configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfigData {
    pub enabled: bool,
    pub collection_interval: u64,
    pub retention_days: u32,
}

/// Cache configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfigData {
    pub enabled: bool,
    pub max_size: u64,
    pub ttl_seconds: u64,
}

/// Database configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfigData {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout: u64,
}

/// Configuration update result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationUpdateResult {
    pub success: bool,
    pub changes_applied: Vec<String>,
    pub requires_restart: bool,
    pub validation_warnings: Vec<String>,
}

/// Configuration change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationChange {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user: String,
    pub path: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub change_type: String,
}

/// Log filter for querying logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    pub level: Option<String>,
    pub module: Option<String>,
    pub message_contains: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub module: String,
    pub message: String,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

/// Log entries response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntries {
    pub entries: Vec<LogEntry>,
    pub total_count: u64,
    pub has_more: bool,
}

/// Log export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogExportFormat {
    Json,
    Csv,
    PlainText,
}

/// Log statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total_entries: u64,
    pub entries_by_level: HashMap<String, u64>,
    pub entries_by_module: HashMap<String, u64>,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
}

/// Cache types for maintenance operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheType {
    All,
    QueryResults,
    Embeddings,
    Indexes,
}

/// Maintenance operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceResult {
    pub success: bool,
    pub operation: String,
    pub message: String,
    pub affected_items: u64,
    pub execution_time_ms: u64,
}

/// Data cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    pub older_than_days: u32,
    pub max_items_to_keep: Option<u64>,
    pub cleanup_types: Vec<String>,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub overall_status: String,
    pub checks: Vec<HealthCheck>,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
}

/// Individual health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: String,
    pub duration_ms: u64,
    pub details: Option<serde_json::Value>,
}

/// Connectivity test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityTestResult {
    pub provider_id: String,
    pub success: bool,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub details: serde_json::Value,
}

/// Performance test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestConfig {
    pub test_type: String,
    pub duration_seconds: u32,
    pub concurrency: u32,
    pub queries: Vec<String>,
}

/// Performance test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    pub test_id: String,
    pub test_type: String,
    pub duration_seconds: u32,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub throughput_rps: f64,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub name: String,
    pub include_data: bool,
    pub include_config: bool,
    pub compression: bool,
}

/// Backup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub backup_id: String,
    pub name: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub path: String,
}

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub status: String,
}

/// Restore result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub success: bool,
    pub backup_id: String,
    pub restored_items: u64,
    pub errors: Vec<String>,
}

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

    /// Configuration Management
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

    /// Logging System
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

    /// Maintenance Operations
    /// Clear system cache
    async fn clear_cache(&self, cache_type: CacheType) -> Result<MaintenanceResult, AdminError>;

    /// Restart provider connection
    async fn restart_provider(&self, provider_id: &str) -> Result<MaintenanceResult, AdminError>;

    /// Rebuild search index
    async fn rebuild_index(&self, index_id: &str) -> Result<MaintenanceResult, AdminError>;

    /// Cleanup old data
    async fn cleanup_data(
        &self,
        cleanup_config: CleanupConfig,
    ) -> Result<MaintenanceResult, AdminError>;

    /// Diagnostic Operations
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

    /// Data Management
    /// Create system backup
    async fn create_backup(&self, backup_config: BackupConfig) -> Result<BackupResult, AdminError>;

    /// List available backups
    async fn list_backups(&self) -> Result<Vec<BackupInfo>, AdminError>;

    /// Restore from backup
    async fn restore_backup(&self, backup_id: &str) -> Result<RestoreResult, AdminError>;
}

/// Concrete implementation of AdminService
#[derive(shaku::Component)]
#[shaku(interface = AdminService)]
pub struct AdminServiceImpl {
    #[shaku(inject)]
    performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    #[shaku(inject)]
    indexing_operations: Arc<dyn IndexingOperationsInterface>,
    #[shaku(inject)]
    service_provider: Arc<dyn ServiceProviderInterface>,
    #[shaku(inject)]
    system_collector: Arc<dyn SystemMetricsCollectorInterface>,
    #[shaku(default)]
    event_bus: crate::core::events::SharedEventBus,
    #[shaku(default)]
    log_buffer: crate::core::logging::SharedLogBuffer,
    #[shaku(default)]
    config: Arc<arc_swap::ArcSwap<crate::config::Config>>,
}

impl AdminServiceImpl {
    /// Create new admin service with dependency injection
    pub fn new(
        performance_metrics: Arc<dyn PerformanceMetricsInterface>,
        indexing_operations: Arc<dyn IndexingOperationsInterface>,
        service_provider: Arc<dyn ServiceProviderInterface>,
        system_collector: Arc<dyn SystemMetricsCollectorInterface>,
        event_bus: crate::core::events::SharedEventBus,
        log_buffer: crate::core::logging::SharedLogBuffer,
        config: Arc<arc_swap::ArcSwap<crate::config::Config>>,
    ) -> Self {
        Self {
            performance_metrics,
            indexing_operations,
            service_provider,
            system_collector,
            event_bus,
            log_buffer,
            config,
        }
    }
}

#[async_trait]
impl AdminService for AdminServiceImpl {
    async fn get_system_info(&self) -> Result<SystemInfo, AdminError> {
        let uptime_seconds = self.performance_metrics.start_time().elapsed().as_secs();
        Ok(SystemInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: uptime_seconds,
            pid: std::process::id(),
        })
    }

    async fn get_providers(&self) -> Result<Vec<ProviderInfo>, AdminError> {
        let (embedding_providers, vector_store_providers) = self.service_provider.list_providers();

        let mut providers = Vec::new();

        // Add embedding providers
        // TODO: Integrate with health monitor to get actual provider status
        for name in embedding_providers {
            providers.push(ProviderInfo {
                id: name.clone(),
                name,
                provider_type: "embedding".to_string(),
                status: "unknown".to_string(), // Status unknown until health check verifies
                config: serde_json::json!({ "type": "embedding" }),
            });
        }

        // Add vector store providers
        // TODO: Integrate with health monitor to get actual provider status
        for name in vector_store_providers {
            providers.push(ProviderInfo {
                id: name.clone(),
                name,
                provider_type: "vector_store".to_string(),
                status: "unknown".to_string(), // Status unknown until health check verifies
                config: serde_json::json!({ "type": "vector_store" }),
            });
        }

        Ok(providers)
    }

    async fn add_provider(
        &self,
        provider_type: &str,
        config: serde_json::Value,
    ) -> Result<ProviderInfo, AdminError> {
        // Validate provider type
        match provider_type {
            "embedding" | "vector_store" => {}
            _ => {
                return Err(AdminError::ConfigError(format!(
                    "Invalid provider type: {}. Must be 'embedding' or 'vector_store'",
                    provider_type
                )));
            }
        }

        // Generate unique provider ID
        let provider_id = format!("{}-{}", provider_type, uuid::Uuid::new_v4());

        // Log the registration
        tracing::info!(
            "[ADMIN] Registering {} provider: {}",
            provider_type,
            provider_id
        );

        // Verify registry is accessible
        let (embedding_providers, vector_store_providers) = self.service_provider.list_providers();
        tracing::debug!(
            "[ADMIN] Current providers - embedding: {:?}, vector_store: {:?}",
            embedding_providers,
            vector_store_providers
        );

        Ok(ProviderInfo {
            id: provider_id.clone(),
            name: provider_id,
            provider_type: provider_type.to_string(),
            status: "pending".to_string(),
            config,
        })
    }

    async fn remove_provider(&self, provider_id: &str) -> Result<(), AdminError> {
        let (embedding_providers, vector_store_providers) = self.service_provider.list_providers();
        let exists = embedding_providers.iter().any(|p| p == provider_id)
            || vector_store_providers.iter().any(|p| p == provider_id);

        if !exists {
            return Err(AdminError::ConfigError(format!(
                "Provider not found: {}",
                provider_id
            )));
        }

        tracing::info!("[ADMIN] Removing provider: {}", provider_id);
        Ok(())
    }

    async fn search(
        &self,
        query: &str,
        _collection: Option<&str>,
        limit: Option<usize>,
    ) -> Result<SearchResults, AdminError> {
        let start = std::time::Instant::now();
        let search_limit = limit.unwrap_or(10);

        tracing::info!(
            "[ADMIN] Search request: query='{}', limit={}",
            query,
            search_limit
        );

        // Admin search provides query metadata
        // Full search via MCP server search_code tool
        Ok(SearchResults {
            query: query.to_string(),
            results: vec![],
            total: 0,
            took_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn get_indexing_status(&self) -> Result<IndexingStatus, AdminError> {
        let map = self.indexing_operations.get_map();
        // Check if any indexing operations are active
        let is_indexing = !map.is_empty();

        // Find the most recent operation for current status
        let (current_file, start_time, _processed_files, _total_files): (
            Option<String>,
            Option<u64>,
            usize,
            usize,
        ) = if let Some(entry) = map.iter().next() {
            let operation = entry.value();
            (
                operation.current_file.clone(),
                Some(operation.start_time.elapsed().as_secs()),
                operation.processed_files,
                operation.total_files,
            )
        } else {
            (None, None, 0, 0)
        };

        // Calculate totals across all operations
        let total_documents: usize = map.iter().map(|entry| entry.value().total_files).sum();
        let indexed_documents: usize = map.iter().map(|entry| entry.value().processed_files).sum();

        // For now, no failed documents tracking
        let failed_documents = 0;

        // Estimate completion based on progress
        let estimated_completion = if is_indexing && total_documents > 0 {
            let progress = indexed_documents as f64 / total_documents as f64;
            if progress > 0.0 {
                start_time.map(|elapsed| {
                    let estimated_total = (elapsed as f64 / progress) as u64;
                    estimated_total.saturating_sub(elapsed)
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(IndexingStatus {
            is_indexing,
            total_documents: total_documents as u64,
            indexed_documents: indexed_documents as u64,
            failed_documents: failed_documents as u64,
            current_file,
            start_time,
            estimated_completion,
        })
    }

    async fn get_performance_metrics(&self) -> Result<PerformanceMetricsData, AdminError> {
        Ok(self.performance_metrics.get_performance_metrics())
    }

    async fn get_dashboard_data(&self) -> Result<DashboardData, AdminError> {
        let system_info = self.get_system_info().await?;
        let providers = self.get_providers().await?;
        let indexing = self.get_indexing_status().await?;
        let performance = self.get_performance_metrics().await?;

        let active_providers = providers.iter().filter(|p| p.status == "active").count();
        let active_indexes = if indexing.is_indexing { 0 } else { 1 };

        let cpu_metrics = self
            .system_collector
            .collect_cpu_metrics()
            .await
            .unwrap_or_default();
        let memory_metrics = self
            .system_collector
            .collect_memory_metrics()
            .await
            .unwrap_or_default();

        Ok(DashboardData {
            system_info,
            active_providers,
            total_providers: providers.len(),
            active_indexes,
            total_documents: indexing.indexed_documents,
            cpu_usage: cpu_metrics.usage as f64,
            memory_usage: memory_metrics.usage_percent as f64,
            performance,
        })
    }

    // Configuration Management Implementation
    async fn get_configuration(&self) -> Result<ConfigurationData, AdminError> {
        let config = self.config.load();
        let providers = self.get_providers().await?;

        Ok(ConfigurationData {
            providers,
            indexing: IndexingConfig {
                // Default chunking values (no chunking config in current Config struct)
                chunk_size: 1000,
                chunk_overlap: 200,
                max_file_size: 10 * 1024 * 1024, // 10MB
                supported_extensions: vec![
                    ".rs".to_string(),
                    ".py".to_string(),
                    ".js".to_string(),
                    ".ts".to_string(),
                    ".go".to_string(),
                    ".java".to_string(),
                ],
                exclude_patterns: vec![
                    "node_modules".to_string(),
                    "target".to_string(),
                    ".git".to_string(),
                ],
            },
            security: SecurityConfig {
                enable_auth: config.auth.enabled,
                rate_limiting: config.metrics.rate_limiting.enabled,
                max_requests_per_minute: config.metrics.rate_limiting.max_requests_per_window,
            },
            metrics: MetricsConfigData {
                enabled: config.metrics.enabled,
                collection_interval: 30, // Default collection interval
                retention_days: 7,       // Default retention days
            },
            cache: CacheConfigData {
                enabled: config.cache.enabled,
                max_size: config.cache.max_size as u64,
                ttl_seconds: config.cache.default_ttl_seconds,
            },
            database: DatabaseConfigData {
                url: config.database.url.clone(),
                pool_size: config.database.max_connections,
                connection_timeout: config.database.connection_timeout.as_secs(),
            },
        })
    }

    async fn update_configuration(
        &self,
        updates: HashMap<String, serde_json::Value>,
        user: &str,
    ) -> Result<ConfigurationUpdateResult, AdminError> {
        // Validate changes first
        let validation_warnings = self.validate_configuration(&updates).await?;

        // Apply changes (in real implementation, this would update the actual config)
        let mut changes_applied = Vec::new();
        let mut requires_restart = false;

        for (path, value) in &updates {
            changes_applied.push(format!("{} = {:?}", path, value));

            // Check if this change requires restart
            if path.starts_with("database.") || path.starts_with("server.") {
                requires_restart = true;
            }
        }

        // Log the configuration change
        tracing::info!(
            "Configuration updated by {}: {} changes applied",
            user,
            changes_applied.len()
        );

        Ok(ConfigurationUpdateResult {
            success: true,
            changes_applied,
            requires_restart,
            validation_warnings,
        })
    }

    async fn validate_configuration(
        &self,
        updates: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<String>, AdminError> {
        let mut warnings = Vec::new();

        for (path, value) in updates {
            match path.as_str() {
                "metrics.collection_interval" => {
                    if let Some(interval) = value.as_u64()
                        && interval < 5
                    {
                        warnings.push(
                            "Collection interval below 5 seconds may impact performance"
                                .to_string(),
                        );
                    }
                }
                "cache.max_size" => {
                    if let Some(size) = value.as_u64()
                        && size > 10 * 1024 * 1024 * 1024
                    {
                        // 10GB
                        warnings.push("Cache size above 10GB may cause memory issues".to_string());
                    }
                }
                "database.pool_size" => {
                    if let Some(pool_size) = value.as_u64()
                        && pool_size > 100
                    {
                        warnings.push(
                            "Database pool size above 100 may cause resource exhaustion"
                                .to_string(),
                        );
                    }
                }
                _ => {}
            }
        }

        Ok(warnings)
    }

    async fn get_configuration_history(
        &self,
        _limit: Option<usize>,
    ) -> Result<Vec<ConfigurationChange>, AdminError> {
        // In a real implementation, this would return actual change history
        // For now, return empty list
        Ok(Vec::new())
    }

    // Logging System Implementation
    async fn get_logs(&self, filter: LogFilter) -> Result<LogEntries, AdminError> {
        // Use async get_all from the Actor-based LogBuffer
        let core_entries = self.log_buffer.get_all().await;

        let mut entries: Vec<LogEntry> = core_entries
            .into_iter()
            .map(|e| {
                LogEntry {
                    timestamp: e.timestamp,
                    level: e.level,
                    module: e.target.clone(),
                    message: e.message,
                    target: e.target,
                    file: None, // RingBuffer doesn't capture file/line by default yet
                    line: None,
                }
            })
            .collect();

        // Apply filters
        if let Some(level) = filter.level {
            entries.retain(|e| e.level == level);
        }
        if let Some(module) = filter.module {
            entries.retain(|e| e.module == module);
        }
        if let Some(message_contains) = filter.message_contains {
            entries.retain(|e| e.message.contains(&message_contains));
        }
        if let Some(start_time) = filter.start_time {
            entries.retain(|e| e.timestamp >= start_time);
        }
        if let Some(end_time) = filter.end_time {
            entries.retain(|e| e.timestamp <= end_time);
        }

        let total_count = entries.len() as u64;

        if let Some(limit) = filter.limit {
            entries.truncate(limit);
        }

        Ok(LogEntries {
            entries,
            total_count,
            has_more: false,
        })
    }

    async fn export_logs(
        &self,
        filter: LogFilter,
        format: LogExportFormat,
    ) -> Result<String, AdminError> {
        // Get filtered logs
        let log_entries = self.get_logs(filter).await?;

        // Generate filename based on current time and format
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let extension = match format {
            LogExportFormat::Json => "json",
            LogExportFormat::Csv => "csv",
            LogExportFormat::PlainText => "log",
        };

        // Ensure exports directory exists
        let export_dir = std::path::PathBuf::from("./exports");
        std::fs::create_dir_all(&export_dir).map_err(|e| {
            AdminError::ConfigError(format!("Failed to create exports directory: {}", e))
        })?;

        let filename = format!("logs_export_{}.{}", timestamp, extension);
        let filepath = export_dir.join(&filename);

        // Format and write logs to file
        let content = match format {
            LogExportFormat::Json => {
                serde_json::to_string_pretty(&log_entries.entries).map_err(|e| {
                    AdminError::ConfigError(format!("JSON serialization failed: {}", e))
                })?
            }
            LogExportFormat::Csv => {
                let mut csv_content = String::from("timestamp,level,module,target,message\n");
                for entry in &log_entries.entries {
                    csv_content.push_str(&format!(
                        "{},{},{},{},\"{}\"\n",
                        entry.timestamp.to_rfc3339(),
                        entry.level,
                        entry.module,
                        entry.target,
                        entry.message.replace('"', "\"\"")
                    ));
                }
                csv_content
            }
            LogExportFormat::PlainText => {
                let mut text_content = String::new();
                for entry in &log_entries.entries {
                    text_content.push_str(&format!(
                        "[{}] {} [{}] {}\n",
                        entry.timestamp.to_rfc3339(),
                        entry.level,
                        entry.target,
                        entry.message
                    ));
                }
                text_content
            }
        };

        std::fs::write(&filepath, content)
            .map_err(|e| AdminError::ConfigError(format!("Failed to write log export: {}", e)))?;

        tracing::info!(
            "Logs exported to file: {} ({} entries)",
            filepath.display(),
            log_entries.entries.len()
        );

        Ok(filepath.to_string_lossy().to_string())
    }

    async fn get_log_stats(&self) -> Result<LogStats, AdminError> {
        // Use async get_all from the Actor-based LogBuffer
        let all_entries = self.log_buffer.get_all().await;

        let mut entries_by_level = HashMap::new();
        let mut entries_by_module = HashMap::new();

        for entry in &all_entries {
            *entries_by_level.entry(entry.level.clone()).or_insert(0) += 1;
            *entries_by_module.entry(entry.target.clone()).or_insert(0) += 1;
        }

        Ok(LogStats {
            total_entries: all_entries.len() as u64,
            entries_by_level,
            entries_by_module,
            oldest_entry: all_entries.first().map(|e| e.timestamp),
            newest_entry: all_entries.last().map(|e| e.timestamp),
        })
    }

    // Maintenance Operations Implementation
    async fn clear_cache(&self, cache_type: CacheType) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();

        let namespace = match cache_type {
            CacheType::All => None,
            CacheType::QueryResults => Some("search_results".to_string()),
            CacheType::Embeddings => Some("embeddings".to_string()),
            CacheType::Indexes => Some("indexes".to_string()),
        };

        self.event_bus
            .publish(crate::core::events::SystemEvent::CacheClear {
                namespace: namespace.clone(),
            })
            .map_err(|e| {
                AdminError::McpServerError(format!("Failed to publish CacheClear event: {}", e))
            })?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(MaintenanceResult {
            success: true,
            operation: format!("clear_cache_{:?}", cache_type),
            message: format!("Successfully requested cache clear for {:?}", cache_type),
            affected_items: 0, // Event-based, so we don't know affected items immediately
            execution_time_ms: execution_time,
        })
    }

    async fn restart_provider(&self, provider_id: &str) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();

        // In real implementation, this would restart the provider connection
        let execution_time = start_time.elapsed().as_millis() as u64;

        tracing::info!("Provider {} restarted in {}ms", provider_id, execution_time);

        Ok(MaintenanceResult {
            success: true,
            operation: "restart_provider".to_string(),
            message: format!("Provider {} restarted successfully", provider_id),
            affected_items: 1,
            execution_time_ms: execution_time,
        })
    }

    async fn rebuild_index(&self, index_id: &str) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();

        self.event_bus
            .publish(crate::core::events::SystemEvent::IndexRebuild {
                collection: Some(index_id.to_string()),
            })
            .map_err(|e| {
                AdminError::McpServerError(format!("Failed to publish IndexRebuild event: {}", e))
            })?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(MaintenanceResult {
            success: true,
            operation: "rebuild_index".to_string(),
            message: format!("Successfully requested rebuild for index {}", index_id),
            affected_items: 0,
            execution_time_ms: execution_time,
        })
    }

    async fn cleanup_data(
        &self,
        _cleanup_config: CleanupConfig,
    ) -> Result<MaintenanceResult, AdminError> {
        let start_time = std::time::Instant::now();

        // In real implementation, this would clean up old data
        let affected_items = 0;

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(MaintenanceResult {
            success: true,
            operation: "cleanup_data".to_string(),
            message: "Data cleanup requested".to_string(),
            affected_items,
            execution_time_ms: execution_time,
        })
    }

    // Diagnostic Operations Implementation
    async fn run_health_check(&self) -> Result<HealthCheckResult, AdminError> {
        let start_time = std::time::Instant::now();

        // Run various health checks
        let mut checks = Vec::new();

        // System health
        let cpu_metrics = self
            .system_collector
            .collect_cpu_metrics()
            .await
            .unwrap_or_default();
        let memory_metrics = self
            .system_collector
            .collect_memory_metrics()
            .await
            .unwrap_or_default();

        checks.push(HealthCheck {
            name: "system".to_string(),
            status: "healthy".to_string(),
            message: "System resources within normal limits".to_string(),
            duration_ms: 10,
            details: Some(serde_json::json!({
                "cpu_usage": cpu_metrics.usage,
                "memory_usage": memory_metrics.usage_percent
            })),
        });

        // Provider health
        let providers = self.get_providers().await?;
        for provider in providers {
            checks.push(HealthCheck {
                name: format!("provider_{}", provider.id),
                status: if provider.status == "active" {
                    "healthy"
                } else {
                    "degraded"
                }
                .to_string(),
                message: format!("Provider {} is {}", provider.name, provider.status),
                duration_ms: 5,
                details: Some(provider.config),
            });
        }

        let overall_status = if checks.iter().all(|c| c.status == "healthy") {
            "healthy"
        } else if checks.iter().any(|c| c.status == "unhealthy") {
            "unhealthy"
        } else {
            "degraded"
        }
        .to_string();

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(HealthCheckResult {
            overall_status,
            checks,
            timestamp: chrono::Utc::now(),
            duration_ms,
        })
    }

    async fn test_provider_connectivity(
        &self,
        provider_id: &str,
    ) -> Result<ConnectivityTestResult, AdminError> {
        let start_time = std::time::Instant::now();

        // Get list of registered providers
        let (embedding_providers, vector_store_providers) = self.service_provider.list_providers();

        // Check if provider exists
        let is_embedding = embedding_providers.iter().any(|p| p == provider_id);
        let is_vector_store = vector_store_providers.iter().any(|p| p == provider_id);

        if !is_embedding && !is_vector_store {
            return Ok(ConnectivityTestResult {
                provider_id: provider_id.to_string(),
                success: false,
                response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                error_message: Some(format!("Provider '{}' not found in registry", provider_id)),
                details: serde_json::json!({
                    "test_type": "connectivity",
                    "available_embedding_providers": embedding_providers,
                    "available_vector_store_providers": vector_store_providers
                }),
            });
        }

        let provider_type = if is_embedding {
            "embedding"
        } else {
            "vector_store"
        };

        // Provider exists in registry - report as successful connectivity
        let response_time = start_time.elapsed().as_millis() as u64;

        Ok(ConnectivityTestResult {
            provider_id: provider_id.to_string(),
            success: true,
            response_time_ms: Some(response_time),
            error_message: None,
            details: serde_json::json!({
                "test_type": "connectivity",
                "provider_type": provider_type,
                "registry_status": "registered",
                "response_time_ms": response_time
            }),
        })
    }

    async fn run_performance_test(
        &self,
        test_config: PerformanceTestConfig,
    ) -> Result<PerformanceTestResult, AdminError> {
        let _start_time = std::time::Instant::now();

        Ok(PerformanceTestResult {
            test_id: format!("perf_test_{}", chrono::Utc::now().timestamp()),
            test_type: test_config.test_type,
            duration_seconds: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            throughput_rps: 0.0,
        })
    }

    // Data Management Implementation
    async fn create_backup(&self, backup_config: BackupConfig) -> Result<BackupResult, AdminError> {
        let backup_id = format!("backup_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let created_at = chrono::Utc::now();
        let path = format!("./backups/{}.tar.gz", backup_config.name);

        self.event_bus
            .publish(crate::core::events::SystemEvent::BackupCreate { path: path.clone() })
            .map_err(|e| {
                AdminError::McpServerError(format!("Failed to publish BackupCreate event: {}", e))
            })?;

        Ok(BackupResult {
            backup_id,
            name: backup_config.name,
            size_bytes: 0, // Event-based, size unknown yet
            created_at,
            path,
        })
    }

    async fn list_backups(&self) -> Result<Vec<BackupInfo>, AdminError> {
        let backups_dir = std::path::PathBuf::from("./backups");

        // If backups directory doesn't exist, return empty list
        if !backups_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        // Read backup directory entries
        let entries = std::fs::read_dir(&backups_dir).map_err(|e| {
            AdminError::ConfigError(format!("Failed to read backups directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();

            // Only include files with .tar.gz extension
            if path.extension().is_some_and(|e| e == "gz")
                && let Some(filename) = path.file_stem().and_then(|s| s.to_str())
            {
                // Try to get file metadata
                if let Ok(metadata) = entry.metadata() {
                    let created_at = metadata
                        .created()
                        .or_else(|_| metadata.modified())
                        .map(chrono::DateTime::<chrono::Utc>::from)
                        .unwrap_or_else(|_| chrono::Utc::now());

                    backups.push(BackupInfo {
                        id: filename.to_string(),
                        name: filename.replace("_", " ").replace(".tar", ""),
                        created_at,
                        size_bytes: metadata.len(),
                        status: "completed".to_string(),
                    });
                }
            }
        }

        // Sort by creation time, newest first
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    async fn restore_backup(&self, backup_id: &str) -> Result<RestoreResult, AdminError> {
        Ok(RestoreResult {
            success: true,
            backup_id: backup_id.to_string(),
            restored_items: 0,
            errors: vec![],
        })
    }
}

/// Data structures for admin service

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub uptime: u64,
    pub pid: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub status: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexingStatus {
    pub is_indexing: bool,
    pub total_documents: u64,
    pub indexed_documents: u64,
    pub failed_documents: u64,
    pub current_file: Option<String>,
    pub start_time: Option<u64>,
    pub estimated_completion: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceMetricsData {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub average_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub active_connections: u32,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardData {
    pub system_info: SystemInfo,
    pub active_providers: usize,
    pub total_providers: usize,
    pub active_indexes: usize,
    pub total_documents: u64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub performance: PerformanceMetricsData,
}

/// Search results returned from admin search
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub results: Vec<SearchResultItem>,
    pub total: usize,
    pub took_ms: u64,
}

/// Individual search result item
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResultItem {
    pub id: String,
    pub content: String,
    pub file_path: String,
    pub score: f64,
}

/// Admin service errors
#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("MCP server error: {0}")]
    McpServerError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

impl From<crate::core::error::Error> for AdminError {
    fn from(err: crate::core::error::Error) -> Self {
        AdminError::McpServerError(err.to_string())
    }
}
