//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#dependency-injection)
//!
//! Provider Handles - Runtime-swappable provider wrappers
//!
//! Type aliases for the generic `Handle<T>`.
//! All handle types use the same underlying generic implementation.
//!
//! ## Pattern
//!
//! ```text
//! linkme registry → Resolver → Handle<T> (RwLock) → Domain Services
//!                      ↑
//!              AdminService.switch_provider()
//! ```

use mcb_domain::ports::{
    CacheProvider, EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};

use super::handle::Handle;

// ============================================================================
// Type Aliases
// ============================================================================

/// Handle for runtime-swappable embedding provider
///
/// Wraps the current embedding provider in a `RwLock`, allowing admin API
/// to switch providers without restarting the application.
pub type EmbeddingProviderHandle = Handle<dyn EmbeddingProvider>;

/// Handle for runtime-swappable vector store provider
///
/// Wraps the current vector store provider in a `RwLock`, allowing admin API
/// to switch providers without restarting the application.
pub type VectorStoreProviderHandle = Handle<dyn VectorStoreProvider>;

/// Handle for runtime-swappable cache provider
///
/// Wraps the current cache provider in a `RwLock`, allowing admin API
/// to switch providers without restarting the application.
pub type CacheProviderHandle = Handle<dyn CacheProvider>;

/// Handle for runtime-swappable language chunking provider
///
/// Wraps the current language chunking provider in a `RwLock`, allowing admin API
/// to switch providers without restarting the application.
pub type LanguageProviderHandle = Handle<dyn LanguageChunkingProvider>;
