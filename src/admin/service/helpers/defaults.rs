//! Default configuration constants
//!
//! Centralized default values for runtime configuration thresholds and system metrics.
//! These values are used when environment variables are not configured.

// Health Check Thresholds - CPU
pub const DEFAULT_HEALTH_CPU_UNHEALTHY_PERCENT: f64 = 90.0;
pub const DEFAULT_HEALTH_CPU_DEGRADED_PERCENT: f64 = 75.0;

// Health Check Thresholds - Memory
pub const DEFAULT_HEALTH_MEMORY_UNHEALTHY_PERCENT: f64 = 90.0;
pub const DEFAULT_HEALTH_MEMORY_DEGRADED_PERCENT: f64 = 80.0;

// Health Check Thresholds - Disk
pub const DEFAULT_HEALTH_DISK_UNHEALTHY_PERCENT: f64 = 90.0;
pub const DEFAULT_HEALTH_DISK_DEGRADED_PERCENT: f64 = 80.0;

// Health Check Thresholds - Database Pool
pub const DEFAULT_HEALTH_DB_POOL_UNHEALTHY_PERCENT: f64 = 95.0;
pub const DEFAULT_HEALTH_DB_POOL_DEGRADED_PERCENT: f64 = 80.0;

// Health Check Thresholds - Cache Hit Rate
pub const DEFAULT_HEALTH_CACHE_HIT_RATE_DEGRADED: f64 = 0.5; // 50%

// Performance Test Multipliers
pub const DEFAULT_PERF_P95_MULTIPLIER: f64 = 1.2;
pub const DEFAULT_PERF_P99_MULTIPLIER: f64 = 1.5;

// Indexing Configuration
pub const DEFAULT_INDEXING_ENABLED: bool = true;
pub const DEFAULT_INDEXING_PENDING_OPERATIONS: u64 = 0;

// Cache Configuration
pub const DEFAULT_CACHE_ENABLED: bool = true;
pub const DEFAULT_CACHE_ENTRIES_COUNT: u64 = 0;
pub const DEFAULT_CACHE_HIT_RATE: f64 = 0.0;
pub const DEFAULT_CACHE_SIZE_BYTES: u64 = 0;
pub const DEFAULT_CACHE_MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024 * 1024; // 10GB

// Database Configuration
pub const DEFAULT_DB_CONNECTED: bool = true;
pub const DEFAULT_DB_ACTIVE_CONNECTIONS: u32 = 0;
pub const DEFAULT_DB_IDLE_CONNECTIONS: u32 = 0;
pub const DEFAULT_DB_POOL_SIZE: u32 = 20;
