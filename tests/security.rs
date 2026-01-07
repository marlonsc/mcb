//! Security tests for production hardening validation
//!
//! Tests security headers, request validation, and protection
//! against common web vulnerabilities.

use crate::metrics::MetricsApiServer;
use crate::server::SecurityConfig;
use axum::{
    body::Body,
    http::{Method, Request, StatusCode, Uri},
    routing::get,
    Router,
};
use tower::ServiceExt;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_headers_added() {
        let config = SecurityConfig::default();
        let app = Router::new().route("/test", get(|| async { "OK" })).layer(
            axum::middleware::from_fn_with_state(config, security_middleware),
        );

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Check security headers are present
        let headers = response.headers();
        assert!(headers.contains_key("content-security-policy"));
        assert!(headers.contains_key("strict-transport-security"));
        assert!(headers.contains_key("x-frame-options"));
        assert!(headers.contains_key("x-content-type-options"));
        assert!(headers.contains_key("referrer-policy"));
        assert!(headers.contains_key("x-request-id"));
        assert!(headers.contains_key("server"));
    }

    #[tokio::test]
    async fn test_request_size_limit() {
        let mut config = SecurityConfig::default();
        config.max_request_size = 100; // Very small limit
        let app = Router::new().route("/test", get(|| async { "OK" })).layer(
            axum::middleware::from_fn_with_state(config, security_middleware),
        );

        let large_body = "x".repeat(200); // Body larger than limit
        let request = Request::builder()
            .uri("/test")
            .method(Method::POST)
            .header("content-length", "200")
            .body(Body::from(large_body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_path_traversal_blocked() {
        let config = SecurityConfig::default();
        let app = Router::new().route("/test", get(|| async { "OK" })).layer(
            axum::middleware::from_fn_with_state(config, security_middleware),
        );

        // Test path traversal attempt
        let request = Request::builder()
            .uri("/../../../etc/passwd")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_xss_attempt_blocked() {
        let config = SecurityConfig::default();
        let app = Router::new().route("/test", get(|| async { "OK" })).layer(
            axum::middleware::from_fn_with_state(config, security_middleware),
        );

        // Test XSS attempt in query parameter
        let request = Request::builder()
            .uri("/test?q=<script>alert(1)</script>")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_sql_injection_attempt_blocked() {
        let config = SecurityConfig::default();
        let app = Router::new().route("/test", get(|| async { "OK" })).layer(
            axum::middleware::from_fn_with_state(config, security_middleware),
        );

        // Test SQL injection attempt
        let request = Request::builder()
            .uri("/test?id=1' UNION SELECT * FROM users--")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_request_validation_middleware() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(request_validation_middleware));

        // Test invalid method
        let request = Request::builder()
            .uri("/test")
            .method(Method::TRACE)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_uri_validation() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(request_validation_middleware));

        // Test URI with null byte
        let request = Request::builder()
            .uri("/test%00")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_security_disabled() {
        let config = SecurityConfig {
            enabled: false,
            ..Default::default()
        };
        let app = Router::new().route("/test", get(|| async { "OK" })).layer(
            axum::middleware::from_fn_with_state(config, security_middleware),
        );

        // Even with suspicious URI, should pass when security is disabled
        let request = Request::builder()
            .uri("/test/../../../etc/passwd")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_hsts_header() {
        let mut config = SecurityConfig::default();
        config.hsts_enabled = true;
        config.hsts_max_age = 86400; // 1 day
        config.hsts_include_subdomains = true;

        let app = Router::new().route("/test", get(|| async { "OK" })).layer(
            axum::middleware::from_fn_with_state(config, security_middleware),
        );

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        let hsts_header = response.headers().get("strict-transport-security").unwrap();
        assert_eq!(hsts_header, "max-age=86400; includeSubDomains");
    }

    #[tokio::test]
    async fn test_content_security_policy() {
        let mut config = SecurityConfig::default();
        config.content_security_policy = Some("default-src 'self'".to_string());

        let app = Router::new().route("/test", get(|| async { "OK" })).layer(
            axum::middleware::from_fn_with_state(config, security_middleware),
        );

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        let csp_header = response.headers().get("content-security-policy").unwrap();
        assert_eq!(csp_header, "default-src 'self'");
    }
}
