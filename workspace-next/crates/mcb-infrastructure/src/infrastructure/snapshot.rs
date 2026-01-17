//! State Store Adapter
//!
//! Null implementation of the state store port for testing.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::StateStoreProvider;

/// Null implementation for testing and Shaku DI default
#[derive(shaku::Component)]
#[shaku(interface = StateStoreProvider)]
pub struct NullStateStoreProvider;

impl NullStateStoreProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullStateStoreProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StateStoreProvider for NullStateStoreProvider {
    async fn save(&self, _key: &str, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn load(&self, _key: &str) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn delete(&self, _key: &str) -> Result<()> {
        Ok(())
    }
}
