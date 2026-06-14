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

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::{RetryConfig, retry_with_backoff};

    fn config(max_attempts: usize) -> RetryConfig {
        RetryConfig::new(max_attempts, Duration::from_millis(0))
    }

    #[tokio::test]
    async fn returns_first_success_without_retrying() {
        let calls = AtomicUsize::new(0);
        let result: Result<u8, ()> = retry_with_backoff(
            config(3),
            |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                async { Ok(7) }
            },
            |_| true,
        )
        .await;

        assert_eq!(result, Ok(7));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn returns_last_error_after_exhausting_attempts() {
        let calls = AtomicUsize::new(0);
        let result: Result<(), usize> = retry_with_backoff(
            config(3),
            |attempt| {
                calls.fetch_add(1, Ordering::SeqCst);
                async move { Err(attempt) }
            },
            |_| true,
        )
        .await;

        assert_eq!(result, Err(2));
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn stops_immediately_when_predicate_rejects() {
        let calls = AtomicUsize::new(0);
        let result: Result<(), &str> = retry_with_backoff(
            config(5),
            |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                async { Err("fatal") }
            },
            |_| false,
        )
        .await;

        assert_eq!(result, Err("fatal"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
