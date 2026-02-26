//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
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

pub mod database;
pub mod embedding;
pub mod events;
pub mod language;
/// Project detection provider registry.
pub mod project_detection;
/// Service registry (context, indexing, search, memory, agent session, validation).
pub mod services;

pub use services::{
    AGENT_SESSION_SERVICE_NAME, CONTEXT_SERVICE_NAME, INDEXING_SERVICE_NAME, MEMORY_SERVICE_NAME,
    SEARCH_SERVICE_NAME, SERVICES_REGISTRY, ServiceBuilder, ServiceRegistryEntry,
    VALIDATION_SERVICE_NAME, resolve_agent_session_service, resolve_context_service,
    resolve_indexing_service, resolve_memory_service, resolve_search_service,
    resolve_validation_service,
};
pub mod validation;
pub use validation::{
    VALIDATION_PROVIDERS, VALIDATOR_ENTRIES, ValidationProviderConfig, ValidatorEntry,
    build_validators, list_validator_entries, list_validator_names, run_validators,
};
/// VCS provider registry.
pub mod vcs;
/// Vector store provider registry.
pub mod vector_store;
