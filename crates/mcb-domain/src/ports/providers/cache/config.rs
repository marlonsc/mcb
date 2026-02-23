//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!
#![allow(missing_docs)]

use std::time::Duration;

use serde::{Deserialize, Serialize};

pub const DEFAULT_CACHE_TTL_SECS: u64 = 300;
pub const DEFAULT_CACHE_NAMESPACE: &str = "default";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryConfig {
    pub ttl: Option<Duration>,
    pub namespace: Option<String>,
}

impl CacheEntryConfig {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ttl: Some(Duration::from_secs(DEFAULT_CACHE_TTL_SECS)),
            namespace: None,
        }
    }

    #[must_use]
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    #[must_use]
    pub fn with_ttl_secs(mut self, secs: u64) -> Self {
        self.ttl = Some(Duration::from_secs(secs));
        self
    }

    #[must_use]
    pub fn with_namespace<S: Into<String>>(mut self, namespace: S) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    #[must_use]
    pub fn effective_ttl(&self) -> Duration {
        self.ttl
            .unwrap_or(Duration::from_secs(DEFAULT_CACHE_TTL_SECS))
    }

    #[must_use]
    pub fn effective_namespace(&self) -> String {
        self.namespace
            .clone()
            .unwrap_or_else(|| DEFAULT_CACHE_NAMESPACE.to_owned())
    }
}

impl Default for CacheEntryConfig {
    fn default() -> Self {
        Self::new()
    }
}
