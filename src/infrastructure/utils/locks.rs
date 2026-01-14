//! RwLock utilities - Eliminates repetitive lock poisoning error handling (DRY)
//!
//! Provides convenient extension methods for common RwLock patterns

use crate::domain::error::{Error, Result};
use std::sync::RwLock;

/// Extension trait for RwLock providing convenient access patterns
pub trait RwLockExt<T> {
    /// Acquire read lock and extract value, handling poisoning
    ///
    /// Replaces: `.read().map_err(|_| Error::generic("Lock poisoned"))?`
    fn read_guard(&self) -> Result<std::sync::RwLockReadGuard<'_, T>>;

    /// Acquire write lock and extract value, handling poisoning
    ///
    /// Replaces: `.write().map_err(|_| Error::generic("Lock poisoned"))?`
    fn write_guard(&self) -> Result<std::sync::RwLockWriteGuard<'_, T>>;

    /// Extract value from read lock without holding lock
    ///
    /// # Arguments
    /// * `extractor` - Closure that extracts value from &T
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::RwLock;
    /// use mcp_context_browser::infrastructure::utils::locks::RwLockExt;
    ///
    /// fn example() -> mcp_context_browser::domain::error::Result<()> {
    ///     let lock = RwLock::new("hello".to_string());
    ///     let value: String = lock.extract(|data| data.clone())?;
    ///     assert_eq!(value, "hello");
    ///     Ok(())
    /// }
    /// ```
    fn extract<F, R>(&self, extractor: F) -> Result<R>
    where
        F: FnOnce(&T) -> R;

    /// Extract and transform value from read lock
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::RwLock;
    /// use mcp_context_browser::infrastructure::utils::locks::RwLockExt;
    ///
    /// fn example() -> mcp_context_browser::domain::error::Result<()> {
    ///     let lock = RwLock::new(vec![1, 2, 3]);
    ///     let count: usize = lock.extract_map(|data| data.len())?;
    ///     assert_eq!(count, 3);
    ///     Ok(())
    /// }
    /// ```
    fn extract_map<F, R>(&self, mapper: F) -> Result<R>
    where
        F: FnOnce(&T) -> R;
}

impl<T> RwLockExt<T> for RwLock<T> {
    fn read_guard(&self) -> Result<std::sync::RwLockReadGuard<'_, T>> {
        self.read().map_err(|_| Error::generic("Lock poisoned"))
    }

    fn write_guard(&self) -> Result<std::sync::RwLockWriteGuard<'_, T>> {
        self.write().map_err(|_| Error::generic("Lock poisoned"))
    }

    fn extract<F, R>(&self, extractor: F) -> Result<R>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.read_guard()?;
        Ok(extractor(&*guard))
    }

    fn extract_map<F, R>(&self, mapper: F) -> Result<R>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.read_guard()?;
        Ok(mapper(&*guard))
    }
}

/// Extension trait for async RwLock (tokio) providing convenient access patterns
///
/// Note: Due to lifetime constraints, async guard methods must be called directly on the lock.
/// Use extract_async and extract_map_async for safe access patterns without manual guard management.
#[allow(async_fn_in_trait)]
pub trait AsyncRwLockExt<T> {
    /// Extract value from read lock asynchronously without holding lock
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mcp_context_browser::infrastructure::utils::locks::AsyncRwLockExt;
    /// use tokio::sync::RwLock;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let lock = RwLock::new("hello".to_string());
    ///     let value: String = lock.extract_async(|data| data.clone()).await?;
    ///     assert_eq!(value, "hello");
    ///     Ok(())
    /// }
    /// ```
    async fn extract_async<F, R>(&self, extractor: F) -> Result<R>
    where
        F: FnOnce(&T) -> R;

    /// Extract and transform value from read lock asynchronously
    async fn extract_map_async<F, R>(&self, mapper: F) -> Result<R>
    where
        F: FnOnce(&T) -> R;
}

impl<T: Send + Sync> AsyncRwLockExt<T> for tokio::sync::RwLock<T> {
    async fn extract_async<F, R>(&self, extractor: F) -> Result<R>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.read().await;
        Ok(extractor(&*guard))
    }

    async fn extract_map_async<F, R>(&self, mapper: F) -> Result<R>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.read().await;
        Ok(mapper(&*guard))
    }
}
