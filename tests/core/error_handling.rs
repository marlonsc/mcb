//! Comprehensive error handling tests for core modules
//!
//! This test suite validates that all core modules properly handle errors
//! without using unwrap/expect, ensuring robust error propagation.

use mcp_context_browser::core::error::{Error, Result};
use mcp_context_browser::core::auth::{AuthService, AuthConfig, UserRole, Permission};
use mcp_context_browser::core::cache::{CacheManager, CacheConfig};
use std::time::Duration;

#[cfg(test)]
mod auth_error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_service_handles_disabled_auth_errors() {
        let config = AuthConfig {
            enabled: false,
            ..Default::default()
        };
        let auth = AuthService::new(config);

        // Should return proper error instead of panicking
        let result = auth.authenticate("user", "pass").await;
        assert!(matches!(result, Err(Error::Generic(_))));
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }

    #[tokio::test]
    async fn test_auth_service_handles_invalid_credentials_errors() {
        let auth = AuthService::default();

        // Should return proper error instead of panicking
        let result = auth.authenticate("invalid@email.com", "wrongpass").await;
        assert!(matches!(result, Err(Error::Generic(_))));
        assert!(result.unwrap_err().to_string().contains("Invalid credentials"));
    }

    #[tokio::test]
    async fn test_auth_service_handles_token_validation_errors() {
        let auth = AuthService::default();

        // Should return proper error for invalid tokens instead of panicking
        let result = auth.validate_token("invalid.jwt.token");
        assert!(matches!(result, Err(Error::Generic(_))));
        assert!(result.unwrap_err().to_string().contains("Invalid token"));

        // Should return proper error for expired tokens instead of panicking
        let result = auth.validate_token("expired.token.here");
        assert!(matches!(result, Err(Error::Generic(_))));
    }

    #[tokio::test]
    async fn test_auth_service_handles_token_generation_errors() {
        let auth = AuthService::default();

        // This should work in normal cases, but we test the error handling path
        let result = auth.authenticate("admin@context.browser", "admin").await;
        assert!(result.is_ok(), "Authentication should succeed with valid credentials");
    }
}

#[cfg(test)]
mod cache_error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_manager_handles_connection_failures() {
        // Test with invalid Redis configuration
        let config = CacheConfig {
            redis_url: "redis://invalid:6379".to_string(),
            default_ttl_seconds: 300,
            max_size: 100,
            enabled: true,
            namespaces: Default::default(),
        };

        // In Exclusive mode, this should FAIL on creation
        let result = CacheManager::new(config).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Redis { .. } | Error::Generic(_)));
    }

    #[tokio::test]
    async fn test_cache_manager_handles_disabled_cache_operations() {
        let config = CacheConfig {
            redis_url: "".to_string(),
            default_ttl_seconds: 300,
            max_size: 0, // Disabled
            enabled: false,
            namespaces: Default::default(),
        };

        let manager = CacheManager::new(config).await.unwrap();

        // Operations on disabled cache should not panic
        let set_result = manager.set("test", "key", "value".to_string()).await;
        assert!(set_result.is_ok()); // Should succeed (no-op)

        let get_result: CacheResult<String> = manager.get("test", "key").await;
        assert!(get_result.is_miss());
    }

    #[tokio::test]
    async fn test_cache_manager_handles_namespace_operations() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config).await.unwrap();

        // These operations should not panic
        let clear_result = manager.clear_namespace("test_ns").await;
        assert!(clear_result.is_ok());

        let delete_result = manager.delete("test_ns", "key").await;
        assert!(delete_result.is_ok());

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    async fn test_cache_manager_handles_large_data_operations() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config).await.unwrap();

        // Test with large data that might cause issues
        let large_data = "x".repeat(1024 * 1024); // 1MB string

        let set_result = manager.set("test", "large_key", large_data.clone()).await;
        assert!(set_result.is_ok());

        let get_result: CacheResult<String> = manager.get("test", "large_key").await;
        assert!(get_result.is_hit());
        assert_eq!(get_result.data().unwrap(), large_data);
    }
}

#[cfg(test)]
mod crypto_error_handling_tests {
    use super::*;
    use mcp_context_browser::core::crypto::{CryptoService, CryptoConfig};

    #[tokio::test]
    async fn test_crypto_service_handles_disabled_crypto_operations() {
        let config = CryptoConfig {
            enabled: false,
            master_key_path: None,
            key_rotation_days: 30,
        };

        let crypto = CryptoService::new(config).await.unwrap();

        // Operations on disabled crypto should not panic
        let encrypt_result = crypto.encrypt("test data".as_bytes()).await;
        assert!(encrypt_result.is_ok()); // Should succeed (no-op) or return proper error

        let decrypt_result = crypto.decrypt(&[1, 2, 3]).await;
        assert!(decrypt_result.is_ok()); // Should succeed (no-op) or return proper error
    }

    #[tokio::test]
    async fn test_crypto_service_handles_key_generation_errors() {
        let config = CryptoConfig::default();
        let crypto = CryptoService::new(config).await.unwrap();

        // Key generation should not panic
        let key_result = crypto.generate_data_key().await;
        assert!(key_result.is_ok());
    }
}

#[cfg(test)]
mod database_error_handling_tests {
    use super::*;
    use mcp_context_browser::core::database::{DatabaseService, DatabaseConfig};

    #[tokio::test]
    async fn test_database_service_handles_disabled_database_operations() {
        let config = DatabaseConfig {
            enabled: false,
            connection_string: None,
            max_connections: 10,
            connection_timeout_secs: 30,
        };

        let db = DatabaseService::new(config).await.unwrap();

        // Operations on disabled database should not panic
        let health_result = db.health_check().await;
        assert!(health_result.is_ok()); // Should succeed or return proper error
    }
}

#[cfg(test)]
mod integration_error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_core_services_handle_cascading_errors() {
        // Test that errors propagate correctly through service layers
        // This ensures no unwrap/expect calls break the error chain

        let auth = AuthService::default();
        let cache_config = CacheConfig::default();
        let cache = CacheManager::new(cache_config).await.unwrap();

        // Test auth failure doesn't crash the system
        let auth_result = auth.authenticate("nonexistent", "wrong").await;
        assert!(auth_result.is_err());

        // Test cache operations still work after auth failure
        let cache_result = cache.set("test", "key", "value".to_string()).await;
        assert!(cache_result.is_ok());
    }

    #[tokio::test]
    async fn test_error_context_preservation() {
        // Test that error context is preserved through multiple layers
        let auth = AuthService::default();

        let result = auth.authenticate("", "").await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        // Error should contain useful context, not just "Generic error"
        let error_msg = error.to_string();
        assert!(!error_msg.is_empty());
        assert!(!error_msg.contains("called `Result::unwrap()`"));
    }
}