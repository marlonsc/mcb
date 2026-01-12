//! Data types for admin service operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::admin::models::{ProviderInfo, IndexingConfig, SecurityConfig};

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

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub uptime: u64,
    pub pid: u32,
}

/// Indexing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStatus {
    pub is_indexing: bool,
    pub total_documents: u64,
    pub indexed_documents: u64,
    pub failed_documents: u64,
    pub current_file: Option<String>,
    pub start_time: Option<u64>,
    pub estimated_completion: Option<u64>,
}

/// Performance metrics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsData {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub average_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub active_connections: u32,
    pub uptime_seconds: u64,
}

/// Dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub results: Vec<SearchResultItem>,
    pub total: usize,
    pub took_ms: u64,
}

/// Individual search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<crate::domain::error::Error> for AdminError {
    fn from(err: crate::domain::error::Error) -> Self {
        AdminError::McpServerError(err.to_string())
    }
}
