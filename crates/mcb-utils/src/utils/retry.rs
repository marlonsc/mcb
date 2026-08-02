//!
//! **Documentation**: [docs/modules/utils.md](../../../../docs/modules/utils.md)
//!
//! Retry utilities with exponential backoff.

use std::future::Future;
use std::time::Duration;

/// Retry configuration for exponential backoff execution.
pub struct RetryConfig {
    /// Maximum number of attempts before returning the last error.
    pub max_attempts: usize,
    /// Base delay used by backoff between retries.
    pub base_delay: Duration,
}

impl RetryConfig {
    /// Create a new retry configuration.
    #[must_use]
    pub const fn new(max_attempts: usize, base_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
        }
    }
}

/// Retry an async operation with linear backoff based on attempt count.
///
/// # Errors
///
/// Returns the last error if all retry attempts fail or the retry predicate rejects the error.
pub async fn retry_with_backoff<T, E, F, Fut, P>(
    config: RetryConfig,
    mut operation: F,
    should_retry: P,
) -> std::result::Result<T, E>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = std::result::Result<T, E>>,
    P: Fn(&E) -> bool,
{
    let attempts = config.max_attempts.max(1);

    // `loop {}` (not `for`) so the compiler proves every exit is a `return`,
    // making a trailing unreachable!/panic path unnecessary.
    let mut attempt = 0;
    loop {
        match operation(attempt).await {
            Ok(value) => return Ok(value),
            Err(error) => {
                attempt += 1;
                if attempt == attempts || !should_retry(&error) {
                    return Err(error);
                }

                tokio::time::sleep(config.base_delay.mul_f64(attempt as f64)).await;
            }
        }
    }
}
