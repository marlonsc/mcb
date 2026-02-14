//! Canonical time utilities — strict, no fallbacks.
//!
//! See [`UTILITIES_POLICY.md`](./UTILITIES_POLICY.md) for rules.
//!
//! All functions return `Result` instead of silently defaulting to 0.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Error;

/// Returns the current Unix timestamp in seconds as `i64`.
///
/// # Errors
///
/// Returns an error if the system clock is before the Unix epoch (extremely rare,
/// but we refuse to silently return 0).
pub fn epoch_secs_i64() -> Result<i64, Error> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::internal(format!("system clock is before Unix epoch: {e}")))?;
    i64::try_from(duration.as_secs())
        .map_err(|e| Error::internal(format!("timestamp overflow for i64: {e}")))
}

/// Returns the current Unix timestamp in seconds as `u64`.
///
/// # Errors
///
/// Returns an error if the system clock is before the Unix epoch.
pub fn epoch_secs_u64() -> Result<u64, Error> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::internal(format!("system clock is before Unix epoch: {e}")))?;
    Ok(duration.as_secs())
}

/// Returns the current Unix timestamp in nanoseconds as `u128`.
///
/// Useful for high-resolution trace IDs and deduplication seeds.
///
/// # Errors
///
/// Returns an error if the system clock is before the Unix epoch.
pub fn epoch_nanos_u128() -> Result<u128, Error> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::internal(format!("system clock is before Unix epoch: {e}")))?;
    Ok(duration.as_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_secs_i64_returns_positive() {
        let ts = epoch_secs_i64().unwrap();
        assert!(ts > 0, "timestamp should be positive, got {ts}");
    }

    #[test]
    fn epoch_secs_u64_returns_positive() {
        let ts = epoch_secs_u64().unwrap();
        assert!(ts > 0, "timestamp should be positive, got {ts}");
    }

    #[test]
    fn epoch_nanos_u128_returns_positive() {
        let nanos = epoch_nanos_u128().unwrap();
        assert!(nanos > 0, "nanos should be positive, got {nanos}");
    }

    #[test]
    fn epoch_secs_i64_monotonic() {
        let a = epoch_secs_i64().unwrap();
        let b = epoch_secs_i64().unwrap();
        assert!(b >= a, "expected monotonic: {b} >= {a}");
    }

    #[test]
    fn epoch_secs_u64_monotonic() {
        let a = epoch_secs_u64().unwrap();
        let b = epoch_secs_u64().unwrap();
        assert!(b >= a, "expected monotonic: {b} >= {a}");
    }

    #[test]
    fn epoch_nanos_u128_monotonic() {
        let a = epoch_nanos_u128().unwrap();
        let b = epoch_nanos_u128().unwrap();
        assert!(b >= a, "expected monotonic: {b} >= {a}");
    }

    #[test]
    fn epoch_secs_i64_reasonable_range() {
        let ts = epoch_secs_i64().unwrap();
        // Should be after 2020-01-01 (1577836800) and before 2100-01-01 (4102444800)
        assert!(ts > 1_577_836_800, "timestamp too old: {ts}");
        assert!(ts < 4_102_444_800, "timestamp too far in future: {ts}");
    }
}
