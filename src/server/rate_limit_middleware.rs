//! HTTP Rate Limiting Middleware
//!
//! Axum middleware for rate limiting HTTP requests using the core RateLimiter.
//! Integrates with the metrics server and provides proper HTTP headers.

use axum::extract::ConnectInfo;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::core::rate_limit::{RateLimitKey, RateLimiter};

/// Simple rate limiting check function
/// This can be used in route handlers directly
pub async fn check_rate_limit_for_ip(
    rate_limiter: &Option<Arc<RateLimiter>>,
    addr: &ConnectInfo<SocketAddr>,
) -> Result<(), (axum::http::StatusCode, String)> {
    let Some(limiter) = rate_limiter else {
        // No rate limiter configured, allow request
        return Ok(());
    };

    let client_ip = addr.0.ip().to_string();
    let key = RateLimitKey::Ip(client_ip);

    match limiter.check_rate_limit(&key).await {
        Ok(result) if result.allowed => Ok(()),
        Ok(result) => {
            let message = format!(
                "Rate limit exceeded. {} requests remaining. Resets in {} seconds.",
                result.remaining, result.reset_in_seconds
            );
            Err((axum::http::StatusCode::TOO_MANY_REQUESTS, message))
        }
        Err(e) => {
            // Log error but allow request to avoid blocking legitimate users
            tracing::error!("Rate limiting check failed: {}", e);
            Ok(())
        }
    }
}

/// Helper function to add rate limit headers to a response
pub fn add_rate_limit_headers(
    headers: &mut axum::http::HeaderMap,
    limiter: &Arc<RateLimiter>,
    result: &crate::core::rate_limit::RateLimitResult,
) {
    headers.insert(
        "X-RateLimit-Limit",
        (limiter.config().max_requests_per_window + limiter.config().burst_allowance)
            .to_string()
            .parse()
            .unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        result.remaining.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Reset",
        result.reset_in_seconds.to_string().parse().unwrap(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_middleware_functions_exist() {
        // Test that functions exist and can be called
        let rate_limiter: Option<Arc<RateLimiter>> = None;
        let addr = ConnectInfo(std::net::SocketAddr::from(([127, 0, 0, 1], 8080)));

        // Should succeed with no rate limiter
        let result = check_rate_limit_for_ip(&rate_limiter, &addr).await;
        assert!(result.is_ok());
    }
}
