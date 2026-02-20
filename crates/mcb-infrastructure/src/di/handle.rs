//! Generic Provider Handle - Runtime-swappable provider wrapper
//!
//! A single generic implementation replacing four specific handle types.
//! Wraps providers in `RwLock` to allow runtime reconfiguration via admin API.
//!
//! ## Pattern
//!
//! ```text
//! linkme registry → Resolver → Handle<T> (RwLock) → Domain Services
//!                      ↑
//!              AdminService.switch_provider()
//! ```

use std::sync::{Arc, RwLock};

/// Generic handle for runtime-swappable providers
///
/// Wraps any provider type in a `RwLock`, allowing admin API to switch
/// providers without restarting the application.
///
/// # Type Parameters
///
/// * `T` - Provider trait type (e.g., `dyn EmbeddingProvider`)
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use mcb_infrastructure::di::Handle;
///
/// // Handle works with any Send + Sync trait object
/// trait MyTrait: Send + Sync {
///     fn name(&self) -> &str;
/// }
///
/// struct MyImpl;
/// impl MyTrait for MyImpl {
///     fn name(&self) -> &str { "impl" }
/// }
///
/// // Create handle with initial provider
/// let handle: Handle<dyn MyTrait> = Handle::new(Arc::new(MyImpl));
///
/// // Get current provider
/// let provider = handle.get();
/// assert_eq!(provider.name(), "impl");
/// ```
pub struct Handle<T: ?Sized + Send + Sync> {
    inner: RwLock<Arc<T>>,
}

impl<T: ?Sized + Send + Sync> Handle<T> {
    /// Create a new handle with an initial provider
    pub fn new(provider: Arc<T>) -> Self {
        Self {
            inner: RwLock::new(provider),
        }
    }

    /// Get the current provider
    ///
    /// Returns a cloned Arc to the current provider, allowing shared access.
    pub fn get(&self) -> Arc<T> {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| {
                tracing::error!(
                    component = "Handle",
                    action = "get",
                    "lock poisoned, recovering with poisoned value"
                );
                poisoned.into_inner()
            })
            .clone()
    }

    /// Set a new provider (used by admin service)
    ///
    /// Replaces the current provider with a new one. Existing references
    /// to the old provider remain valid until dropped.
    pub fn set(&self, new_provider: Arc<T>) {
        *self.inner.write().unwrap_or_else(|poisoned| {
            tracing::error!(
                component = "Handle",
                action = "set",
                "lock poisoned during set, recovering"
            );
            poisoned.into_inner()
        }) = new_provider;
    }
}

impl<T: ?Sized + Send + Sync> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle").finish()
    }
}
