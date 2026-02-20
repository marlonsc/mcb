//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
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

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn epoch_secs_i64_returns_positive() -> TestResult {
        let ts = epoch_secs_i64()?;
        assert!(ts > 0, "timestamp should be positive, got {ts}");
        Ok(())
    }

    #[test]
    fn epoch_secs_u64_returns_positive() -> TestResult {
        let ts = epoch_secs_u64()?;
        assert!(ts > 0, "timestamp should be positive, got {ts}");
        Ok(())
    }

    #[test]
    fn epoch_nanos_u128_returns_positive() -> TestResult {
        let nanos = epoch_nanos_u128()?;
        assert!(nanos > 0, "nanos should be positive, got {nanos}");
        Ok(())
    }

    #[test]
    fn epoch_secs_i64_monotonic() -> TestResult {
        let a = epoch_secs_i64()?;
        let b = epoch_secs_i64()?;
        assert!(b >= a, "expected monotonic: {b} >= {a}");
        Ok(())
    }

    #[test]
    fn epoch_secs_u64_monotonic() -> TestResult {
        let a = epoch_secs_u64()?;
        let b = epoch_secs_u64()?;
        assert!(b >= a, "expected monotonic: {b} >= {a}");
        Ok(())
    }

    #[test]
    fn epoch_nanos_u128_monotonic() -> TestResult {
        let a = epoch_nanos_u128()?;
        let b = epoch_nanos_u128()?;
        assert!(b >= a, "expected monotonic: {b} >= {a}");
        Ok(())
    }

    #[test]
    fn epoch_secs_i64_reasonable_range() -> TestResult {
        use crate::constants::time::{TIMESTAMP_MAX_BOUNDARY, TIMESTAMP_MIN_BOUNDARY};
        let ts = epoch_secs_i64()?;
        // Should be after 2020-01-01 and before 2100-01-01
        assert!(ts > TIMESTAMP_MIN_BOUNDARY, "timestamp too old: {ts}");
        assert!(
            ts < TIMESTAMP_MAX_BOUNDARY,
            "timestamp too far in future: {ts}"
        );
        Ok(())
    }
}
