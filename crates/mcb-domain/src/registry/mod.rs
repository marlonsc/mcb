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
//! use mcb_domain::registry::embedding::{EmbeddingProviderEntry, EMBEDDING_PROVIDERS};
//! use mcb_domain::ports::EmbeddingProvider;
//! use std::sync::Arc;
//!
//! // Providers register via #[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
//! // See mcb-providers for implementation examples
//! ```
//!
//! ### Resolving a Provider (in mcb-infrastructure)
//!
//! ```no_run
//! use mcb_domain::registry::embedding::{EmbeddingProviderConfig, resolve_embedding_provider};
//!
//! fn get_provider() -> Result<(), mcb_domain::Error> {
//!     let config = EmbeddingProviderConfig::new("null");
//!     let provider = resolve_embedding_provider(&config)?;
//!     println!("Using provider: {}", provider.provider_name());
//!     Ok(())
//! }
//! ```

pub mod cache;
pub mod database;
pub mod embedding;
pub mod language;
pub mod validation;
pub mod vector_store;
