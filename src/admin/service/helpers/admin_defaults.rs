//! Admin Operations Default Configuration
//!
//! Centralized default values for admin service operations.
//! All values are configurable via environment variables.

// Activity Feed Configuration
pub const DEFAULT_MAX_ACTIVITIES: usize = 100;
pub const DEFAULT_ACTIVITY_RETENTION_DAYS: u32 = 30;
pub const DEFAULT_ACTIVITY_BUFFER_SIZE: usize = 1000;

// Configuration History
pub const DEFAULT_MAX_HISTORY_ENTRIES: usize = 1000;
pub const DEFAULT_HISTORY_RETENTION_DAYS: u32 = 90;
pub const DEFAULT_CONFIG_QUERY_LIMIT: usize = 100;

// Logging Configuration
pub const DEFAULT_LOG_BUFFER_SIZE: usize = 1000;
pub const DEFAULT_LOG_RETENTION_DAYS: u32 = 7;
pub const DEFAULT_LOG_QUERY_LIMIT: usize = 100;

// Backup Configuration
pub const DEFAULT_BACKUP_RETENTION_DAYS: u32 = 30;
pub const DEFAULT_BACKUP_COMPRESSION_LEVEL: u32 = 6;
pub const DEFAULT_MAX_BACKUPS: usize = 10;

// Route Discovery Configuration
pub const DEFAULT_ROUTE_RATE_LIMIT_HEALTH: u32 = 100;      // requests per minute
pub const DEFAULT_ROUTE_RATE_LIMIT_ADMIN: u32 = 100;       // requests per minute
pub const DEFAULT_ROUTE_RATE_LIMIT_INDEXING: u32 = 10;     // requests per minute
pub const DEFAULT_ROUTE_RATE_LIMIT_SEARCH: u32 = 10;       // requests per minute
pub const DEFAULT_ROUTE_RATE_LIMIT_SHUTDOWN: u32 = 60;     // seconds cooldown
pub const DEFAULT_ROUTE_RATE_LIMIT_RELOAD: u32 = 30;       // seconds cooldown
pub const DEFAULT_ROUTE_RATE_LIMIT_BACKUP: u32 = 60;       // seconds cooldown
pub const DEFAULT_ROUTE_RATE_LIMIT_RESTORE: u32 = 10;      // requests per minute

// Maintenance Operations
pub const DEFAULT_CLEANUP_BATCH_SIZE: usize = 100;
pub const DEFAULT_CLEANUP_RETENTION_DAYS: u32 = 30;
pub const DEFAULT_INDEX_REBUILD_TIMEOUT_SECS: u64 = 3600;  // 1 hour
pub const DEFAULT_CACHE_CLEAR_TIMEOUT_SECS: u64 = 300;     // 5 minutes

// Performance Testing
pub const DEFAULT_PERF_TEST_DURATION_SECS: u32 = 30;
pub const DEFAULT_PERF_TEST_CONCURRENCY: u32 = 4;
pub const DEFAULT_PERF_TEST_TIMEOUT_MS: u64 = 5000;

// Directory Configuration
pub const DEFAULT_BACKUPS_DIR: &str = "./backups";
pub const DEFAULT_DATA_DIR: &str = "./data";
pub const DEFAULT_EXPORTS_DIR: &str = "./exports";

// Time Constants
pub const SECONDS_PER_DAY: u64 = 86400;

// Helper function to read environment variables with defaults
pub fn get_env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn get_env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn get_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
