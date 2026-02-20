//! Dependency Injection System - dill `IoC` + Handle-based Pattern + linkme Registry
//!
//! **Documentation**: [`docs/modules/di.md`](../../../../docs/modules/di.md)
//!
//! This module implements dependency injection using:
//! - **dill Catalog**: `IoC` container for service resolution
//! - **Handle-based pattern**: Runtime-swappable provider handles with `RwLock`
//! - **linkme registry**: Compile-time discovery of external providers
//!
//! ## Architecture
//!
//! ```text
//! linkme (compile-time)     dill Catalog (runtime)     Handle-based
//! ─────────────────────     ─────────────────────      ────────────
//! EMBEDDING_PROVIDERS  →    Resolver → add_value() →   Handle (RwLock)
//! (list of factories)                                       ↓
//!                                                     AdminService
//!                                                    (switch via API)
//! ```
//!
//! ## Key Principles
//!
//! - **`IoC` Container**: dill Catalog manages service lifecycle
//! - **Trait-based DI**: All dependencies injected as `Arc<dyn Trait>`
//! - **Composition Root**: Services composed in catalog.rs `build_catalog()`
//! - **Runtime Switching**: Providers can be changed via admin API
//! - **Testability**: Default local providers (`FastEmbed`, `EdgeVec`) enable isolated testing
//!
//! ## For Consumer Crates (mcb-server tests, golden tests, etc.)
//!
//! - `create_test_dependencies()` — isolated repos + shared providers from `AppContext`
//! - `create_memory_repository()` / `create_memory_repository_with_executor()` — standalone repos
//! - `create_vcs_entity_repository()` — standalone VCS repo
//!
//! ## Config-Driven Initialization
//!
//! All providers initialize from `config/default.toml` + overrides.
//! Use `TestConfigBuilder` for test-specific overrides.
//! **NEVER import from `mcb_providers` directly outside this crate.**

pub mod admin;
pub mod bootstrap;
pub mod catalog;
pub mod database_resolver;
pub mod handle;
pub mod handles;
pub mod modules;
pub mod provider_resolvers;
pub mod repositories;
pub mod resolver;
pub mod test_factory;
pub mod vcs;

pub use admin::{
    AdminService, CacheAdminInterface, CacheAdminService, EmbeddingAdminInterface,
    EmbeddingAdminService, LanguageAdminInterface, LanguageAdminService, ProviderInfo,
    ProviderResolver, VectorStoreAdminInterface, VectorStoreAdminService,
};
pub use bootstrap::*;
pub use catalog::build_catalog;
pub use handle::Handle;
pub use handles::{
    CacheHandleExt, CacheProviderHandle, EmbeddingHandleExt, EmbeddingProviderHandle,
    LanguageProviderHandle, VectorStoreProviderHandle,
};
pub use modules::{DomainServicesContainer, DomainServicesFactory, ServiceDependencies};
pub use provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};
pub use repositories::{
    create_memory_repository, create_memory_repository_with_executor, create_vcs_entity_repository,
};
pub use resolver::{
    AvailableProviders, ResolvedProviders, list_available_providers, resolve_providers,
};
pub use test_factory::create_test_dependencies;
pub use vcs::default_vcs_provider;
