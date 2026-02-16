//! Cache configuration utilities
//!
//! Infrastructure-specific cache utilities only.
//! Type definitions (`CacheEntryConfig`, `CacheStats`) are in mcb-domain.
//! Use `mcb_domain::ports::providers::cache::{CacheEntryConfig`, `CacheStats`} directly.

use mcb_domain::error::{Error, Result};

/// Maximum allowed length for cache keys.
const CACHE_KEY_MAX_LENGTH: usize = 250;

/// Cache key utilities
pub struct CacheKey;

impl CacheKey {
    /// Create a namespaced cache key
    #[must_use]
    pub fn namespaced(namespace: &str, key: &str) -> String {
        format!("{namespace}:{key}")
    }

    /// Extract namespace from a namespaced key
    #[must_use]
    pub fn extract_namespace(key: &str) -> Option<&str> {
        key.split(':').next()
    }

    /// Extract the key part from a namespaced key
    #[must_use]
    pub fn extract_key(key: &str) -> &str {
        key.split_once(':').map_or(key, |x| x.1)
    }

    /// Validate cache key format
    ///
    /// # Errors
    ///
    /// Returns an error if the key is empty, too long, or contains invalid characters.
    pub fn validate_key(key: &str) -> Result<()> {
        if key.is_empty() {
            return Err(Error::Configuration {
                message: "Cache key cannot be empty".to_owned(),
                source: None,
            });
        }

        if key.len() > CACHE_KEY_MAX_LENGTH {
            return Err(Error::Configuration {
                message: format!("Cache key too long (max {CACHE_KEY_MAX_LENGTH} characters)"),
                source: None,
            });
        }

        // Check for invalid characters
        if key
            .chars()
            .any(|c| c.is_control() || c == '\n' || c == '\r')
        {
            return Err(Error::Configuration {
                message: "Cache key contains invalid characters".to_owned(),
                source: None,
            });
        }

        Ok(())
    }

    /// Sanitize a cache key by removing/replacing invalid characters
    #[must_use]
    pub fn sanitize_key(key: &str) -> String {
        key.chars()
            .map(|c| {
                if c.is_control() || c == '\n' || c == '\r' {
                    '_'
                } else {
                    c
                }
            })
            .take(CACHE_KEY_MAX_LENGTH)
            .collect()
    }
}
