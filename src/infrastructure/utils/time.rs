//! Time utilities - Centralized time functions (DRY)
//!
//! Eliminates `SystemTime::now().duration_since(UNIX_EPOCH)` pattern

/// Time utilities - eliminates `SystemTime::now().duration_since(UNIX_EPOCH)` pattern
pub struct TimeUtils;

impl TimeUtils {
    /// Get current Unix timestamp in seconds
    #[inline]
    pub fn now_unix_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Get current Unix timestamp in milliseconds
    #[inline]
    pub fn now_unix_millis() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    }

    /// Check if a timestamp has expired given a TTL in seconds
    #[inline]
    pub fn is_expired(timestamp: u64, ttl_secs: u64) -> bool {
        Self::now_unix_secs().saturating_sub(timestamp) > ttl_secs
    }

    /// Calculate age in seconds from a timestamp
    #[inline]
    pub fn age_secs(timestamp: u64) -> u64 {
        Self::now_unix_secs().saturating_sub(timestamp)
    }
}
