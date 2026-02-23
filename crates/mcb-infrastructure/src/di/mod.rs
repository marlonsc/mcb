//! Dependency Injection — Composition Root + linkme Provider Registry
//!
//! Compile-time provider discovery via `#[distributed_slice]` (linkme),
//! runtime resolution via `provider_resolvers`, and service assembly
//! via `DomainServicesFactory`.

pub mod bootstrap;
pub mod modules;
pub mod provider_resolvers;
pub mod repositories;
pub mod resolver;
pub mod vcs;

pub use bootstrap::*;
pub use modules::{DomainServicesContainer, DomainServicesFactory, ServiceDependencies};
pub use provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};
pub use repositories::{create_memory_repository, create_memory_repository_with_db};
pub use resolver::{
    AvailableProviders, ResolvedProviders, list_available_providers, resolve_providers,
};
pub use vcs::default_vcs_provider;
