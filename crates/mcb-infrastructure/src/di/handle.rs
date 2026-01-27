//! Generic Provider Handle - Runtime-swappable provider wrapper
//!
//! A single generic implementation replacing four specific handle types.
//! Wraps providers in RwLock to allow runtime reconfiguration via admin API.
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
/// Wraps any provider type in a RwLock, allowing admin API to switch
/// providers without restarting the application.
///
/// # Type Parameters
///
/// * `T` - Provider trait type (e.g., `dyn EmbeddingProvider`)
///
/// # Example
///
/// ```ignore
/// use mcb_infrastructure::di::Handle;
/// use mcb_domain::ports::providers::EmbeddingProvider;
///
/// // Create handle with initial provider
/// let handle = Handle::new(Arc::new(my_provider));
///
/// // Get current provider
/// let provider = handle.get();
///
/// // Switch to new provider (admin API)
/// handle.set(Arc::new(new_provider));
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
            .expect("Handle lock poisoned") // mcb-validate-ignore: lock_poisoning_recovery
            .clone()
    }

    /// Set a new provider (used by admin service)
    ///
    /// Replaces the current provider with a new one. Existing references
    /// to the old provider remain valid until dropped.
    pub fn set(&self, new_provider: Arc<T>) {
        *self.inner.write().expect("Handle lock poisoned") = new_provider; // mcb-validate-ignore: lock_poisoning_recovery
    }
}

impl<T: ?Sized + Send + Sync> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle").finish()
    }
}
