//! Canonical time utilities — strict, no fallbacks.
//!
//! All functions return `Result` instead of silently defaulting to 0.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::UtilsError;

/// Returns the current Unix timestamp in seconds as `i64`.
///
/// # Errors
///
/// Returns an error if the system clock is before the Unix epoch (extremely rare,
/// but we refuse to silently return 0).
pub fn epoch_secs_i64() -> Result<i64, UtilsError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| UtilsError::Time(format!("system clock is before Unix epoch: {e}")))?;
    i64::try_from(duration.as_secs())
        .map_err(|e| UtilsError::Time(format!("timestamp overflow for i64: {e}")))
}

/// Returns the current Unix timestamp in seconds as `u64`.
///
/// # Errors
///
/// Returns an error if the system clock is before the Unix epoch.
pub fn epoch_secs_u64() -> Result<u64, UtilsError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| UtilsError::Time(format!("system clock is before Unix epoch: {e}")))?;
    Ok(duration.as_secs())
}

/// Returns the current Unix timestamp in nanoseconds as `u128`.
///
/// Useful for high-resolution trace IDs and deduplication seeds.
///
/// # Errors
///
/// Returns an error if the system clock is before the Unix epoch.
pub fn epoch_nanos_u128() -> Result<u128, UtilsError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| UtilsError::Time(format!("system clock is before Unix epoch: {e}")))?;
    Ok(duration.as_nanos())
}
