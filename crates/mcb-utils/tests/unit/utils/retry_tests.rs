//! Unit tests for retry utilities.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use mcb_utils::utils::retry::{RetryConfig, retry_with_backoff};

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
