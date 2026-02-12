//! Cache configuration utilities
//!
//! Infrastructure-specific cache utilities only.
//! Type definitions (CacheEntryConfig, CacheStats) are in mcb-domain.
//! Use mcb_domain::ports::providers::cache::{CacheEntryConfig, CacheStats} directly.

use mcb_domain::error::{Error, Result};

/// Cache key utilities
pub struct CacheKey;

impl CacheKey {
    /// Create a namespaced cache key
    pub fn namespaced(namespace: &str, key: &str) -> String {
        format!("{}:{}", namespace, key)
    }

    /// Extract namespace from a namespaced key
    pub fn extract_namespace(key: &str) -> Option<&str> {
        key.split(':').next()
    }

    /// Extract the key part from a namespaced key
    pub fn extract_key(key: &str) -> &str {
        key.split_once(':').map(|x| x.1).unwrap_or(key)
    }

    /// Validate cache key format
    pub fn validate_key(key: &str) -> Result<()> {
        if key.is_empty() {
            return Err(Error::Configuration {
                message: "Cache key cannot be empty".to_string(),
                source: None,
            });
        }

        if key.len() > 250 {
            return Err(Error::Configuration {
                message: "Cache key too long (max 250 characters)".to_string(),
                source: None,
            });
        }

        // Check for invalid characters
        if key
            .chars()
            .any(|c| c.is_control() || c == '\n' || c == '\r')
        {
            return Err(Error::Configuration {
                message: "Cache key contains invalid characters".to_string(),
                source: None,
            });
        }

        Ok(())
    }

    /// Sanitize a cache key by removing/replacing invalid characters
    pub fn sanitize_key(key: &str) -> String {
        key.chars()
            .map(|c| {
                if c.is_control() || c == '\n' || c == '\r' {
                    '_'
                } else {
                    c
                }
            })
            .take(250)
            .collect()
    }
}
