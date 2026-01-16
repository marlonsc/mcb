//! HTTP Client Pool Tests
//!
//! Tests for the HTTP client pool implementation.

use mcb_infrastructure::adapters::http_client::{HttpClientConfig, HttpClientPool, HttpClientProvider};
use std::time::Duration;

#[test]
fn test_pool_creation() {
    let pool = HttpClientPool::new();
    assert!(pool.is_ok());
}

#[test]
fn test_pool_is_enabled() {
    let pool = HttpClientPool::new().unwrap();
    assert!(pool.is_enabled());
}

#[test]
fn test_pool_config() {
    let pool = HttpClientPool::new().unwrap();
    let config = pool.config();
    assert!(config.timeout.as_secs() > 0);
    assert!(!config.user_agent.is_empty());
}

#[test]
fn test_custom_timeout() {
    let pool = HttpClientPool::new().unwrap();
    let custom_client = pool.client_with_timeout(Duration::from_secs(60));
    assert!(custom_client.is_ok());
}

#[test]
fn test_custom_config() {
    let config = HttpClientConfig::with_timeout(Duration::from_secs(120));
    let pool = HttpClientPool::with_config(config);
    assert!(pool.is_ok());
    assert_eq!(pool.unwrap().config().timeout.as_secs(), 120);
}

#[test]
fn test_pool_clone() {
    let pool = HttpClientPool::new().unwrap();
    let cloned = pool.clone();
    assert!(cloned.is_enabled());
}
