#![allow(missing_docs)]

use async_trait::async_trait;

use crate::error::Result;

use super::{CacheEntryConfig, CacheStats};

#[async_trait]
pub trait CacheProvider: Send + Sync + std::fmt::Debug {
    async fn get_json(&self, key: &str) -> Result<Option<String>>;
    async fn set_json(&self, key: &str, value: &str, config: CacheEntryConfig) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<bool>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn clear(&self) -> Result<()>;
    async fn stats(&self) -> Result<CacheStats>;
    async fn size(&self) -> Result<usize>;
    fn provider_name(&self) -> &str;
}
