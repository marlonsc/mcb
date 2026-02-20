use std::time::Duration;

pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn eventually<T, F, Fut>(timeout: Duration, interval: Duration, check: F) -> Option<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Option<T>>,
{
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if let Some(result) = check().await {
            return Some(result);
        }
        if tokio::time::Instant::now() >= deadline {
            return None;
        }
        tokio::time::sleep(interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::{TEST_TIMEOUT, eventually};
    use std::time::Duration;

    #[tokio::test]
    async fn eventually_returns_value() {
        assert!(TEST_TIMEOUT >= Duration::from_secs(30));
        let result = eventually(
            Duration::from_millis(50),
            Duration::from_millis(1),
            || async { Some(1usize) },
        )
        .await;
        assert_eq!(result, Some(1));
    }
}
