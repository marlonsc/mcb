//! Provider Registry System
//!
//! Defines the auto-registration infrastructure for plugin providers.
//! Uses the `linkme` crate for compile-time registration of providers
//! that can be discovered and instantiated at runtime.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Provider Registration Flow                    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  1. Provider defines:  #[linkme::distributed_slice(PROVIDERS)]  │
//! │                        static ENTRY: ProviderEntry = ...        │
//! │                              ↓                                  │
//! │  2. Registry declares: #[linkme::distributed_slice]             │
//! │                        pub static PROVIDERS: [Entry] = [..]     │
//! │                              ↓                                  │
//! │  3. Resolver queries:  PROVIDERS.iter()                         │
//! │                              ↓                                  │
//! │  4. Config selects:    "provider = ollama" → OllamaProvider     │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ### Registering a Provider (in mcb-providers)
//!
//! ```no_run
//! use mcb_application::ports::registry::embedding::{EmbeddingProviderEntry, EMBEDDING_PROVIDERS};
//! use mcb_domain::ports::providers::EmbeddingProvider;
//! use std::sync::Arc;
//!
//! // Providers register via #[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
//! // See mcb-providers for implementation examples
//! ```
//!
//! ### Resolving a Provider (in mcb-infrastructure)
//!
//! ```no_run
//! use mcb_application::ports::registry::embedding::{EmbeddingProviderConfig, resolve_embedding_provider};
//!
//! fn get_provider() -> Result<(), String> {
//!     let config = EmbeddingProviderConfig::new("null");
//!     let provider = resolve_embedding_provider(&config)?;
//!     println!("Using provider: {}", provider.provider_name());
//!     Ok(())
//! }
//! ```

pub mod cache;
pub mod embedding;
pub mod language;
pub mod vector_store;

// Re-export all registry types and functions
pub use cache::{
    CACHE_PROVIDERS, CacheProviderConfig, CacheProviderEntry, list_cache_providers,
    resolve_cache_provider,
};
pub use embedding::{
    EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry, list_embedding_providers,
    resolve_embedding_provider,
};
pub use language::{
    LANGUAGE_PROVIDERS, LanguageProviderConfig, LanguageProviderEntry, list_language_providers,
    resolve_language_provider,
};
pub use vector_store::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
    list_vector_store_providers, resolve_vector_store_provider,
};
