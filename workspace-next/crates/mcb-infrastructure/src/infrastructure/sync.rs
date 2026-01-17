//! Distributed Lock Adapter
//!
//! Null implementation of the distributed lock port for testing.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::{LockGuard, LockProvider};

/// Null implementation for testing and Shaku DI default
#[derive(shaku::Component)]
#[shaku(interface = LockProvider)]
pub struct NullLockProvider;

impl NullLockProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullLockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LockProvider for NullLockProvider {
    async fn acquire_lock(&self, key: &str) -> Result<LockGuard> {
        Ok(LockGuard {
            key: key.to_string(),
            token: "null-token".to_string(),
        })
    }

    async fn release_lock(&self, _guard: LockGuard) -> Result<()> {
        Ok(())
    }
}
