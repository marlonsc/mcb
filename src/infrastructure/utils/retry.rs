//! Retry utilities - Async retry with exponential backoff (~15 lines per use) (DRY)
//!
//! Eliminates boilerplate retry loops across the codebase

use std::future::Future;
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries (doubles each attempt)
    pub initial_delay_ms: u64,
    /// Maximum delay cap (prevents exponential explosion)
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 500,
            max_delay_ms: 5000,
        }
    }
}

impl RetryConfig {
    /// Create with custom attempts
    pub fn with_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Create with custom initial delay
    pub fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    /// Create with custom max delay
    pub fn with_max_delay(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }
}

/// Retry utilities - saves ~15 lines per retry pattern
pub struct RetryUtils;

impl RetryUtils {
    /// Retry an async operation with exponential backoff
    ///
    /// # Arguments
    /// * `config` - Retry configuration (attempts, delays)
    /// * `operation` - Async closure returning Result<T, E>
    /// * `should_retry` - Predicate on error to decide if retry should continue
    /// * `context` - Description for logging (e.g., "index creation")
    ///
    /// # Example
    /// ```ignore
    /// // Before: ~15 lines of retry logic
    /// // After: 6 lines
    /// RetryUtils::retry_with_backoff(
    ///     RetryConfig::default(),
    ///     || async { client.create_index(name).await },
    ///     |e| e.to_string().contains("NotFound"),
    ///     "index creation",
    /// ).await?;
    /// ```
    pub async fn retry_with_backoff<T, E, F, Fut, R>(
        config: RetryConfig,
        mut operation: F,
        should_retry: R,
        context: &str,
    ) -> std::result::Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = std::result::Result<T, E>>,
        R: Fn(&E) -> bool,
        E: std::fmt::Display,
    {
        let mut last_error = None;

        for attempt in 0..config.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if should_retry(&e) && attempt + 1 < config.max_attempts {
                        let delay = std::cmp::min(
                            config.initial_delay_ms * 2u64.pow(attempt),
                            config.max_delay_ms,
                        );
                        tracing::debug!(
                            "{} attempt {} failed ({}), retrying in {}ms...",
                            context,
                            attempt + 1,
                            e,
                            delay
                        );
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        last_error = Some(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // Should not reach here, but return last error if we do
        Err(last_error.expect("retry loop should have at least one attempt"))
    }

    /// Simplified retry - always retries on any error
    ///
    /// # Example
    /// ```ignore
    /// RetryUtils::retry(3, 500, || async { fetch_data().await }, "data fetch").await?;
    /// ```
    pub async fn retry<T, E, F, Fut>(
        max_attempts: u32,
        initial_delay_ms: u64,
        operation: F,
        context: &str,
    ) -> std::result::Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display,
    {
        Self::retry_with_backoff(
            RetryConfig::default()
                .with_attempts(max_attempts)
                .with_initial_delay(initial_delay_ms),
            operation,
            |_| true, // Always retry on error
            context,
        )
        .await
    }

    /// Calculate delay for a given attempt (useful for custom retry loops)
    #[inline]
    pub fn calculate_delay(attempt: u32, initial_ms: u64, max_ms: u64) -> Duration {
        let delay = std::cmp::min(initial_ms * 2u64.pow(attempt), max_ms);
        Duration::from_millis(delay)
    }
}
