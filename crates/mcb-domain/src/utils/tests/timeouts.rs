//! Test timeout and polling helpers.
//!
//! Centralized in `mcb-domain` for consistent timeout values across all test crates.

use std::time::{Duration, Instant};

/// Default timeout for integration/e2e tests.
pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Poll a sync check function until it returns `Some(T)` or timeout expires.
pub fn eventually<T, F>(timeout: Duration, interval: Duration, check: F) -> Option<T>
where
    F: Fn() -> Option<T>,
{
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(result) = check() {
            return Some(result);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(interval);
    }
}
