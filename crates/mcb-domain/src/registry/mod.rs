//! Provider Registry System (CA/DI backbone)
//!
//! This module implements the project's **Static Dependency Injection** pattern.
//! Unlike dynamic DI containers that resolve at runtime via reflection,
//! MCB uses `linkme` to perform **Link-Time Registration** (Static CA).
//!
//! ### Core DI Components
//!
//! 1. **Distributed Slices**: Global arrays declared with `#[linkme::distributed_slice]`
//!    that serve as the registry "slots" during the build process.
//! 2. **Provider Entries**: Metadata structs (e.g. [`embedding::EmbeddingProviderEntry`])
//!    that store factory functions and canonical provider names.
//! 3. **Service Resolution**: Submodules (e.g., [`services`]) define factory logic
//!    to instantiate complex domain services using registered providers.
//! 4. **Resolution Context**: The [`ServiceResolutionContext`] provides a shared pool
//!    of infrastructure dependencies (Database, Config) to avoid circular imports.
//!
//! This approach ensures zero-cost discovery, compile-time safety (no missing imports),
//! and strict separation between domain (interfaces) and implementations (providers).
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Static DI / Registration Flow                 │
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

/// Admin operations provider registry.
pub mod admin_operations;
/// Database provider registry.
pub mod database;
/// Embedding provider registry.
pub mod embedding;
/// Event bus provider registry.
pub mod events;
/// Language services provider registry.
pub mod language;
/// Project detection provider registry.
pub mod project_detection;
/// Service registry (context, indexing, search, memory, agent session, validation).
pub mod services;

pub use admin_operations::{
    INDEXING_OPERATIONS_PROVIDERS, IndexingOperationsProviderConfig,
    IndexingOperationsProviderEntry, VALIDATION_OPERATIONS_PROVIDERS,
    ValidationOperationsProviderConfig, ValidationOperationsProviderEntry,
    list_indexing_operations_providers, list_validation_operations_providers,
    resolve_indexing_operations_provider, resolve_validation_operations_provider,
};
pub use services::{
    AGENT_SESSION_SERVICE_NAME, CONTEXT_SERVICE_NAME, HIGHLIGHT_SERVICE_NAME,
    INDEXING_SERVICE_NAME, MEMORY_SERVICE_NAME, SEARCH_SERVICE_NAME, SERVICES_REGISTRY,
    ServiceBuilder, ServiceRegistryEntry, VALIDATION_SERVICE_NAME, resolve_agent_session_service,
    resolve_context_service, resolve_highlight_service, resolve_indexing_service,
    resolve_memory_service, resolve_search_service, resolve_validation_service,
};
pub mod validation;
pub use validation::{
    VALIDATOR_ENTRIES, ValidatorEntry, build_all_validators, build_named_validators, build_report,
    list_validator_entries, list_validator_names, validator_count, violation_to_entry,
};
/// GraphQL schema provider registry.
pub mod graphql;
/// Hybrid search provider registry.
pub mod hybrid_search;
/// DI resolution context (opaque DB/config, domain ports).
pub mod resolution_context;
/// VCS provider registry.
pub mod vcs;
/// Vector store provider registry.
pub mod vector_store;
pub use resolution_context::ServiceResolutionContext;
