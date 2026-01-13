//! Status utilities - Consistent status checking and constants (DRY)
//!
//! Centralizes status strings and health checking logic

/// Common status values used across admin interface
pub const HEALTHY: &str = "healthy";
pub const DEGRADED: &str = "degraded";
pub const CRITICAL: &str = "critical";
pub const ACTIVE: &str = "active";
pub const INACTIVE: &str = "inactive";
pub const INDEXING: &str = "indexing";
pub const IDLE: &str = "idle";
pub const BUSY: &str = "busy";
pub const UNKNOWN: &str = "unknown";

/// Activity level strings
pub mod activity_level {
    pub const SUCCESS: &str = "success";
    pub const WARNING: &str = "warning";
    pub const ERROR: &str = "error";
    pub const INFO: &str = "info";
}

/// Status checking utilities for consistent status evaluation
pub struct StatusUtils;

impl StatusUtils {
    /// Check if status indicates healthy/active/available state
    ///
    /// Normalizes inconsistent status values used across the codebase.
    /// Returns true for: "available", "active", "healthy", "success", "ready", "online"
    pub fn is_healthy(status: &str) -> bool {
        matches!(
            status.to_lowercase().as_str(),
            "available" | "active" | "healthy" | "success" | "ready" | "online"
        )
    }

    /// Check if status indicates degraded/warning state
    ///
    /// Returns true for: "degraded", "warning", "slow", "partial"
    pub fn is_degraded(status: &str) -> bool {
        matches!(
            status.to_lowercase().as_str(),
            "degraded" | "warning" | "slow" | "partial"
        )
    }

    /// Check if status indicates error/failure state
    ///
    /// Returns true for: "error", "failed", "unavailable", "down", "offline"
    pub fn is_error(status: &str) -> bool {
        matches!(
            status.to_lowercase().as_str(),
            "error" | "failed" | "unavailable" | "down" | "offline"
        )
    }

    /// Check if status indicates processing state
    ///
    /// Returns true for: "indexing", "processing", "building", "syncing", "updating"
    pub fn is_processing(status: &str) -> bool {
        matches!(
            status.to_lowercase().as_str(),
            "indexing" | "processing" | "building" | "syncing" | "updating"
        )
    }
}

/// Health check utilities for determining system status
pub struct HealthUtils;

impl HealthUtils {
    /// Determine health status based on CPU and memory usage
    pub fn compute_status(cpu_usage: f64, memory_usage: f64) -> &'static str {
        const HEALTHY_THRESHOLD: f64 = 85.0;
        match (
            cpu_usage < HEALTHY_THRESHOLD,
            memory_usage < HEALTHY_THRESHOLD,
        ) {
            (true, true) => HEALTHY,
            (false, false) => CRITICAL,
            _ => DEGRADED,
        }
    }
}
