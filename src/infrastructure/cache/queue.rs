//! Queue operations extension for CacheProvider
//!
//! This module provides generic queue operations for cache providers.
//! Since generic methods don't work with dyn traits, this extension trait
//! allows code to use queue operations on concrete cache providers.

use crate::domain::error::Result;
use crate::infrastructure::cache::CacheProvider;
use async_trait::async_trait;
use std::time::Duration;

/// Extension trait for queue operations on cache providers
/// Use this with concrete cache providers that need queue functionality
#[async_trait]
pub trait CacheProviderQueue: CacheProvider {
    /// Enqueue an item in a queue
    async fn enqueue_item<T: serde::Serialize + Send>(
        &self,
        namespace: &str,
        queue_name: &str,
        item: T,
    ) -> Result<()> {
        let serialized = serde_json::to_vec(&item).map_err(|e| {
            crate::domain::error::Error::generic(format!("Failed to serialize item: {}", e))
        })?;
        self.set(
            namespace,
            queue_name,
            serialized,
            Duration::from_secs(86400),
        )
        .await
    }

    /// Get all items from a queue
    async fn get_queue<T: serde::de::DeserializeOwned>(
        &self,
        namespace: &str,
        queue_name: &str,
    ) -> Result<Vec<T>> {
        match self.get(namespace, queue_name).await? {
            Some(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
                crate::domain::error::Error::generic(format!("Failed to deserialize queue: {}", e))
            }),
            None => Ok(Vec::new()),
        }
    }

    /// Remove an item from a queue
    async fn remove_item<T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + Send>(
        &self,
        namespace: &str,
        queue_name: &str,
        item: T,
    ) -> Result<()> {
        let mut queue: Vec<T> = self.get_queue(namespace, queue_name).await?;
        queue.retain(|q| q != &item);
        let serialized = serde_json::to_vec(&queue).map_err(|e| {
            crate::domain::error::Error::generic(format!("Failed to serialize queue: {}", e))
        })?;
        self.set(
            namespace,
            queue_name,
            serialized,
            Duration::from_secs(86400),
        )
        .await
    }
}

// Implement for all types that implement CacheProvider
impl<T: CacheProvider + ?Sized> CacheProviderQueue for T {}
